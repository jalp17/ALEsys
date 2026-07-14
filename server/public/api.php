<?php
/**
 * ALEsys API Proxy
 * 
 * Proxy PHP hacia el backend Rust para multi-usuario
 */

header('Content-Type: application/json');
session_start();

require_once __DIR__ . '/../includes/AlesysProxy.php';
require_once __DIR__ . '/../includes/Auth.php';

// Verificar autenticación
if (!Auth::isLoggedIn()) {
    http_response_code(401);
    echo json_encode(['error' => 'No autorizado']);
    exit;
}

$proxy = new Alesys\AlesysProxy();
$sessionId = $_SESSION['alesys_session'] ?? null;

if (!$sessionId) {
    // Crear sesión si no existe
    $sessionManager = new Alesys\SessionManager();
    $sessionId = $sessionManager->createSession($_SESSION['user_id'] ?? 1);
    $_SESSION['alesys_session'] = $sessionId;
}

// Router simple
$endpoint = $_GET['endpoint'] ?? '';
$method = $_SERVER['REQUEST_METHOD'];

try {
    switch ($endpoint) {
        case 'chat':
            if ($method !== 'POST') throw new Exception('Método no permitido');
            handleChat($proxy, $sessionId);
            break;
        
        case 'sessions':
            if ($method === 'GET') {
                handleListSessions($sessionId);
            } elseif ($method === 'POST') {
                handleCreateSession($sessionId);
            } else {
                throw new Exception('Método no permitido');
            }
            break;
        
        default:
            throw new Exception('Endpoint no encontrado: ' . $endpoint);
    }
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['error' => $e->getMessage()]);
}

function handleChat($proxy, $sessionId) {
    $input = json_decode(file_get_contents('php://input'), true);
    $query = $input['query'] ?? '';
    
    if (empty($query)) {
        throw new Exception('Query requerido');
    }
    
    $response = $proxy->chat($query, $sessionId);
    echo json_encode($response);
}

function handleListSessions($sessionId) {
    // TODO: Implementar
    echo json_encode(['sessions' => []]);
}

function handleCreateSession($sessionId) {
    // TODO: Implementar
    echo json_encode(['session_id' => $sessionId]);
}