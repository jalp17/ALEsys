#!/usr/bin/env python3
"""Script para explorar y seleccionar modelos disponibles en OpenRouter."""
import os
import sys
import json
import httpx

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from config import OPENROUTER


ENV_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".env")


def fetch_models():
    """Obtiene la lista de modelos de OpenRouter."""
    if not OPENROUTER.api_key:
        print("❌ OPENROUTER_API_KEY no configurada")
        return []

    try:
        with httpx.Client(
            base_url="https://openrouter.ai/api/v1",
            timeout=30,
            headers={
                "Authorization": f"Bearer {OPENROUTER.api_key}",
            },
        ) as client:
            response = client.get("/models")
            response.raise_for_status()
            return response.json()["data"]
    except httpx.RequestError as e:
        print(f"❌ Error de conexión: {e}")
        return []


def filter_models(models, query="", provider="", free_only=False):
    """Filtra modelos por búsqueda, proveedor y si son gratuitos."""
    results = []
    for m in models:
        model_id = m.get("id", "")
        name = m.get("name", "")
        pricing = m.get("pricing", {})
        prompt_price = float(pricing.get("prompt", "0") or "0")
        completion_price = float(pricing.get("completion", "0") or "0")

        is_free = prompt_price == 0 and completion_price == 0

        if free_only and not is_free:
            continue

        if provider and provider.lower() not in model_id.lower():
            continue

        if query and query.lower() not in model_id.lower() and query.lower() not in name.lower():
            continue

        results.append({
            "id": model_id,
            "name": name,
            "context_length": m.get("context_length", "?"),
            "pricing": pricing,
            "is_free": is_free,
        })
    return results


def print_model(m, index=None):
    """Imprime información de un modelo."""
    prefix = f"[{index}] " if index is not None else ""
    free_tag = " 🆓 GRATIS" if m["is_free"] else ""
    print(f"{prefix}{m['id']}{free_tag}")
    print(f"      Nombre: {m['name']}")
    print(f"      Contexto: {m['context_length']}")
    if not m["is_free"]:
        p = m["pricing"]
        print(f"      Precio: ${p.get('prompt', '?')}/1M prompt, ${p.get('completion', '?')}/1M completion")
    print()


def load_env():
    """Carga el archivo .env existente."""
    env_vars = {}
    if os.path.exists(ENV_FILE):
        with open(ENV_FILE) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    env_vars[k.strip()] = v.strip()
    return env_vars


def save_model_to_env(model_id):
    """Guarda el modelo seleccionado en .env"""
    env_vars = load_env()
    env_vars["OPENROUTER_MODEL"] = model_id

    with open(ENV_FILE, "w") as f:
        f.write("# ALEsys - Variables de entorno\n")
        for k, v in env_vars.items():
            f.write(f"{k}={v}\n")

    print(f"\n✅ Modelo guardado en {ENV_FILE}: OPENROUTER_MODEL={model_id}")
    print("   El pipeline usará este modelo en la próxima ejecución.")


def select_and_configure_model(models):
    """Permite seleccionar un modelo y configurarlo."""
    print("\n--- Seleccionar y configurar modelo ---")
    print("Ingresa el número del modelo de la lista mostrada,")
    print("o escribe el ID completo del modelo.")

    model_id = input("\nModelo a usar: ").strip()

    if model_id.isdigit():
        idx = int(model_id)
        if 0 <= idx < len(models):
            model_id = models[idx]["id"]
        else:
            print("❌ Índice inválido")
            return

    found = next((m for m in models if m["id"] == model_id), None)
    if not found:
        print(f"❌ Modelo '{model_id}' no encontrado")
        return

    print(f"\nSeleccionado: {found['id']}")
    print(f"             {found['name']}")

    confirm = input("¿Guardar como modelo por defecto? (s/n): ").strip().lower()
    if confirm == "s":
        save_model_to_env(model_id)

        test = input("¿Probar el modelo ahora? (s/n): ").strip().lower()
        if test == "s":
            test_model(found["id"])


def test_model(model_id):
    """Prueba un modelo específico."""
    print(f"\nProbando {model_id}...")
    try:
        with httpx.Client(
            base_url="https://openrouter.ai/api/v1",
            timeout=30,
            headers={
                "Authorization": f"Bearer {OPENROUTER.api_key}",
                "Content-Type": "application/json",
            },
        ) as client:
            resp = client.post("/chat/completions", json={
                "model": model_id,
                "messages": [{"role": "user", "content": "Di OK"}],
                "max_tokens": 20,
            })
            if resp.status_code == 200:
                print(f"✅ Funciona: {resp.json()['choices'][0]['message']['content'][:50]}")
            else:
                print(f"❌ Error {resp.status_code}: {resp.text[:200]}")
    except Exception as e:
        print(f"❌ Error: {e}")


def main():
    print("=== Explorador y Configurador de Modelos OpenRouter ===\n")

    models = fetch_models()
    if not models:
        return

    print(f"Total modelos: {len(models)}")
    free_count = sum(1 for m in models if float(m.get("pricing", {}).get("prompt", "0") or "0") == 0)
    print(f"Gratuitos: {free_count}")

    current_model = os.getenv("OPENROUTER_MODEL", "google/gemma-4-31b-it:free")
    print(f"Modelo actual: {current_model}")

    while True:
        print("\n" + "="*60)
        print("Opciones:")
        print("  1. Buscar por nombre/ID")
        print("  2. Filtrar por proveedor")
        print("  3. Solo modelos gratuitos")
        print("  4. Top 20 gratuitos")
        print("  5. Seleccionar y CONFIGURAR modelo")
        print("  6. Probar modelo actual")
        print("  7. Salir")

        choice = input("\nElige (1-7): ").strip()

        if choice == "1":
            query = input("Buscar: ").strip()
            results = filter_models(models, query=query)
            print(f"\n{len(results)} resultados:")
            for i, m in enumerate(results[:20]):
                print_model(m, i)

        elif choice == "2":
            provider = input("Proveedor (google, openai, meta-llama...): ").strip()
            results = filter_models(models, provider=provider)
            print(f"\n{len(results)} modelos de '{provider}':")
            for i, m in enumerate(results[:30]):
                print_model(m, i)

        elif choice == "3":
            results = filter_models(models, free_only=True)
            print(f"\n{len(results)} gratuitos:")
            for i, m in enumerate(results[:30]):
                print_model(m, i)

        elif choice == "4":
            results = filter_models(models, free_only=True)
            for i, m in enumerate(sorted(results, key=lambda x: x["id"])[:20]):
                print_model(m, i)

        elif choice == "5":
            select_and_configure_model(models)

        elif choice == "6":
            test_model(current_model)

        elif choice == "7":
            break


if __name__ == "__main__":
    main()