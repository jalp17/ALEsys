#!/usr/bin/env python3
"""
Script para instalar y configurar llama-cpp-python con soporte para GPU (CUDA/Vulkan).
Requisitos:
- Python 3.8+
- pip
- CMake
- Compilador C++ (gcc/clang)
- CUDA Toolkit (opcional, para GPU)
"""

import os
import subprocess
import sys
from pathlib import Path


def install_vulkan_tools():
    """Instala vulkan-tools si no está disponible."""
    try:
        subprocess.run(["apt-get", "update"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        subprocess.run(["apt-get", "install", "-y", "vulkan-tools"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        print("✅ vulkan-tools instalado.")
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("⚠️ No se pudo instalar vulkan-tools (solo funciona en sistemas basados en Debian/Ubuntu).")
        return False


def detect_gpu_support():
    """Detecta soporte para CUDA (NVIDIA) y Vulkan."""
    cuda_available = False
    vulkan_available = False
    
    # Detectar CUDA (NVIDIA)
    try:
        subprocess.run(["nvidia-smi"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        cuda_available = True
        print("✅ CUDA (NVIDIA GPU) detectado.")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("⚠️ CUDA no disponible (no se detectó GPU NVIDIA).")
    
    # Detectar Vulkan (método 1: vulkaninfo)
    try:
        subprocess.run(["vulkaninfo"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        vulkan_available = True
        print("✅ Vulkan detectado (vulkaninfo).")
    except FileNotFoundError:
        print("⚠️ vulkaninfo no encontrado. Intentando instalar vulkan-tools...")
        if install_vulkan_tools():
            try:
                subprocess.run(["vulkaninfo"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
                vulkan_available = True
                print("✅ Vulkan detectado después de instalar vulkan-tools.")
            except (subprocess.CalledProcessError, FileNotFoundError):
                print("⚠️ Vulkan no disponible incluso después de instalar vulkan-tools.")
        else:
            print("⚠️ No se pudo verificar Vulkan (vulkaninfo no disponible).")
    except subprocess.CalledProcessError:
        print("⚠️ Vulkan no disponible (error al ejecutar vulkaninfo).")
    
    # Método alternativo: buscar librerías Vulkan
    if not vulkan_available:
        try:
            result = subprocess.run(["ldconfig", "-p"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
            if b"libvulkan.so" in result.stdout:
                vulkan_available = True
                print("✅ Vulkan detectado (libvulkan.so encontrado).")
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("⚠️ No se pudo verificar la presencia de libvulkan.so.")
    
    return cuda_available, vulkan_available


def install_llama_cpp():
    """Instala llama-cpp-python con soporte para GPU."""
    print("🔧 Instalando llama-cpp-python...")
    
    # Detectar soporte para GPU
    cuda_available, vulkan_available = detect_gpu_support()
    
    # Configurar CMAKE_ARGS según el hardware detectado
    cmake_args = []
    if cuda_available:
        cmake_args.append("-DGGML_CUDA=on")
    else:
        cmake_args.append("-DGGML_CUDA=off")
    
    if vulkan_available:
        cmake_args.append("-DLLAMA_VULKAN=on")
    else:
        cmake_args.append("-DLLAMA_VULKAN=off")
    
    env = os.environ.copy()
    env["CMAKE_ARGS"] = " ".join(cmake_args)
    print(f"🛠️ Configuración de compilación: {env['CMAKE_ARGS']}")
    
    try:
        subprocess.run(
            [
                sys.executable, "-m", "pip", "install", 
                "llama-cpp-python", "--force-reinstall", "--upgrade"
            ],
            env=env,
            check=True,
            timeout=1800  # 30 minutos
        )
        print("✅ llama-cpp-python instalado correctamente.")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Error al instalar llama-cpp-python: {e}")
        return False
    except subprocess.TimeoutExpired:
        print("❌ Timeout: La instalación tardó demasiado. Reintenta con más tiempo.")
        return False


def verify_installation():
    """Verifica que llama-cpp-python esté instalado."""
    try:
        import llama_cpp
        print(f"✅ llama-cpp-python versión: {llama_cpp.__version__}")
        return True
    except ImportError:
        print("❌ llama-cpp-python no está instalado.")
        return False


def download_model(model_url: str, output_path: str):
    """Descarga un modelo GGUF desde una URL."""
    print(f"📥 Descargando modelo GGUF desde {model_url}...")
    
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    try:
        subprocess.run(
            ["wget", "-O", str(output_path), model_url],
            check=True,
            timeout=3600  # 1 hora
        )
        print(f"✅ Modelo descargado en {output_path}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Error al descargar el modelo: {e}")
        return False
    except subprocess.TimeoutExpired:
        print("❌ Timeout: La descarga tardó demasiado.")
        return False


def configure_env(model_path: str):
    """Configura las variables de entorno para llama.cpp."""
    print("🛠️ Configurando variables de entorno...")
    
    env_file = Path(".env")
    lines = []
    
    if env_file.exists():
        with open(env_file, "r") as f:
            lines = f.readlines()
    
    # Actualizar o agregar variables
    new_lines = []
    model_path_added = False
    backend_added = False
    
    for line in lines:
        if line.startswith("EMBEDDING_BACKEND="):
            new_lines.append("EMBEDDING_BACKEND=llama.cpp\n")
            backend_added = True
        elif line.startswith("EMBEDDING_GGUF_PATH="):
            new_lines.append(f"EMBEDDING_GGUF_PATH={model_path}\n")
            model_path_added = True
        else:
            new_lines.append(line)
    
    if not backend_added:
        new_lines.append("EMBEDDING_BACKEND=llama.cpp\n")
    if not model_path_added:
        new_lines.append(f"EMBEDDING_GGUF_PATH={model_path}\n")
    
    # Agregar variables adicionales si no existen
    additional_vars = [
        "EMBEDDING_LLAMA_CPP_LIB_PATH=\n",
        "EMBEDDING_DEVICE=cpu\n",
        "EMBEDDING_DIM=384\n"
    ]
    
    for var in additional_vars:
        if not any(var.startswith(line.split("=")[0]) for line in new_lines):
            new_lines.append(var)
    
    with open(env_file, "w") as f:
        f.writelines(new_lines)
    
    print("✅ Variables de entorno configuradas en .env")


def main():
    print("🚀 Configuración de llama-cpp-python para ALEsys")
    
    # 1. Instalar llama-cpp-python
    if not install_llama_cpp():
        print("❌ Falló la instalación de llama-cpp-python.")
        sys.exit(1)
    
    # 2. Verificar instalación
    if not verify_installation():
        print("❌ Falló la verificación de llama-cpp-python.")
        sys.exit(1)
    
    # 3. Descargar modelo (opcional)
    model_url = input(
        "📥 Ingresa la URL del modelo GGUF (ej: https://huggingface.co/.../model.gguf) o deja vacío para omitir: "
    ).strip()
    
    if model_url:
        model_path = input(
            "📁 Ingresa la ruta donde guardar el modelo (ej: ./models/bge-small-en-v1.5-f16.gguf): "
        ).strip()
        
        if not download_model(model_url, model_path):
            print("⚠️ Advertencia: No se descargó el modelo.")
        else:
            configure_env(model_path)
    else:
        print("⏩ Omitiendo descarga de modelo.")
        
    print("\n🎉 Configuración completada.")
    print("📌 Para usar llama.cpp, ejecuta:")
    print("   python pipeline.py")


if __name__ == "__main__":
    main()