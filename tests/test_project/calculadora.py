"""
calculadora.py — Clase principal de la calculadora.

Implementa operaciones aritméticas básicas con historial
de operaciones y validación de entrada.
"""

from typing import Union

Number = Union[int, float]


class Calculadora:
    """Calculadora con historial de operaciones.
    
    Ejemplo de uso:
        calc = Calculadora()
        resultado = calc.sumar(5, 3)
        print(resultado)  # 8
        print(calc.historial)  # ["5 + 3 = 8"]
    """

    def __init__(self):
        """Inicializa la calculadora con historial vacío."""
        self.historial: list[str] = []
        self._ultimo_resultado: Number = 0

    def sumar(self, a: Number, b: Number) -> Number:
        """Suma dos números.
        
        Args:
            a: Primer sumando
            b: Segundo sumando
            
        Returns:
            Resultado de a + b
        """
        resultado = a + b
        self._registrar(f"{a} + {b} = {resultado}")
        return resultado

    def restar(self, a: Number, b: Number) -> Number:
        """Resta dos números.
        
        Args:
            a: Minuendo
            b: Sustraendo
            
        Returns:
            Resultado de a - b
        """
        resultado = a - b
        self._registrar(f"{a} - {b} = {resultado}")
        return resultado

    def multiplicar(self, a: Number, b: Number) -> Number:
        """Multiplica dos números.
        
        Args:
            a: Primer factor
            b: Segundo factor
            
        Returns:
            Resultado de a * b
        """
        resultado = a * b
        self._registrar(f"{a} × {b} = {resultado}")
        return resultado

    def dividir(self, a: Number, b: Number) -> float:
        """Divide dos números.
        
        Args:
            a: Dividendo
            b: Divisor
            
        Returns:
            Resultado de a / b
            
        Raises:
            ZeroDivisionError: Si b es cero
        """
        if b == 0:
            raise ZeroDivisionError("No se puede dividir por cero")
        resultado = a / b
        self._registrar(f"{a} ÷ {b} = {resultado}")
        return resultado

    def potencia(self, base: Number, exponente: Number) -> Number:
        """Calcula la potencia de un número.
        
        Args:
            base: Base de la potencia
            exponente: Exponente
            
        Returns:
            base elevado a exponente
        """
        resultado = base ** exponente
        self._registrar(f"{base}^{exponente} = {resultado}")
        return resultado

    def modulo(self, a: Number, b: Number) -> Number:
        """Calcula el módulo (resto de la división).
        
        Args:
            a: Dividendo
            b: Divisor
            
        Returns:
            Resto de a / b
            
        Raises:
            ZeroDivisionError: Si b es cero
        """
        if b == 0:
            raise ZeroDivisionError("No se puede calcular módulo con divisor cero")
        resultado = a % b
        self._registrar(f"{a} mod {b} = {resultado}")
        return resultado

    def _registrar(self, operacion: str) -> None:
        """Registra una operación en el historial."""
        self.historial.append(operacion)
        # Extraer resultado numérico
        try:
            self._ultimo_resultado = float(operacion.split("=")[-1].strip())
        except (ValueError, IndexError):
            pass

    @property
    def ultimo_resultado(self) -> Number:
        """Retorna el último resultado calculado."""
        return self._ultimo_resultado

    def limpiar_historial(self) -> None:
        """Limpia el historial de operaciones."""
        self.historial.clear()
        self._ultimo_resultado = 0

    def mostrar_historial(self) -> str:
        """Retorna el historial formateado como string."""
        if not self.historial:
            return "Historial vacío"
        return "\n".join(
            f"  {i+1}. {op}" for i, op in enumerate(self.historial)
        )
