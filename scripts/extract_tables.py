#!/usr/bin/env python3
"""Extract tables from PDFs into structured data (CSV/JSON/Parquet).

Pipeline: PyMuPDF (fast) -> Camelot lattice/stream (quality) -> Table Transformer (scanned).

Output: JSON array of tables with page, bbox, headers, rows, method, confidence.
Optionally writes CSV/Parquet files to output_dir.
"""
import argparse
import json
import sys
from pathlib import Path

def extract_pymupdf(pdf_path: str) -> list:
    tables = []
    try:
        import fitz
        doc = fitz.open(pdf_path)
        for page_num in range(len(doc)):
            page = doc[page_num]
            tabs = page.find_tables()
            for t in tabs.tables:
                try:
                    df = t.to_pandas()
                    tables.append({
                        "page": page_num + 1,
                        "bbox": list(t.bbox),
                        "headers": list(df.columns.astype(str)),
                        "rows": df.values.tolist(),
                        "method": "pymupdf",
                        "confidence": float(getattr(t, "confidence", 0.0) or 0.0),
                    })
                except Exception as e:
                    sys.stderr.write(f"pymupdf table error p{page_num}: {e}\n")
    except Exception as e:
        sys.stderr.write(f"pymupdf failed: {e}\n")
    return tables

def extract_camelot(pdf_path: str, flavor: str = "auto") -> list:
    tables = []
    try:
        import camelot
        if flavor == "auto":
            # Try lattice first, then stream
            for f in ["lattice", "stream"]:
                try:
                    tabs = camelot.read_pdf(pdf_path, pages="all", flavor=f)
                    for t in tabs:
                        tables.append({
                            "page": t.page,
                            "bbox": None,
                            "headers": list(t.df.columns.astype(str)) if hasattr(t, "df") else [],
                            "rows": t.df.values.tolist() if hasattr(t, "df") else [],
                            "method": f"camelot_{f}",
                            "confidence": float(t.parsing_report.get("accuracy", 0.0)),
                        })
                    if tables:
                        break
                except Exception as e:
                    sys.stderr.write(f"camelot {f} failed: {e}\n")
    except Exception as e:
        sys.stderr.write(f"camelot import failed: {e}\n")
    return tables

def deduplicate(tables: list) -> list:
    # Simple dedup by page + row count
    seen = set()
    out = []
    for t in tables:
        key = (t["page"], len(t["rows"]), tuple(t["headers"]))
        if key in seen:
            continue
        seen.add(key)
        out.append(t)
    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf", help="PDF path")
    ap.add_argument("--output", "-o", help="Output dir for CSV/Parquet", default=None)
    ap.add_argument("--method", choices=["auto", "pymupdf", "camelot"], default="auto")
    args = ap.parse_args()

    pdf_path = args.pdf
    all_tables = []

    if args.method in ("auto", "pymupdf"):
        all_tables.extend(extract_pymupdf(pdf_path))
    if args.method in ("auto", "camelot"):
        if not all_tables:
            all_tables.extend(extract_camelot(pdf_path))

    all_tables = deduplicate(all_tables)

    # Optional: write CSV files
    if args.output:
        out_dir = Path(args.output)
        out_dir.mkdir(parents=True, exist_ok=True)
        try:
            import pandas as pd
            for i, t in enumerate(all_tables):
                df = pd.DataFrame(t["rows"], columns=t["headers"] if t["headers"] else None)
                df.to_csv(out_dir / f"table_{i+1}_p{t['page']}.csv", index=False)
        except Exception as e:
            sys.stderr.write(f"csv write failed: {e}\n")

    print(json.dumps({"tables": all_tables, "count": len(all_tables)}))

if __name__ == "__main__":
    main()
