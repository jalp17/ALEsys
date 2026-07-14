"""
operaciones.py — Funciones matemáticas avanzadas.

Incluye operaciones trigonométricas, estadísticas
y funciones de utilidad matemática.
"""

import math
from typing import Sequence

Number = int | float


# ══════════════════════════════════════════════════
# FUNCIONES TRIGONOMÉTRICAS
# ══════════════════════════════════════════════════

def seno(angulo_grados: Number) -> float:
    """Calcula el seno de un ángulo en grados.
    
    Args:
        angulo_grados: Ángulo en grados sexagesimales
        
    Returns:
        Valor del seno (entre -1 y 1)
    """
    radianes = math.radians(angulo_grados)
    return round(math.sin(radianes), 10)


def coseno(angulo_grados: Number) -> float:
    """Calcula el coseno de un ángulo en grados.
    
    Args:
        angulo_grados: Ángulo en grados sexagesimales
        
    Returns:
        Valor del coseno (entre -1 y 1)
    """
    radianes = math.radians(angulo_grados)
    return round(math.cos(radianes), 10)


def tangente(angulo_grados: Number) -> float:
    """Calcula la tangente de un ángulo en grados.
    
    Args:
        angulo_grados: Ángulo en grados sexagesimales
        
    Returns:
        Valor de la tangente
        
    Raises:
        ValueError: Si el ángulo produce tangente indefinida (90°, 270°, etc.)
    """
    if angulo_grados % 180 == 90:
        raise ValueError(f"Tangente indefinida para {angulo_grados}°")
    radianes = math.radians(angulo_grados)
    return round(math.tan(radianes), 10)


# ══════════════════════════════════════════════════
# FUNCIONES ESTADÍSTICAS
# ══════════════════════════════════════════════════

def media(numeros: Sequence[Number]) -> float:
    """Calcula la media aritmética.
    
    Args:
        numeros: Secuencia de números
        
    Returns:
        Media aritmética
        
    Raises:
        ValueError: Si la secuencia está vacía
    """
    if not numeros:
        raise ValueError("La secuencia no puede estar vacía")
    return sum(numeros) / len(numeros)


def mediana(numeros: Sequence[Number]) -> float:
    """Calcula la mediana.
    
    Args:
        numeros: Secuencia de números
        
    Returns:
        Mediana de la secuencia
        
    Raises:
        ValueError: Si la secuencia está vacía
    """
    if not numeros:
        raise ValueError("La secuencia no puede estar vacía")
    
    ordenados = sorted(numeros)
    n = len(ordenados)
    
    if n % 2 == 0:
        return (ordenados[n // 2 - 1] + ordenados[n // 2]) / 2
    return float(ordenados[n // 2])


def desviacion_estandar(numeros: Sequence[Number]) -> float:
    """Calcula la desviación estándar poblacional.
    
    Args:
        numeros: Secuencia de números
        
    Returns:
        Desviación estándar
        
    Raises:
        ValueError: Si la secuencia tiene menos de 2 elementos
    """
    if len(numeros) < 2:
        raise ValueError("Se necesitan al menos 2 elementos")
    
    m = media(numeros)
    varianza = sum((x - m) ** 2 for x in numeros) / len(numeros)
    return math.sqrt(varianza)


def rango(numeros: Sequence[Number]) -> Number:
    """Calcula el rango (máximo - mínimo).
    
    Args:
        numeros: Secuencia de números
        
    Returns:
        Diferencia entre el valor máximo y mínimo
    """
    if not numeros:
        raise ValueError("La secuencia no puede estar vacía")
    return max(numeros) - min(numeros)


# ══════════════════════════════════════════════════
# FUNCIONES DE UTILIDAD
# ══════════════════════════════════════════════════

def factorial(n: int) -> int:
    """Calcula el factorial de un número entero no negativo.
    
    Args:
        n: Número entero >= 0
        
    Returns:
        n! (factorial de n)
        
    Raises:
        ValueError: Si n es negativo
    """
    if n < 0:
        raise ValueError("El factorial no está definido para números negativos")
    return math.factorial(n)


def raiz_cuadrada(n: Number) -> float:
    """Calcula la raíz cuadrada.
    
    Args:
        n: Número >= 0
        
    Returns:
        Raíz cuadrada de n
        
    Raises:
        ValueError: Si n es negativo
    """
    if n < 0:
        raise ValueError("No se puede calcular la raíz cuadrada de un número negativo")
    return math.sqrt(n)


def logaritmo(n: Number, base: Number = math.e) -> float:
    """Calcula el logaritmo.
    
    Args:
        n: Número > 0
        base: Base del logaritmo (default: e para logaritmo natural)
        
    Returns:
        Logaritmo de n en la base dada
        
    Raises:
        ValueError: Si n <= 0 o base <= 0/1
    """
    if n <= 0:
        raise ValueError("El logaritmo requiere un número positivo")
    if base <= 0 or base == 1:
        raise ValueError("La base debe ser positiva y diferente de 1")
    return math.log(n, base)


def es_primo(n: int) -> bool:
    """Verifica si un número es primo.
    
    Args:
        n: Número entero
        
    Returns:
        True si n es primo, False en caso contrario
    """
    if n < 2:
        return False
    if n < 4:
        return True
    if n % 2 == 0 or n % 3 == 0:
        return False
    i = 5
    while i * i <= n:
        if n % i == 0 or n % (i + 2) == 0:
            return False
        i += 6
    return True
