<?php
namespace Alesys;

/**
 * Proxy hacia el backend Rust
 */
class AlesysProxy {
    private $baseURL;
    private $httpClient;

    public function __construct() {
        $this->baseURL = getenv('ALESYS_CORE_URL') ?: 'http://alesys-core:3000';
        
        $this->httpClient = new \GuzzleHttp\Client([
            'base_uri' => $this->baseURL,
            'timeout' => 30.0,
            'headers' => [
                'Authorization' => 'Bearer ' . $this->getServiceToken(),
            ],
        ]);
    }

    public function chat(string $query, string $sessionId): array {
        $response = $this->httpClient->post('/api/chat', [
            'json' => [
                'query' => $query,
                'session_id' => $sessionId,
            ],
        ]);

        return json_decode($response->getBody()->getContents(), true);
    }

    public function generate(string $prompt, string $filePath, string $language): array {
        $response = $this->httpClient->post('/api/generate', [
            'json' => [
                'prompt' => $prompt,
                'file_path' => $filePath,
                'language' => $language,
            ],
        ]);

        return json_decode($response->getBody()->getContents(), true);
    }

    private function getServiceToken(): string {
        return getenv('ALESYS_SERVICE_TOKEN') ?: 'dev-token';
    }
}