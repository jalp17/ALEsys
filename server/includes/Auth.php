<?php
namespace Alesys;

/**
 * Autenticación básica (placeholder para Fase 2)
 */
class Auth {
    public static function isLoggedIn(): bool {
        // TODO: Implementar auth real con base de datos
        return true;
    }

    public static function login(string $username, string $password): bool {
        // TODO: Implementar
        return true;
    }

    public static function logout(): void {
        session_destroy();
    }
}