import logging
from typing import Optional

from config import EMBEDDING
from db_manager import DatabaseManager
from embedder import Embedder

logger = logging.getLogger(__name__)


class ChatAgent:
    def __init__(self, top_k: int = 5):
        self.top_k = top_k
        self.db = DatabaseManager()
        self.embedder = Embedder()
        self._chat_history: list[dict] = []

    def retrieve(self, query: str) -> list[dict]:
        vector = self.embedder.encode(query)
        cur = self.db.cursor
        cur.execute(
            f"""
            SELECT f.id, f.content, f.chunk_index, d.filename,
                   1 - (f.embedding <=> %s::vector) AS similarity
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            ORDER BY f.embedding <=> %s::vector
            LIMIT %s
            """,
            (vector, vector, self.top_k),
        )
        rows = cur.fetchall()

        fragments = []
        for r in rows:
            cur.execute(
                "SELECT name, type FROM entidades WHERE fragmento_id = %s",
                (r["id"],),
            )
            entities = [{"name": e["name"], "type": e["type"]} for e in cur.fetchall()]
            fragments.append({
                "id": r["id"],
                "content": r["content"],
                "filename": r["filename"],
                "similarity": round(r["similarity"], 4),
                "entities": entities,
            })
        return fragments

    def ask(self, question: str) -> str:
        fragments = self.retrieve(question)
        if not fragments:
            return "No se encontró información relevante."

        context_parts = []
        for i, f in enumerate(fragments, 1):
            context_parts.append(
                f"--- Fragmento {i} ({f['filename']}, similitud: {f['similarity']}) ---"
            )
            context_parts.append(f["content"])
            if f.get("entities"):
                context_parts.append(
                    "Entidades: " + ", ".join(e["name"] for e in f["entities"])
                )
        context = "\n\n".join(context_parts)

        if self._chat_history:
            history_lines = ["Historial de la conversación:"]
            for msg in self._chat_history[-4:]:
                role = "Usuario" if msg["role"] == "user" else "Asistente"
                history_lines.append(f"  {role}: {msg['content'][:200]}")
            context = "\n".join(history_lines) + "\n\n" + context

        from extractor import Extractor
        extractor = Extractor()
        answer = extractor.answer(question, context)
        extractor.close()

        self._chat_history.append({"role": "user", "content": question})
        self._chat_history.append({"role": "assistant", "content": answer})
        return answer
