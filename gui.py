#!/usr/bin/env python3
"""
gui.py — Interfaz gráfica ligera de ALEsys.

Usa tkinter (incluido en Python, cero dependencias extra).
Paneles:
  - Modelos: Gestión del directorio de modelos GGUF
  - Proyectos: Lista y gestión de proyectos indexados
  - Chat: Entrada de preguntas y visualización de respuestas
  - Logs/Progreso: Monitoreo en tiempo real

Uso:
    python gui.py
"""

import json
import logging
import os
import sys
import threading
import time
import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
from pathlib import Path
from io import StringIO

# ── Rutas del sistema ──────────────────────────────────────────────
BASE_DIR = Path(__file__).resolve().parent
PROJECTS_DIR = BASE_DIR / "projects"
DEFAULT_MODELS_DIR = Path.home() / "llama.cpp" / "build-vulkan" / "bin" / "models"
CONFIG_FILE = BASE_DIR / "alesys_gui_config.json"


# ── Colores (tema oscuro compacto) ─────────────────────────────────
COLORS = {
    "bg":          "#1e1e2e",
    "bg_panel":    "#252536",
    "bg_input":    "#2d2d44",
    "fg":          "#cdd6f4",
    "fg_dim":      "#6c7086",
    "fg_accent":   "#89b4fa",
    "fg_green":    "#a6e3a1",
    "fg_yellow":   "#f9e2af",
    "fg_red":      "#f38ba8",
    "fg_purple":   "#cba6f7",
    "border":      "#45475a",
    "select_bg":   "#313244",
    "btn_bg":      "#363654",
    "btn_active":  "#45457a",
}


class TextHandler(logging.Handler):
    """Logging handler que redirige logs a un widget Text de tkinter."""

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
    """Interfaz gráfica principal de ALEsys."""

    def __init__(self):
        self.root = tk.Tk()
        self.root.title("ALEsys — Asistente de Desarrollo Multi-Proyecto")
        self.root.geometry("1100x720")
        self.root.minsize(800, 500)
        self.root.configure(bg=COLORS["bg"])

        # Estado
        self.models_dir = tk.StringVar(value=str(DEFAULT_MODELS_DIR))
        self.selected_project = tk.StringVar(value="")
        self.web_search_enabled = tk.BooleanVar(value=True)
        self.skip_summaries = tk.BooleanVar(value=False)
        self.parallel_load = tk.BooleanVar(value=True)
        self.context_size = tk.IntVar(value=4096)
        self._chat_agent = None
        self._is_busy = False

        # Cargar config persistente
        self._load_gui_config()

        # Estilo ttk
        self._setup_style()

        # Construir UI
        self._build_ui()

        # Setup logging al panel de logs
        self._setup_logging()

        # Cargar datos iniciales
        self.root.after(100, self._refresh_models)
        self.root.after(200, self._refresh_projects)

    # ── Estilo ─────────────────────────────────────────────────────
    def _setup_style(self):
        style = ttk.Style()
        style.theme_use("clam")

        style.configure(".", 
            background=COLORS["bg"], 
            foreground=COLORS["fg"],
            fieldbackground=COLORS["bg_input"],
            borderwidth=0,
        )
        style.configure("TFrame", background=COLORS["bg"])
        style.configure("Panel.TFrame", background=COLORS["bg_panel"])
        style.configure("TLabel", background=COLORS["bg"], foreground=COLORS["fg"])
        style.configure("Panel.TLabel", background=COLORS["bg_panel"], foreground=COLORS["fg"])
        style.configure("Header.TLabel", 
            background=COLORS["bg_panel"], 
            foreground=COLORS["fg_accent"],
            font=("monospace", 10, "bold"),
        )
        style.configure("TButton",
            background=COLORS["btn_bg"],
            foreground=COLORS["fg"],
            padding=(8, 4),
        )
        style.map("TButton",
            background=[("active", COLORS["btn_active"])],
        )
        style.configure("Accent.TButton",
            background=COLORS["fg_accent"],
            foreground=COLORS["bg"],
        )
        style.configure("TEntry",
            fieldbackground=COLORS["bg_input"],
            foreground=COLORS["fg"],
            insertcolor=COLORS["fg"],
        )
        style.configure("TCheckbutton",
            background=COLORS["bg_panel"],
            foreground=COLORS["fg"],
        )
        style.configure("Treeview",
            background=COLORS["bg_input"],
            foreground=COLORS["fg"],
            fieldbackground=COLORS["bg_input"],
            rowheight=22,
        )
        style.configure("Treeview.Heading",
            background=COLORS["bg_panel"],
            foreground=COLORS["fg_accent"],
        )
        style.map("Treeview",
            background=[("selected", COLORS["select_bg"])],
        )
        style.configure("TProgressbar",
            background=COLORS["fg_accent"],
            troughcolor=COLORS["bg_input"],
        )

    # ── Build UI ───────────────────────────────────────────────────
    def _build_ui(self):
        # Header
        header = ttk.Frame(self.root)
        header.pack(fill=tk.X, padx=8, pady=(8, 0))
        ttk.Label(header, text="⚡ ALEsys", font=("monospace", 14, "bold"),
                  foreground=COLORS["fg_accent"]).pack(side=tk.LEFT)
        ttk.Label(header, text="Asistente IA Local · RAG Multi-Proyecto",
                  foreground=COLORS["fg_dim"]).pack(side=tk.LEFT, padx=(10, 0))

        # PanedWindow principal (izq: sidebar, der: chat+logs)
        main_pane = ttk.PanedWindow(self.root, orient=tk.HORIZONTAL)
        main_pane.pack(fill=tk.BOTH, expand=True, padx=8, pady=8)

        # ─ Sidebar izquierda ──────────────────────────────────
        sidebar = ttk.Frame(main_pane, style="Panel.TFrame")
        main_pane.add(sidebar, weight=1)

        self._build_models_panel(sidebar)
        self._build_projects_panel(sidebar)
        self._build_options_panel(sidebar)

        # ─ Panel derecho (chat + logs) ────────────────────────
        right_pane = ttk.PanedWindow(main_pane, orient=tk.VERTICAL)
        main_pane.add(right_pane, weight=3)

        self._build_chat_panel(right_pane)
        self._build_logs_panel(right_pane)

    def _build_models_panel(self, parent):
        """Panel de gestión de modelos GGUF."""
        frame = ttk.LabelFrame(parent, text=" 📁 Directorio de Modelos ", padding=6)
        frame.pack(fill=tk.X, padx=6, pady=(6, 3))

        # Ruta del directorio
        row = ttk.Frame(frame, style="Panel.TFrame")
        row.pack(fill=tk.X)
        self.models_entry = ttk.Entry(row, textvariable=self.models_dir, width=30)
        self.models_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 4))
        ttk.Button(row, text="…", width=3, 
                   command=self._browse_models_dir).pack(side=tk.RIGHT)

        # Lista de modelos
        self.models_tree = ttk.Treeview(
            frame, columns=("size",), show="tree headings", height=4
        )
        self.models_tree.bind("<<TreeviewSelect>>", self._on_model_select)
        self.models_tree.bind("<<TreeviewSelect>>", self._on_model_select)
        self.models_tree.heading("#0", text="Modelo", anchor=tk.W)
        self.models_tree.heading("size", text="Tamaño", anchor=tk.E)
        self.models_tree.column("#0", width=180)
        self.models_tree.column("size", width=70, anchor=tk.E)
        self.models_tree.pack(fill=tk.X, pady=(4, 0))

        ttk.Button(frame, text="🔄 Refrescar", 
                   command=self._refresh_models).pack(anchor=tk.E, pady=(3, 0))
        # área de texto para información del modelo seleccionado
        self.model_info_display = scrolledtext.ScrolledText(
            frame,
            wrap=tk.WORD,
            height=5,
            font=("monospace", 9),
            bg=COLORS["bg_input"],
            fg=COLORS["fg_dim"],
            state=tk.DISABLED,
        )
        self.model_info_display.pack(fill=tk.X, pady=(4,0))

    def _build_projects_panel(self, parent):
        """Panel de gestión de proyectos."""
        frame = ttk.LabelFrame(parent, text=" 📂 Proyectos ", padding=6)
        frame.pack(fill=tk.BOTH, expand=True, padx=6, pady=3)

        # Lista de proyectos
        self.projects_tree = ttk.Treeview(
            frame, columns=("lang", "status", "vectors"), 
            show="tree headings", height=5,
        )
        self.projects_tree.heading("#0", text="Proyecto", anchor=tk.W)
        self.projects_tree.heading("lang", text="Lang", anchor=tk.W)
        self.projects_tree.heading("status", text="Estado", anchor=tk.W)
        self.projects_tree.heading("vectors", text="Vecs", anchor=tk.E)
        self.projects_tree.column("#0", width=100)
        self.projects_tree.column("lang", width=50)
        self.projects_tree.column("status", width=70)
        self.projects_tree.column("vectors", width=45, anchor=tk.E)
        self.projects_tree.pack(fill=tk.BOTH, expand=True)
        self.projects_tree.bind("<<TreeviewSelect>>", self._on_project_select)

        # Botones de acción
        btn_row = ttk.Frame(frame, style="Panel.TFrame")
        btn_row.pack(fill=tk.X, pady=(4, 0))
        ttk.Button(btn_row, text="➕ Nuevo", 
                   command=self._new_project).pack(side=tk.LEFT, padx=(0, 3))
        ttk.Button(btn_row, text="📊 Indexar", 
                   command=self._index_project).pack(side=tk.LEFT, padx=(0, 3))
        ttk.Button(btn_row, text="🔄", width=3,
                   command=self._refresh_projects).pack(side=tk.RIGHT)

    def _build_options_panel(self, parent):
        """Panel de opciones."""
        frame = ttk.LabelFrame(parent, text=" ⚙ Opciones ", padding=6)
        frame.pack(fill=tk.X, padx=6, pady=(3, 6))

        ttk.Checkbutton(frame, text="Búsqueda web (DuckDuckGo)",
                        variable=self.web_search_enabled,
                        style="TCheckbutton").pack(anchor=tk.W)
        ttk.Checkbutton(frame, text="Omitir resúmenes LLM al indexar",
                        variable=self.skip_summaries,
                        style="TCheckbutton").pack(anchor=tk.W)
        ttk.Checkbutton(frame, text="Cargar modelos en paralelo",
                        variable=self.parallel_load,
                        style="TCheckbutton").pack(anchor=tk.W)
        # contexto
        ctx_row = ttk.Frame(frame, style="Panel.TFrame")
        ctx_row.pack(fill=tk.X, pady=(2,0))
        ttk.Label(ctx_row, text="Contexto (tokens):", style="Panel.TLabel").pack(side=tk.LEFT)
        ttk.Entry(ctx_row, textvariable=self.context_size, width=6).pack(side=tk.LEFT, padx=(4,0))

    def _build_chat_panel(self, parent):
        """Panel principal de chat."""
        chat_frame = ttk.Frame(parent, style="Panel.TFrame")
        parent.add(chat_frame, weight=3)

        # Indicador de proyecto + estado
        top_bar = ttk.Frame(chat_frame, style="Panel.TFrame")
        top_bar.pack(fill=tk.X, padx=8, pady=(8, 4))
        ttk.Label(top_bar, text="💬 Chat", style="Header.TLabel").pack(side=tk.LEFT)
        self.project_label = ttk.Label(
            top_bar, text="(ningún proyecto seleccionado)", 
            foreground=COLORS["fg_dim"], background=COLORS["bg_panel"],
        )
        self.project_label.pack(side=tk.LEFT, padx=(8, 0))
        self.status_label = ttk.Label(
            top_bar, text="", foreground=COLORS["fg_green"],
            background=COLORS["bg_panel"],
        )
        self.status_label.pack(side=tk.RIGHT)

        # Área de respuestas (scrollable)
        self.chat_display = scrolledtext.ScrolledText(
            chat_frame,
            wrap=tk.WORD,
            font=("monospace", 10),
            bg=COLORS["bg_input"],
            fg=COLORS["fg"],
            insertbackground=COLORS["fg"],
            selectbackground=COLORS["select_bg"],
            relief=tk.FLAT,
            padx=10,
            pady=8,
            state=tk.DISABLED,
        )
        self.chat_display.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 4))

        # Tags para formateo
        self.chat_display.tag_configure("user", foreground=COLORS["fg_green"], 
                                        font=("monospace", 10, "bold"))
        self.chat_display.tag_configure("assistant", foreground=COLORS["fg_accent"])
        self.chat_display.tag_configure("system", foreground=COLORS["fg_dim"],
                                        font=("monospace", 9, "italic"))
        self.chat_display.tag_configure("error", foreground=COLORS["fg_red"])

        # Barra de entrada
        input_frame = ttk.Frame(chat_frame, style="Panel.TFrame")
        input_frame.pack(fill=tk.X, padx=8, pady=(0, 8))

        self.chat_input = ttk.Entry(input_frame, font=("monospace", 11))
        self.chat_input.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 4))
        self.chat_input.bind("<Return>", lambda e: self._send_question())

        self.send_btn = ttk.Button(input_frame, text="Enviar ➤",
                                   command=self._send_question)
        self.send_btn.pack(side=tk.RIGHT)

        # Progress bar
        self.progress = ttk.Progressbar(chat_frame, mode="indeterminate", length=200)
        self.progress.pack(fill=tk.X, padx=8, pady=(0, 4))

    def _build_logs_panel(self, parent):
        """Panel de logs/progreso."""
        logs_frame = ttk.Frame(parent, style="Panel.TFrame")
        parent.add(logs_frame, weight=1)

        top = ttk.Frame(logs_frame, style="Panel.TFrame")
        top.pack(fill=tk.X, padx=8, pady=(6, 2))
        ttk.Label(top, text="📋 Logs", style="Header.TLabel").pack(side=tk.LEFT)
        ttk.Button(top, text="Limpiar", 
                   command=self._clear_logs).pack(side=tk.RIGHT)

        self.log_display = scrolledtext.ScrolledText(
            logs_frame,
            wrap=tk.WORD,
            font=("monospace", 9),
            bg=COLORS["bg"],
            fg=COLORS["fg_dim"],
            insertbackground=COLORS["fg"],
            relief=tk.FLAT,
            height=6,
            padx=8,
            pady=4,
            state=tk.DISABLED,
        )
        self.log_display.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 6))

    # ── Logging ────────────────────────────────────────────────────
    def _setup_logging(self):
        handler = TextHandler(self.log_display)
        handler.setFormatter(logging.Formatter(
            "%(asctime)s │ %(name)-20s │ %(message)s", datefmt="%H:%M:%S"
        ))
        root_logger = logging.getLogger("ALEsys")
        root_logger.setLevel(logging.INFO)
        root_logger.addHandler(handler)

        # También capturar logs de sentence_transformers
        for name in ("sentence_transformers", "transformers"):
            logging.getLogger(name).setLevel(logging.WARNING)

    # ── Config persistente ─────────────────────────────────────────
    def _load_gui_config(self):
        if CONFIG_FILE.exists():
            try:
                with open(CONFIG_FILE) as f:
                    cfg = json.load(f)
                if "models_dir" in cfg:
                    self.models_dir.set(cfg["models_dir"])
                if "web_search" in cfg:
                    self.web_search_enabled.set(cfg["web_search"])
                if "skip_summaries" in cfg:
                    self.skip_summaries.set(cfg["skip_summaries"])
                if "parallel_load" in cfg:
                    self.parallel_load.set(cfg["parallel_load"])
                if "context_size" in cfg:
                    self.context_size.set(cfg["context_size"])
            except Exception:
                pass

    def _save_gui_config(self):
        cfg = {
            "models_dir": self.models_dir.get(),
            "web_search": self.web_search_enabled.get(),
            "skip_summaries": self.skip_summaries.get(),
            "parallel_load": self.parallel_load.get(),
            "context_size": self.context_size.get(),
        }
        try:
            with open(CONFIG_FILE, "w") as f:
                json.dump(cfg, f, indent=2)
        except Exception:
            pass

    # ── Acciones: Modelos ──────────────────────────────────────────
    def _browse_models_dir(self):
        d = filedialog.askdirectory(
            title="Seleccionar directorio de modelos GGUF",
            initialdir=self.models_dir.get(),
        )
        if d:
            self.models_dir.set(d)
            self._save_gui_config()
            self._refresh_models()

    def _refresh_models(self):
        tree = self.models_tree
        tree.delete(*tree.get_children())

        models_path = Path(self.models_dir.get())
        if not models_path.exists():
            tree.insert("", tk.END, text="(directorio no encontrado)", values=("",))
            return

        gguf_files = sorted(models_path.glob("*.gguf"))
        if not gguf_files:
            tree.insert("", tk.END, text="(sin modelos .gguf)", values=("",))
            return

        for f in gguf_files:
            size_mb = f.stat().st_size / (1024 ** 2)
            if size_mb >= 1024:
                size_str = f"{size_mb/1024:.1f} GB"
            else:
                size_str = f"{size_mb:.0f} MB"
            tree.insert("", tk.END, text=f.name, values=(size_str,))
        # clear info display when refreshing list
        if hasattr(self, 'model_info_display'):
            self.model_info_display.config(state=tk.NORMAL)
            self.model_info_display.delete('1.0', tk.END)
            self.model_info_display.config(state=tk.DISABLED)

    # ── Acciones: Proyectos ────────────────────────────────────────
    def _refresh_projects(self):
        tree = self.projects_tree
        tree.delete(*tree.get_children())

        PROJECTS_DIR.mkdir(parents=True, exist_ok=True)
        projects = sorted(d for d in PROJECTS_DIR.iterdir() if d.is_dir())

        if not projects:
            tree.insert("", tk.END, text="(sin proyectos)", values=("", "", ""))
            return

        for proj in projects:
            config_path = proj / "config.json"
            info_path = proj / "vector_db" / "index_info.json"

            lang = "?"
            status = "No indexado"
            vectors = "-"

            if config_path.exists():
                try:
                    with open(config_path) as f:
                        cfg = json.load(f)
                    lang = cfg.get("language", "?")[:6]
                except Exception:
                    pass

            if info_path.exists():
                try:
                    with open(info_path) as f:
                        info = json.load(f)
                    vectors = str(info.get("num_vectors", "?"))
                    status = "✓ Listo"
                except Exception:
                    status = "✓"

            tree.insert("", tk.END, text=proj.name, 
                       values=(lang, status, vectors))

    def _on_project_select(self, event):
        sel = self.projects_tree.selection()
        if sel:
            name = self.projects_tree.item(sel[0], "text")
            if name and not name.startswith("("):
                self.selected_project.set(name)
                self.project_label.config(text=f"Proyecto: {name}")
                self._chat_agent = None  # Reset agent for new project

    def _on_model_select(self, event):
        sel = self.models_tree.selection()
        if not sel:
            return
        name = self.models_tree.item(sel[0], "text")
        if not name:
            return
        model_path = Path(self.models_dir.get()) / name
        self._run_in_thread(self._populate_model_info, str(model_path))

    def _populate_model_info(self, path):
        """Carga metadata del modelo y actualiza el widget de info."""
        logger = logging.getLogger("ALEsys.GUI")
        try:
            from core.memory_manager import MemoryManager
            mm = MemoryManager()
            info = mm.probe_model(path)
        except Exception as e:
            logger.error(f"Error al sondear modelo: {e}")
            info = {"error": str(e)}
        lines = [f"{k}: {v}" for k, v in info.items()]
        text = "\n".join(lines)
        def update():
            self.model_info_display.config(state=tk.NORMAL)
            self.model_info_display.delete("1.0", tk.END)
            self.model_info_display.insert(tk.END, text)
            self.model_info_display.config(state=tk.DISABLED)
        self.root.after(0, update)

    def _new_project(self):
        """Diálogo para crear un nuevo proyecto."""
        dialog = tk.Toplevel(self.root)
        dialog.title("Nuevo Proyecto")
        dialog.geometry("450x200")
        dialog.configure(bg=COLORS["bg_panel"])
        dialog.transient(self.root)
        dialog.grab_set()

        ttk.Label(dialog, text="Nombre del proyecto:", 
                  style="Panel.TLabel").pack(padx=12, pady=(12, 2), anchor=tk.W)
        name_var = tk.StringVar()
        ttk.Entry(dialog, textvariable=name_var, width=40).pack(padx=12, fill=tk.X)

        ttk.Label(dialog, text="Ruta del código fuente:",
                  style="Panel.TLabel").pack(padx=12, pady=(8, 2), anchor=tk.W)
        
        path_frame = ttk.Frame(dialog, style="Panel.TFrame")
        path_frame.pack(padx=12, fill=tk.X)
        path_var = tk.StringVar()
        ttk.Entry(path_frame, textvariable=path_var).pack(side=tk.LEFT, fill=tk.X, expand=True)
        def _browse():
            d = filedialog.askdirectory(title="Seleccionar código fuente")
            if d:
                path_var.set(d)
        ttk.Button(path_frame, text="…", width=3, command=_browse).pack(side=tk.RIGHT, padx=(4, 0))

        def _create():
            name = name_var.get().strip()
            path = path_var.get().strip()
            if not name or not path:
                messagebox.showwarning("Campos vacíos", "Completa nombre y ruta.")
                return
            if not Path(path).exists():
                messagebox.showerror("Error", f"Ruta no encontrada: {path}")
                return
            self._run_in_thread(self._do_init_project, name, path)
            dialog.destroy()

        ttk.Button(dialog, text="Crear Proyecto", command=_create).pack(pady=12)

    def _do_init_project(self, name, path):
        """Ejecuta init en un hilo separado."""
        self._set_busy(True, "Creando proyecto...")
        try:
            sys.argv = ["main.py", "init", name, path, "--force"]
            # Import and run directly instead of subprocess
            from core.indexer import PROJECTS_DIR as p_dir
            project_dir = p_dir / name
            project_dir.mkdir(parents=True, exist_ok=True)
            (project_dir / "vector_db").mkdir(parents=True, exist_ok=True)

            source_path = Path(path).resolve()
            config = {
                "project_name": name,
                "source_path": str(source_path),
                "language": self._detect_language(source_path),
                "extensions": [
                    ".py", ".js", ".ts", ".jsx", ".tsx", ".java", ".cpp", ".c",
                    ".h", ".hpp", ".cs", ".go", ".rs", ".rb", ".php",
                    ".html", ".css", ".scss", ".vue", ".svelte",
                    ".json", ".yaml", ".yml", ".toml",
                    ".md", ".txt", ".sh", ".bash", ".sql",
                ],
                "exclude_dirs": [
                    "node_modules", ".git", "__pycache__", "venv", ".venv",
                    ".idea", ".vscode", "dist", "build", ".next", "target",
                ],
                "exclude_files": ["*.pyc", "*.lock", "*.log", "*.min.js", "*.min.css"],
                "chunk_size": 1500,
                "chunk_overlap": 200,
                "max_file_size_kb": 500,
            }
            with open(project_dir / "config.json", "w", encoding="utf-8") as f:
                json.dump(config, f, ensure_ascii=False, indent=2)

            logger = logging.getLogger("ALEsys.GUI")
            logger.info(f"✓ Proyecto creado: {name} → {source_path}")

            self.root.after(0, self._refresh_projects)
        except Exception as e:
            self.root.after(0, lambda: self._chat_append(
                f"Error creando proyecto: {e}", "error"))
        finally:
            self._set_busy(False)

    @staticmethod
    def _detect_language(source_path):
        ext_map = {
            ".py": "python", ".js": "javascript", ".ts": "typescript",
            ".java": "java", ".cpp": "c++", ".c": "c", ".go": "go",
            ".rs": "rust", ".rb": "ruby", ".php": "php",
        }
        counts = {}
        try:
            for f in source_path.rglob("*"):
                if f.is_file() and f.suffix.lower() in ext_map:
                    lang = ext_map[f.suffix.lower()]
                    counts[lang] = counts.get(lang, 0) + 1
        except Exception:
            pass
        return max(counts, key=counts.get) if counts else "unknown"

    def _index_project(self):
        """Indexa el proyecto seleccionado en un hilo."""
        name = self.selected_project.get()
        if not name:
            messagebox.showinfo("Info", "Selecciona un proyecto primero.")
            return
        self._run_in_thread(self._do_index, name)

    def _do_index(self, name):
        """Ejecuta indexación en un hilo separado."""
        self._set_busy(True, "Indexando...")
        logger = logging.getLogger("ALEsys.GUI")
        try:
            from core.indexer import ProjectIndexer

            models_dir = self.models_dir.get()
            indexer = ProjectIndexer(
                project_name=name,
                models_dir=models_dir,
                parallel_load=self.parallel_load.get(),
                context_size=self.context_size.get(),
            )
            stats = indexer.run(skip_summaries=self.skip_summaries.get())

            self._chat_append(
                f"✓ Indexación completada: {stats.get('chunks_created', 0)} chunks, "
                f"{stats.get('files_indexed', 0)} archivos, "
                f"{stats.get('total_time_s', 0):.1f}s",
                "system",
            )
            self.root.after(0, self._refresh_projects)
        except Exception as e:
            logger.error(f"Error indexando: {e}")
            self._chat_append(f"Error indexando: {e}", "error")
        finally:
            self._set_busy(False)

    # ── Acciones: Chat ─────────────────────────────────────────────
    def _send_question(self):
        question = self.chat_input.get().strip()
        if not question or self._is_busy:
            return

        name = self.selected_project.get()
        if not name:
            self._chat_append("⚠ Selecciona un proyecto primero.", "error")
            return

        self.chat_input.delete(0, tk.END)
        self._chat_append(f"Tú: {question}", "user")
        self._run_in_thread(self._do_ask, name, question)

    def _do_ask(self, project_name, question):
        """Ejecuta pregunta RAG en un hilo separado."""
        self._set_busy(True, "Pensando...")
        logger = logging.getLogger("ALEsys.GUI")
        try:
            from core.chat_agent import ChatAgent

            # Crear/reutilizar agente
            if (self._chat_agent is None or 
                self._chat_agent.project_name != project_name):
                self._chat_agent = ChatAgent(
                    project_name=project_name,
                    models_dir=self.models_dir.get(),
                    enable_web_search=self.web_search_enabled.get(),
                    parallel_load=self.parallel_load.get(),
                    context_size=self.context_size.get(),
                )

            # Generar respuesta (sin streaming por simplicidad en GUI)
            response = self._do_ask_sync(question)
            self._chat_append(f"ALEsys: {response}", "assistant")

        except FileNotFoundError as e:
            self._chat_append(f"⚠ {e}", "error")
        except Exception as e:
            logger.error(f"Error en chat: {e}")
            self._chat_append(f"Error: {e}", "error")
        finally:
            self._set_busy(False)

    def _do_ask_sync(self, question):
        """Ejecuta ask sin streaming (adecuado para GUI)."""
        agent = self._chat_agent

        # 1. Retrieve
        chunks = agent.retrieve(question)
        context = agent._build_context(chunks)

        # 2. Web search (opcional)
        web_context = ""
        if agent.enable_web_search and agent._should_web_search(question):
            if agent._web_searcher is None:
                from core.web_search import WebSearcher
                agent._web_searcher = WebSearcher()
            results = agent._web_searcher.search(question)
            web_context = agent._web_searcher.format_results_as_context(results)

        # 3. Load model & generate
        model_path = agent._find_conversational_model()
        from core.memory_manager import MemoryManager
        mm = MemoryManager()
        llm = mm.load_model(str(model_path), n_ctx=4096, n_gpu_layers=-1)

        messages = agent._build_messages(question, context, web_context)
        response = agent._generate_response(llm, messages)

        # Save history
        agent._chat_history.append({"role": "user", "content": question})
        agent._chat_history.append({"role": "assistant", "content": response})

        return response

    # ── Chat display ───────────────────────────────────────────────
    def _chat_append(self, text, tag="system"):
        """Añade texto al display de chat (thread-safe)."""
        def _do():
            self.chat_display.config(state=tk.NORMAL)
            self.chat_display.insert(tk.END, text + "\n\n", tag)
            self.chat_display.see(tk.END)
            self.chat_display.config(state=tk.DISABLED)
        self.root.after(0, _do)

    # ── Utilidades ─────────────────────────────────────────────────
    def _run_in_thread(self, target, *args):
        """Ejecuta una función en un hilo daemon."""
        t = threading.Thread(target=target, args=args, daemon=True)
        t.start()

    def _set_busy(self, busy, status=""):
        """Establece estado busy con indicador de progreso."""
        self._is_busy = busy
        def _do():
            if busy:
                self.progress.start(10)
                self.send_btn.config(state=tk.DISABLED)
                self.status_label.config(text=f"⏳ {status}", foreground=COLORS["fg_yellow"])
            else:
                self.progress.stop()
                self.send_btn.config(state=tk.NORMAL)
                self.status_label.config(text="✓ Listo", foreground=COLORS["fg_green"])
        self.root.after(0, _do)

    def _clear_logs(self):
        self.log_display.config(state=tk.NORMAL)
        self.log_display.delete("1.0", tk.END)
        self.log_display.config(state=tk.DISABLED)

    # ── Run ────────────────────────────────────────────────────────
    def run(self):
        """Inicia el loop principal de la GUI."""
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
        self.root.mainloop()

    def _on_close(self):
        self._save_gui_config()
        # Descargar modelo si hay uno cargado
        try:
            from core.memory_manager import MemoryManager
            mm = MemoryManager()
            mm.unload_model()
        except Exception:
            pass
        self.root.destroy()


def main():
    app = ALEsysGUI()
    app.run()


if __name__ == "__main__":
    main()
