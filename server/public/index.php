<?php
/**
 * ALEsys WebUI - Entry Point
 * 
 * Sirve el frontend y maneja autenticación básica
 */

session_start();

// Verificar autenticación (placeholder para Fase 2)
$isLoggedIn = true;  // TODO: Implementar auth real en Fase 2
$userId = $_SESSION['user_id'] ?? 1;

// Servir frontend desde el build de webui
$frontendPath = __DIR__ . '/../webui/dist/index.html';

if (!file_exists($frontendPath)) {
    die('Frontend no encontrado. Ejecuta: cd webui && npm run build:web');
}

// Inyectar configuración para el frontend
$content = file_get_contents($frontendPath);
$content = str_replace(
    '</head>',
    '<script>
        window.VITE_CONFIG = {
            mode: "web",
            apiBase: "/api",
            wsUrl: "wss://" + window.location.host + "/ws",
            sessionId: "' . session_id() . '",
            userId: ' . $userId . '
        };
    </script>
    </head>',
    $content
);

echo $content;