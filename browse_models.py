#!/usr/bin/env python3
"""Script para explorar y seleccionar modelos disponibles en OpenRouter."""
import os
import sys
import json
import httpx

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from config import OPENROUTER


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

        # Verificar si es gratuito
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


def main():
    print("=== Explorador de Modelos OpenRouter ===\n")
    
    models = fetch_models()
    if not models:
        print("No se pudieron obtener modelos.")
        return

    print(f"Total modelos disponibles: {len(models)}")
    
    # Agrupar por proveedor
    providers = {}
    for m in models:
        provider = m["id"].split("/")[0] if "/" in m["id"] else "unknown"
        providers[provider] = providers.get(provider, 0) + 1

    print("\nProveedores disponibles:")
    for p, count in sorted(providers.items()):
        print(f"  {p}: {count} modelos")

    # Menú interactivo
    while True:
        print("\n" + "="*60)
        print("Opciones:")
        print("  1. Buscar por nombre/ID")
        print("  2. Filtrar por proveedor")
        print("  3. Solo modelos gratuitos")
        print("  4. Top 20 modelos gratuitos")
        print("  5. Ver detalles de un modelo")
        print("  6. Salir")
        
        choice = input("\nElige opción (1-6): ").strip()
        
        if choice == "1":
            query = input("Buscar: ").strip()
            results = filter_models(models, query=query)
            print(f"\n{len(results)} resultados:")
            for i, m in enumerate(results[:20]):
                print_model(m, i)
            if len(results) > 20:
                print(f"... y {len(results) - 20} más")
                
        elif choice == "2":
            provider = input("Proveedor (ej: google, openai, meta-llama): ").strip()
            results = filter_models(models, provider=provider)
            print(f"\n{len(results)} modelos de '{provider}':")
            for i, m in enumerate(results[:30]):
                print_model(m, i)
                
        elif choice == "3":
            results = filter_models(models, free_only=True)
            print(f"\n{len(results)} modelos gratuitos:")
            for i, m in enumerate(results[:30]):
                print_model(m, i)
                
        elif choice == "4":
            results = filter_models(models, free_only=True)
            free_sorted = sorted(results, key=lambda x: x["id"])
            print(f"\nTop 20 modelos gratuitos:")
            for i, m in enumerate(free_sorted[:20]):
                print_model(m, i)
                
        elif choice == "5":
            model_id = input("ID del modelo: ").strip()
            found = next((m for m in models if m["id"] == model_id), None)
            if found:
                print("\n" + json.dumps(found, indent=2, ensure_ascii=False))
            else:
                print("❌ Modelo no encontrado")
                
        elif choice == "6":
            break
            
        else:
            print("Opción inválida")


if __name__ == "__main__":
    main()