#!/usr/bin/env python3
"""Script para probar la conexión a OpenRouter y el modelo configurado."""
import os
import sys

# Añadir el directorio del proyecto al path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from config import OPENROUTER
import httpx


def test_openrouter():
    print(f"Modelo configurado: {OPENROUTER.model}")
    print(f"Base URL: {OPENROUTER.base_url}")
    print(f"API Key: {'configurada' if OPENROUTER.api_key else 'NO CONFIGURADA'}")
    print("-" * 50)

    if not OPENROUTER.api_key:
        print("ERROR: OPENROUTER_API_KEY no está configurada")
        return False

    payload = {
        "model": OPENROUTER.model,
        "messages": [{"role": "user", "content": "Di 'OK' si recibes este mensaje"}],
        "max_tokens": 50,
        "temperature": 0.1,
    }

    try:
        with httpx.Client(
            base_url=OPENROUTER.base_url,
            timeout=30,
            headers={
                "Authorization": f"Bearer {OPENROUTER.api_key}",
                "Content-Type": "application/json",
            },
        ) as client:
            print("Enviando petición a OpenRouter...")
            response = client.post("/chat/completions", json=payload)
            
            print(f"Status code: {response.status_code}")
            
            if response.status_code == 200:
                data = response.json()
                content = data["choices"][0]["message"]["content"]
                print(f"✅ Respuesta: {content}")
                return True
            else:
                print(f"❌ Error {response.status_code}: {response.text}")
                return False
                
    except httpx.TimeoutException:
        print("❌ Timeout: La petición tardó demasiado")
        return False
    except httpx.RequestError as e:
        print(f"❌ Error de conexión: {e}")
        return False


if __name__ == "__main__":
    success = test_openrouter()
    sys.exit(0 if success else 1)