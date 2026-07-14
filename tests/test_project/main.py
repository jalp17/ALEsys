#!/usr/bin/env python3
"""
main.py — CLI de la calculadora de prueba.

Punto de entrada principal que combina las operaciones
de calculadora.py y operaciones.py en una interfaz de
línea de comandos interactiva.
"""

from calculadora import Calculadora
from operaciones import (
    seno, coseno, tangente,
    media, mediana, desviacion_estandar,
    factorial, raiz_cuadrada, es_primo,
)


def mostrar_menu() -> None:
    """Muestra el menú principal de la calculadora."""
    print("""
╔══════════════════════════════════════╗
║      CALCULADORA CIENTÍFICA         ║
╠══════════════════════════════════════╣
║  1. Operaciones básicas             ║
║  2. Trigonometría                   ║
║  3. Estadísticas                    ║
║  4. Funciones especiales            ║
║  5. Ver historial                   ║
║  6. Salir                           ║
╚══════════════════════════════════════╝
""")


def menu_basicas(calc: Calculadora) -> None:
    """Submenú de operaciones básicas."""
    print("\nOperaciones: +, -, *, /, ^, mod")
    try:
        a = float(input("  Primer número: "))
        op = input("  Operación (+, -, *, /, ^, mod): ").strip()
        b = float(input("  Segundo número: "))

        operaciones = {
            "+": calc.sumar,
            "-": calc.restar,
            "*": calc.multiplicar,
            "/": calc.dividir,
            "^": calc.potencia,
            "mod": calc.modulo,
        }

        if op not in operaciones:
            print(f"  ✗ Operación '{op}' no reconocida")
            return

        resultado = operaciones[op](a, b)
        print(f"  ✓ Resultado: {resultado}")

    except (ValueError, ZeroDivisionError) as e:
        print(f"  ✗ Error: {e}")


def menu_trigonometria() -> None:
    """Submenú de funciones trigonométricas."""
    print("\nFunciones: sin, cos, tan")
    try:
        angulo = float(input("  Ángulo (grados): "))
        func = input("  Función (sin/cos/tan): ").strip().lower()

        funciones = {"sin": seno, "cos": coseno, "tan": tangente}

        if func not in funciones:
            print(f"  ✗ Función '{func}' no reconocida")
            return

        resultado = funciones[func](angulo)
        print(f"  ✓ {func}({angulo}°) = {resultado}")

    except (ValueError,) as e:
        print(f"  ✗ Error: {e}")


def menu_estadisticas() -> None:
    """Submenú de funciones estadísticas."""
    print("\nIngresa números separados por comas:")
    try:
        entrada = input("  Números: ").strip()
        numeros = [float(x.strip()) for x in entrada.split(",")]

        print(f"  Media: {media(numeros):.4f}")
        print(f"  Mediana: {mediana(numeros):.4f}")
        if len(numeros) >= 2:
            print(f"  Desv. estándar: {desviacion_estandar(numeros):.4f}")

    except (ValueError,) as e:
        print(f"  ✗ Error: {e}")


def menu_especiales() -> None:
    """Submenú de funciones especiales."""
    print("\nFunciones: factorial, raiz, primo")
    try:
        func = input("  Función (factorial/raiz/primo): ").strip().lower()
        n = float(input("  Número: "))

        if func == "factorial":
            print(f"  ✓ {int(n)}! = {factorial(int(n))}")
        elif func == "raiz":
            print(f"  ✓ √{n} = {raiz_cuadrada(n):.6f}")
        elif func == "primo":
            es = es_primo(int(n))
            print(f"  ✓ {int(n)} {'es' if es else 'NO es'} primo")
        else:
            print(f"  ✗ Función '{func}' no reconocida")

    except (ValueError,) as e:
        print(f"  ✗ Error: {e}")


def main():
    """Función principal del CLI de la calculadora."""
    calc = Calculadora()
    print("Bienvenido a la Calculadora Científica")

    while True:
        mostrar_menu()
        opcion = input("Selecciona opción (1-6): ").strip()

        if opcion == "1":
            menu_basicas(calc)
        elif opcion == "2":
            menu_trigonometria()
        elif opcion == "3":
            menu_estadisticas()
        elif opcion == "4":
            menu_especiales()
        elif opcion == "5":
            print(f"\nHistorial:\n{calc.mostrar_historial()}")
        elif opcion == "6":
            print("¡Hasta luego!")
            break
        else:
            print(f"  ✗ Opción '{opcion}' no válida")


if __name__ == "__main__":
    main()
