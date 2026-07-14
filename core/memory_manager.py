"""
memory_manager.py — Gestión de carga/descarga de modelos GGUF en VRAM.

Singleton que asegura que NUNCA haya más de un modelo LLM cargado
simultáneamente en la GPU. Diseñado para AMD Vega 3 APU (7GB VRAM compartida).

Uso:
    from core.memory_manager import MemoryManager
    mm = MemoryManager()
    llm = mm.load_model("ruta/al/modelo.gguf", n_ctx=2048)
    # ... usar llm ...
    mm.unload_model()
"""

import gc
import os
import time
import logging
from pathlib import Path
from typing import Optional

import psutil

logger = logging.getLogger("IA-Dev-System.MemoryManager")


class MemoryManager:
    """Gestor de memoria para modelos LLM cargados con llama-cpp-python.
    
    Implementa patrón Singleton para garantizar control centralizado
    de la VRAM. Solo permite un modelo cargado a la vez.
    """

    _instance: Optional["MemoryManager"] = None
    _initialized: bool = False

    # Límite seguro de VRAM (dejar margen para el SO y embeddings)
    VRAM_LIMIT_MB: int = 6000  # 6GB de 7GB disponibles

    def __new__(cls) -> "MemoryManager":
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self) -> None:
        if self._initialized:
            return
        self._initialized = True
        self._current_model = None
        self._current_model_path: Optional[str] = None
        self._current_model_size_mb: float = 0.0
        self._load_time: float = 0.0
        logger.info("MemoryManager inicializado (Singleton)")
        self._log_system_memory()

    def _log_system_memory(self) -> None:
        """Registra el estado actual de la memoria del sistema."""
        mem = psutil.virtual_memory()
        logger.info(
            f"RAM del sistema: "
            f"{mem.used / (1024**3):.1f}GB usados / "
            f"{mem.total / (1024**3):.1f}GB total "
            f"({mem.percent}% uso)"
        )
        # Intentar leer VRAM de AMD vía sysfs
        vram_info = self._get_amd_vram_info()
        if vram_info:
            logger.info(f"VRAM (AMD): {vram_info}")

    @staticmethod
    def _get_amd_vram_info() -> Optional[str]:
        """Lee información de VRAM desde sysfs de AMD (si está disponible)."""
        try:
            vram_paths = list(Path("/sys/class/drm").glob("card*/device/mem_info_vram_used"))
            if not vram_paths:
                return None
            
            vram_used_bytes = int(vram_paths[0].read_text().strip())
            vram_total_path = vram_paths[0].parent / "mem_info_vram_total"
            vram_total_bytes = int(vram_total_path.read_text().strip()) if vram_total_path.exists() else 0
            
            used_mb = vram_used_bytes / (1024**2)
            total_mb = vram_total_bytes / (1024**2) if vram_total_bytes else 0
            
            if total_mb > 0:
                return f"{used_mb:.0f}MB / {total_mb:.0f}MB ({used_mb/total_mb*100:.1f}%)"
            return f"{used_mb:.0f}MB usados"
        except (OSError, ValueError):
            return None

    @staticmethod
    def _get_model_size_mb(model_path: str) -> float:
        """Obtiene el tamaño del archivo GGUF en MB."""
        try:
            size_bytes = os.path.getsize(model_path)
            return size_bytes / (1024**2)
        except OSError:
            return 0.0

    def _check_vram_available(self, model_path: str) -> bool:
        """Verifica si hay suficiente VRAM estimada para cargar el modelo."""
        model_size_mb = self._get_model_size_mb(model_path)
        if model_size_mb == 0:
            logger.warning(f"No se pudo determinar el tamaño de: {model_path}")
            return True  # Intentar de todos modos

        # Estimación: el modelo en VRAM ocupa ~1.2x el tamaño del GGUF
        estimated_vram_mb = model_size_mb * 1.2
        
        if estimated_vram_mb > self.VRAM_LIMIT_MB:
            logger.error(
                f"Modelo demasiado grande: {model_size_mb:.0f}MB "
                f"(estimado en VRAM: {estimated_vram_mb:.0f}MB, "
                f"límite: {self.VRAM_LIMIT_MB}MB)"
            )
            return False

        logger.info(
            f"Tamaño modelo: {model_size_mb:.0f}MB "
            f"(VRAM estimada: {estimated_vram_mb:.0f}MB)"
        )
        return True

    def load_model(
        self,
        model_path: str,
        n_ctx: int = 2048,
        n_gpu_layers: int = -1,
        verbose: bool = False,
        **kwargs,
    ):
        """Carga un modelo GGUF. Descarga el modelo anterior si existe.
        
        Args:
            model_path: Ruta absoluta al archivo .gguf
            n_ctx: Tamaño del contexto (tokens)
            n_gpu_layers: Capas en GPU (-1 = todas)
            verbose: Activar logs de llama.cpp
            **kwargs: Argumentos extra para Llama()
            
        Returns:
            Instancia de Llama lista para inferencia
            
        Raises:
            FileNotFoundError: Si el modelo no existe
            MemoryError: Si no hay VRAM suficiente
            RuntimeError: Si falla la carga del modelo
        """
        from llama_cpp import Llama

        model_path = str(Path(model_path).resolve())

        # Si ya está cargado el mismo modelo, reutilizar
        if self._current_model is not None and self._current_model_path == model_path:
            logger.info(f"Modelo ya cargado: {Path(model_path).name}")
            return self._current_model

        # Verificar que existe
        if not os.path.isfile(model_path):
            raise FileNotFoundError(
                f"Modelo GGUF no encontrado: {model_path}\n"
                f"Descárgalo y colócalo en la ruta indicada."
            )

        # Descargar modelo anterior si hay uno
        if self._current_model is not None:
            logger.info("Descargando modelo anterior para liberar VRAM...")
            self.unload_model()

        # Verificar VRAM
        if not self._check_vram_available(model_path):
            raise MemoryError(
                f"VRAM insuficiente para cargar {Path(model_path).name}. "
                f"Límite: {self.VRAM_LIMIT_MB}MB"
            )

        # Cargar modelo
        model_name = Path(model_path).name
        logger.info(f"Cargando modelo: {model_name} (n_ctx={n_ctx}, n_gpu_layers={n_gpu_layers})")
        
        start_time = time.time()
        try:
            self._current_model = Llama(
                model_path=model_path,
                n_ctx=n_ctx,
                n_gpu_layers=n_gpu_layers,
                verbose=verbose,
                **kwargs,
            )
        except Exception as e:
            logger.error(f"Error cargando modelo: {e}")
            self._current_model = None
            raise RuntimeError(f"Fallo al cargar {model_name}: {e}") from e

        self._load_time = time.time() - start_time
        self._current_model_path = model_path
        self._current_model_size_mb = self._get_model_size_mb(model_path)

        logger.info(
            f"✓ Modelo cargado en {self._load_time:.1f}s: {model_name} "
            f"({self._current_model_size_mb:.0f}MB)"
        )
        self._log_system_memory()

        return self._current_model

    def unload_model(self) -> None:
        """Descarga el modelo actual de la memoria y libera recursos."""
        if self._current_model is None:
            logger.debug("No hay modelo cargado para descargar")
            return

        model_name = Path(self._current_model_path).name if self._current_model_path else "desconocido"
        logger.info(f"Descargando modelo: {model_name}")

        start_time = time.time()

        # Liberar el modelo
        try:
            del self._current_model
        except Exception as e:
            logger.warning(f"Error eliminando modelo: {e}")

        self._current_model = None
        self._current_model_path = None
        self._current_model_size_mb = 0.0

        # Forzar recolección de basura
        gc.collect()

        elapsed = time.time() - start_time
        logger.info(f"✓ Modelo descargado en {elapsed:.2f}s")
        self._log_system_memory()

    @property
    def is_model_loaded(self) -> bool:
        """Retorna True si hay un modelo cargado actualmente."""
        return self._current_model is not None

    @property
    def current_model_info(self) -> dict:
        """Retorna información del modelo cargado actualmente."""
        if self._current_model is None:
            return {"loaded": False}
        return {
            "loaded": True,
            "path": self._current_model_path,
            "name": Path(self._current_model_path).name if self._current_model_path else None,
            "size_mb": self._current_model_size_mb,
            "load_time_s": self._load_time,
        }

    def get_model(self):
        """Retorna el modelo actualmente cargado o None."""
        return self._current_model

    def __del__(self):
        """Limpieza al destruir el MemoryManager."""
        try:
            self.unload_model()
        except Exception:
            pass
