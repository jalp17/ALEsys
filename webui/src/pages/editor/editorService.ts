/**
 * Editor API Client (Phase 7)
 *
 * Provides typed access to file operations and code execution endpoints.
 */

// =============================================================================
// Types
// =============================================================================

export interface FileTreeEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  children?: FileTreeEntry[];
}

export interface FileContent {
  path: string;
  content: string;
}

export interface DiffResult {
  diff: string;
  lines_added: number;
  lines_removed: number;
  old_content: string;
  new_content: string;
}

export interface FileOperationResult {
  success: boolean;
  message?: string;
  path: string;
}

export interface ExecutionResult {
  exit_code: number;
  stdout: string;
  stderr: string;
  execution_time_ms: number;
  timed_out: boolean;
  language: string;
}

// =============================================================================
// API Client
// =============================================================================

const API_BASE = import.meta.env.VITE_API_URL || '';

/**
 * List files in a directory
 */
export async function listFiles(path: string = ''): Promise<FileTreeEntry[]> {
  const params = new URLSearchParams();
  if (path) params.set('path', path);

  const response = await fetch(`${API_BASE}/api/v1/files?${params}`);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `List files failed: ${response.status}`);
  }

  const data = await response.json();
  return data.entries || [];
}

/**
 * Read file contents
 */
export async function readFile(path: string): Promise<string> {
  const response = await fetch(`${API_BASE}/api/v1/files/${encodeURIComponent(path)}`);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `Read file failed: ${response.status}`);
  }

  const data = await response.json();
  return data.content;
}

/**
 * Write file contents
 */
export async function writeFile(path: string, content: string): Promise<FileOperationResult> {
  const response = await fetch(`${API_BASE}/api/v1/files`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, content }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `Write file failed: ${response.status}`);
  }

  return response.json();
}

/**
 * Modify file with diff generation (requires old_content match)
 */
export async function modifyFile(
  path: string,
  oldContent: string,
  newContent: string
): Promise<DiffResult> {
  const response = await fetch(`${API_BASE}/api/v1/modify`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, old_content: oldContent, new_content: newContent }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `Modify file failed: ${response.status}`);
  }

  return response.json();
}

/**
 * Execute code in sandbox
 */
export async function executeCode(
  code: string,
  language: string,
  timeoutMs: number = 30000,
  memoryLimitMb: number = 256
): Promise<ExecutionResult> {
  const response = await fetch(`${API_BASE}/api/v1/execute`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      code,
      language,
      timeout_ms: timeoutMs,
      memory_limit_mb: memoryLimitMb,
    }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `Execute failed: ${response.status}`);
  }

  return response.json();
}

// =============================================================================
// Helpers
// =============================================================================

/**
 * Detect language from file extension
 */
export function detectLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'rs':
      return 'rust';
    case 'py':
      return 'python';
    case 'js':
    case 'jsx':
    case 'mjs':
      return 'javascript';
    case 'ts':
    case 'tsx':
      return 'typescript';
    case 'json':
      return 'json';
    case 'md':
      return 'markdown';
    case 'html':
      return 'html';
    case 'css':
      return 'css';
    case 'sql':
      return 'sql';
    case 'yaml':
    case 'yml':
      return 'yaml';
    case 'toml':
      return 'toml';
    case 'sh':
    case 'bash':
      return 'shell';
    default:
      return 'plaintext';
  }
}

/**
 * Get language for code execution from file language
 */
export function getExecLanguage(editorLanguage: string): string {
  switch (editorLanguage) {
    case 'python':
      return 'python';
    case 'javascript':
    case 'typescript':
      return 'javascript';
    case 'rust':
      return 'rust';
    default:
      return 'python';
  }
}
