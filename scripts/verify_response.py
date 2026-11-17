#!/usr/bin/env python3
"""Verify LLM response faithfulness and consistency.

Input (stdin JSON):
{
  "question": "...",
  "answer": "...",
  "contexts": ["...", "..."],
  "citations": [{"id":..., "title":..., "authors":[], "year":...}]
}

Output (stdout JSON):
{
  "faithfulness": 0.0-1.0,
  "hallucination_score": 0.0-1.0,
  "numeric_consistency": 0.0-1.0,
  "citations_valid": bool,
  "overall_pass": bool,
  "details": {...}
}
"""
import argparse, json, sys, re, os

def numeric_consistency(answer: str, contexts: list) -> float:
    # Extract numbers from answer and check presence in contexts
    nums_answer = set(re.findall(r'\b\d+(?:\.\d+)?\b', answer))
    if not nums_answer:
        return 1.0
    ctx_text = " ".join(contexts)
    nums_ctx = set(re.findall(r'\b\d+(?:\.\d+)?\b', ctx_text))
    matched = len(nums_answer & nums_ctx)
    return matched / len(nums_answer) if nums_answer else 1.0

def citations_valid(citations: list) -> bool:
    # Basic: all citations have title or id
    if not citations:
        return True
    return all(c.get("title") or c.get("id") for c in citations)

def ragas_faithfulness(answer: str, contexts: list) -> float:
    # LLM-as-judge via OpenRouter if available, else heuristic
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key or api_key.startswith("tu_"):
        # Heuristic: token overlap
        ans_tokens = set(answer.lower().split())
        ctx_tokens = set(" ".join(contexts).lower().split())
        if not ans_tokens:
            return 1.0
        overlap = len(ans_tokens & ctx_tokens) / len(ans_tokens)
        return round(overlap, 3)
    try:
        import urllib.request
        prompt = (
            "Given the context, is the answer faithful? Return only a number 0.0-1.0.\n\n"
            f"Context:\n{' '.join(contexts)[:2000]}\n\nAnswer:\n{answer[:1000]}\n\nScore:"
        )
        data = json.dumps({
            "model": os.environ.get("OPENROUTER_MODEL", "google/gemma-4-31b-it:free"),
            "messages": [{"role": "user", "content": prompt}],
        }).encode()
        req = urllib.request.Request(
            "https://openrouter.ai/api/v1/chat/completions",
            data=data,
            headers={"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            j = json.loads(resp.read())
            text = j["choices"][0]["message"]["content"].strip()
            # extract first float
            m = re.search(r'(\d+(?:\.\d+)?)', text)
            if m:
                val = float(m.group(1))
                if val > 1.0:
                    val = val / 100.0
                return round(min(1.0, max(0.0, val)), 3)
    except Exception as e:
        sys.stderr.write(f"ragas llm judge failed: {e}\n")
    # fallback heuristic
    ans_tokens = set(answer.lower().split())
    ctx_tokens = set(" ".join(contexts).lower().split())
    if not ans_tokens:
        return 1.0
    return round(len(ans_tokens & ctx_tokens) / len(ans_tokens), 3)

def main():
    raw = sys.stdin.read()
    try:
        data = json.loads(raw)
    except Exception:
        data = {}
    question = data.get("question", "")
    answer = data.get("answer", "")
    contexts = data.get("contexts", [])
    citations = data.get("citations", [])

    faith = ragas_faithfulness(answer, contexts)
    num_cons = numeric_consistency(answer, contexts)
    cites_ok = citations_valid(citations)

    # hallucination score = inverse of faithfulness (simplified)
    hall = round(1.0 - faith, 3)

    overall = faith >= 0.5 and num_cons >= 0.5 and cites_ok

    out = {
        "faithfulness": faith,
        "hallucination_score": hall,
        "numeric_consistency": num_cons,
        "citations_valid": cites_ok,
        "overall_pass": overall,
        "details": {
            "question": question[:200],
            "answer_len": len(answer),
            "contexts": len(contexts),
            "citations": len(citations),
        },
    }
    print(json.dumps(out))

if __name__ == "__main__":
    main()
