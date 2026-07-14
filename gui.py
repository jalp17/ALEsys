#!/usr/bin/env python3
import json
import logging
import threading
import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
from pathlib import Path

from db_manager import DatabaseManager
from pipeline import Pipeline

BASE_DIR = Path(__file__).resolve().parent
CONFIG_FILE = BASE_DIR / "alesys_gui_config.json"

COLORS = {
    "bg": "#1e1e2e",
    "bg_panel": "#252536",
    "bg_input": "#2d2d44",
    "fg": "#cdd6f4",
    "fg_dim": "#6c7086",
    "fg_accent": "#89b4fa",
    "fg_green": "#a6e3a1",
    "fg_yellow": "#f9e2af",
    "fg_red": "#f38ba8",
    "border": "#45475a",
    "select_bg": "#313244",
    "btn_bg": "#363654",
    "btn_active": "#45457a",
}


class TextHandler(logging.Handler):
    def __init__(self, text_widget):
        super().__init__()
        self.text_widget = text_widget

    def emit(self, record):
        msg = self.format(record)
        def _append():
            self.text_widget.config(state=tk.NORMAL)
            self.text_widget.insert(tk.END, msg + "\n")
            self.text_widget.see(tk.END)
            self.text_widget.config(state=tk.DISABLED)
        try:
            self.text_widget.after(0, _append)
        except Exception:
            pass


class ALEsysGUI:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("ALEsys — GraphRAG-PG")
        self.root.geometry("1100x720")
        self.root.configure(bg=COLORS["bg"])
        self._is_busy = False
        self._load_gui_config()
        self._setup_style()
        self._build_ui()
        self._setup_logging()
        self.root.after(200, self._refresh_docs)

    def _setup_style(self):
        style = ttk.Style()
        style.theme_use("clam")
        for w in (".", "TFrame", "TLabel", "TButton", "TEntry", "TCheckbutton"):
            style.configure(w, background=COLORS["bg"], foreground=COLORS["fg"])
        style.configure("Panel.TFrame", background=COLORS["bg_panel"])
        style.configure("Header.TLabel", background=COLORS["bg_panel"],
                        foreground=COLORS["fg_accent"], font=("monospace", 10, "bold"))
        style.configure("TButton", background=COLORS["btn_bg"], padding=(8, 4))
        style.map("TButton", background=[("active", COLORS["btn_active"])])
        style.configure("TEntry", fieldbackground=COLORS["bg_input"])
        style.configure("Treeview", background=COLORS["bg_input"],
                        foreground=COLORS["fg"], fieldbackground=COLORS["bg_input"])
        style.map("Treeview", background=[("selected", COLORS["select_bg"])])

    def _build_ui(self):
        header = ttk.Frame(self.root)
        header.pack(fill=tk.X, padx=8, pady=(8, 0))
        ttk.Label(header, text="ALEsys", font=("monospace", 14, "bold"),
                  foreground=COLORS["fg_accent"]).pack(side=tk.LEFT)
        ttk.Label(header, text="GraphRAG-PG · PostgreSQL + pgvector",
                  foreground=COLORS["fg_dim"]).pack(side=tk.LEFT, padx=(10, 0))

        main_pane = ttk.PanedWindow(self.root, orient=tk.HORIZONTAL)
        main_pane.pack(fill=tk.BOTH, expand=True, padx=8, pady=8)

        sidebar = ttk.Frame(main_pane, style="Panel.TFrame")
        main_pane.add(sidebar, weight=1)
        self._build_actions_panel(sidebar)
        self._build_docs_panel(sidebar)

        right_pane = ttk.PanedWindow(main_pane, orient=tk.VERTICAL)
        main_pane.add(right_pane, weight=3)
        self._build_search_panel(right_pane)
        self._build_logs_panel(right_pane)

    def _build_actions_panel(self, parent):
        frame = ttk.LabelFrame(parent, text=" Acciones ", padding=6)
        frame.pack(fill=tk.X, padx=6, pady=(6, 3))

        ttk.Button(frame, text="Ejecutar Pipeline",
                   command=self._run_pipeline).pack(fill=tk.X, pady=2)
        ttk.Button(frame, text="Inicializar BD",
                   command=self._init_db).pack(fill=tk.X, pady=2)
        ttk.Button(frame, text="Eliminar Tablas",
                   command=self._drop_db).pack(fill=tk.X, pady=2)

        ttk.Label(frame, text="Ruta libros:", style="Header.TLabel").pack(anchor=tk.W, pady=(4, 0))
        self.books_var = tk.StringVar(value=self._saved_books_dir)
        ttk.Entry(frame, textvariable=self.books_var).pack(fill=tk.X, pady=2)

        self.progress = ttk.Progressbar(frame, mode="indeterminate", length=200)
        self.progress.pack(fill=tk.X, pady=4)

    def _build_docs_panel(self, parent):
        frame = ttk.LabelFrame(parent, text=" Documentos ", padding=6)
        frame.pack(fill=tk.BOTH, expand=True, padx=6, pady=3)

        self.docs_tree = ttk.Treeview(
            frame, columns=("chunks", "entities"), show="tree headings", height=8
        )
        self.docs_tree.heading("#0", text="Archivo", anchor=tk.W)
        self.docs_tree.heading("chunks", text="Chunks", anchor=tk.E)
        self.docs_tree.heading("entities", text="Entidades", anchor=tk.E)
        self.docs_tree.column("#0", width=180)
        self.docs_tree.column("chunks", width=60, anchor=tk.E)
        self.docs_tree.column("entities", width=70, anchor=tk.E)
        self.docs_tree.pack(fill=tk.BOTH, expand=True)

        ttk.Button(frame, text="Refrescar",
                   command=self._refresh_docs).pack(anchor=tk.E, pady=(3, 0))

    def _build_search_panel(self, parent):
        frame = ttk.Frame(parent, style="Panel.TFrame")
        parent.add(frame, weight=3)

        top = ttk.Frame(frame, style="Panel.TFrame")
        top.pack(fill=tk.X, padx=8, pady=(8, 4))
        ttk.Label(top, text="Búsqueda híbrida", style="Header.TLabel").pack(side=tk.LEFT)

        self.search_input = ttk.Entry(frame, font=("monospace", 11))
        self.search_input.pack(fill=tk.X, padx=8, pady=(0, 4))
        self.search_input.bind("<Return>", lambda e: self._do_search())

        btn_row = ttk.Frame(frame, style="Panel.TFrame")
        btn_row.pack(fill=tk.X, padx=8, pady=(0, 4))
        self.search_mode = tk.StringVar(value="hybrid")
        ttk.Radiobutton(btn_row, text="Vectorial", variable=self.search_mode,
                        value="vector").pack(side=tk.LEFT, padx=(0, 8))
        ttk.Radiobutton(btn_row, text="Grafo", variable=self.search_mode,
                        value="graph").pack(side=tk.LEFT, padx=(0, 8))
        ttk.Radiobutton(btn_row, text="Híbrida", variable=self.search_mode,
                        value="hybrid").pack(side=tk.LEFT, padx=(0, 8))
        ttk.Label(btn_row, text="Top K:").pack(side=tk.LEFT, padx=(20, 2))
        self.top_k_var = tk.StringVar(value="5")
        ttk.Entry(btn_row, textvariable=self.top_k_var, width=4).pack(side=tk.LEFT)
        ttk.Button(btn_row, text="Buscar",
                   command=self._do_search).pack(side=tk.RIGHT)
        ttk.Button(btn_row, text="Preguntar (RAG)",
                   command=self._do_ask).pack(side=tk.RIGHT, padx=(0, 4))

        self.results_text = scrolledtext.ScrolledText(
            frame, wrap=tk.WORD, font=("monospace", 10),
            bg=COLORS["bg_input"], fg=COLORS["fg"],
            insertbackground=COLORS["fg"], state=tk.DISABLED,
        )
        self.results_text.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 8))

    def _build_logs_panel(self, parent):
        frame = ttk.Frame(parent, style="Panel.TFrame")
        parent.add(frame, weight=1)

        top = ttk.Frame(frame, style="Panel.TFrame")
        top.pack(fill=tk.X, padx=8, pady=(6, 2))
        ttk.Label(top, text="Logs", style="Header.TLabel").pack(side=tk.LEFT)
        ttk.Button(top, text="Limpiar",
                   command=lambda: self._clear_logs()).pack(side=tk.RIGHT)

        self.log_text = scrolledtext.ScrolledText(
            frame, wrap=tk.WORD, font=("monospace", 9),
            bg=COLORS["bg"], fg=COLORS["fg_dim"],
            height=6, state=tk.DISABLED,
        )
        self.log_text.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 6))

    def _setup_logging(self):
        handler = TextHandler(self.log_text)
        handler.setFormatter(logging.Formatter("%(asctime)s │ %(message)s", datefmt="%H:%M:%S"))
        root = logging.getLogger()
        root.setLevel(logging.INFO)
        for h in root.handlers[:]:
            root.removeHandler(h)
        root.addHandler(handler)
        for name in ("sentence_transformers", "httpx"):
            logging.getLogger(name).setLevel(logging.WARNING)

    def _load_gui_config(self):
        self._saved_books_dir = ""
        if CONFIG_FILE.exists():
            try:
                with open(CONFIG_FILE) as f:
                    cfg = json.load(f)
                self._saved_books_dir = cfg.get("books_dir", "")
            except Exception as e:
                logger = logging.getLogger("ALEsys.GUI")
                logger.warning("Error al cargar configuración: %s", e)

    def _save_gui_config(self):
        try:
            with open(CONFIG_FILE, "w") as f:
                json.dump({"books_dir": self.books_var.get()}, f, indent=2)
        except Exception:
            pass

    def _run_pipeline(self):
        self._run_in_thread(self._do_pipeline)

    def _do_pipeline(self):
        self._set_busy(True, "Ejecutando pipeline...")
        try:
            Pipeline(books_dir=self.books_var.get() or None).run()
        except Exception as e:
            logging.getLogger("ALEsys.GUI").error("Pipeline falló: %s", e)
        finally:
            self._set_busy(False)
            self.root.after(0, self._refresh_docs)

    def _init_db(self):
        self._run_in_thread(self._do_init_db)

    def _do_init_db(self):
        self._set_busy(True, "Inicializando BD...")
        try:
            DatabaseManager().initialize_tables()
            logging.getLogger("ALEsys.GUI").info("✓ BD inicializada")
        except Exception as e:
            logging.getLogger("ALEsys.GUI").error("Error: %s", e)
        finally:
            self._set_busy(False)

    def _drop_db(self):
        if not messagebox.askyesno("Confirmar", "¿Eliminar todas las tablas?"):
            return
        self._run_in_thread(self._do_drop_db)

    def _do_drop_db(self):
        self._set_busy(True, "Eliminando tablas...")
        try:
            DatabaseManager().drop_tables()
            logging.getLogger("ALEsys.GUI").info("✓ Tablas eliminadas")
        except Exception as e:
            logging.getLogger("ALEsys.GUI").error("Error: %s", e)
        finally:
            self._set_busy(False)
            self.root.after(0, self._refresh_docs)

    def _do_search(self):
        self._run_in_thread(self._search)

    def _search(self):
        query = self.search_input.get().strip()
        if not query:
            return
        self._set_busy(True, "Buscando...")

        try:
            top_k = int(self.top_k_var.get())
        except ValueError:
            top_k = 5

        mode = self.search_mode.get()
        try:
            if mode == "graph":
                from test_queries import graph_search
                result = graph_search(query)
                lines = [f"=== Entidades ({len(result['entities'])}) ==="]
                for e in result["entities"]:
                    lines.append(f"  • {e['name']} ({e['type']})")
                lines.append(f"\n=== Relaciones ({len(result['relations'])}) ===")
                for r in result["relations"]:
                    lines.append(f"  {r['source']} --[{r['relation']}]--> {r['target']}")
                text = "\n".join(lines)
            elif mode == "hybrid":
                from test_queries import hybrid_search
                results = hybrid_search(query, top_k)
                lines = []
                for r in results:
                    entities = ", ".join(e["name"] for e in r.get("entities", []))
                    lines.append(f"[{r['filename']}] sim={r['similarity']}")
                    lines.append(f"  {r['content'][:300]}...")
                    if entities:
                        lines.append(f"  Entidades: {entities}")
                text = "\n\n".join(lines) if lines else "Sin resultados"
            else:
                from test_queries import vector_search
                results = vector_search(query, top_k)
                lines = [f"[{r['filename']}] sim={r['similarity']}\n  {r['content'][:300]}..." for r in results]
                text = "\n\n".join(lines) if lines else "Sin resultados"
        except Exception as e:
            text = f"Error: {e}"

        def update():
            self.results_text.config(state=tk.NORMAL)
            self.results_text.delete("1.0", tk.END)
            self.results_text.insert(tk.END, text)
            self.results_text.config(state=tk.DISABLED)
        self.root.after(0, update)
        self._set_busy(False)

    def _do_ask(self):
        self._run_in_thread(self._ask)

    def _ask(self):
        question = self.search_input.get().strip()
        if not question:
            return
        self._set_busy(True, "Preguntando...")
        try:
            top_k = int(self.top_k_var.get())
        except ValueError:
            top_k = 5
        try:
            from test_queries import ask
            text = ask(question, top_k)
        except Exception as e:
            text = f"Error: {e}"

        def update():
            self.results_text.config(state=tk.NORMAL)
            self.results_text.delete("1.0", tk.END)
            self.results_text.insert(tk.END, text)
            self.results_text.config(state=tk.DISABLED)
        self.root.after(0, update)
        self._set_busy(False)

    def _refresh_docs(self):
        self._run_in_thread(self._do_refresh_docs)

    def _do_refresh_docs(self):
        tree = self.docs_tree
        tree.delete(*tree.get_children())
        try:
            db = DatabaseManager()
            db.initialize_tables()
            cur = db.cursor
            cur.execute(
                """
                SELECT d.id, d.filename, COUNT(f.id) AS chunks,
                       COUNT(DISTINCT e.id) AS entities
                FROM documentos d
                LEFT JOIN fragmentos f ON f.documento_id = d.id
                LEFT JOIN entidades e ON e.fragmento_id = f.id
                GROUP BY d.id ORDER BY d.created_at DESC
                """
            )
            def update():
                for r in cur.fetchall():
                    tree.insert("", tk.END, text=r["filename"],
                                values=(r["chunks"], r["entities"]))
            self.root.after(0, update)
        except Exception:
            pass

    def _clear_logs(self):
        self.log_text.config(state=tk.NORMAL)
        self.log_text.delete("1.0", tk.END)
        self.log_text.config(state=tk.DISABLED)

    def _run_in_thread(self, target):
        t = threading.Thread(target=target, daemon=True)
        t.start()

    def _set_busy(self, busy, status=""):
        self._is_busy = busy
        def update():
            if busy:
                self.progress.start(10)
            else:
                self.progress.stop()
        self.root.after(0, update)

    def _on_close(self):
        self._save_gui_config()
        self.root.destroy()

    def run(self):
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
        self.root.mainloop()


def main():
    app = ALEsysGUI()
    app.run()


if __name__ == "__main__":
    main()
