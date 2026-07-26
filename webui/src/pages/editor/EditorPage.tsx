import { useState, useCallback } from 'react';
import { FileTree } from '../../components/editor/FileTree';
import { MonacoEditor } from '../../components/editor/MonacoEditor';
import { Terminal } from '../../components/editor/Terminal';
import { SuggestionsPanel } from '../../components/PairProgrammerPanel';
import {
  readFile,
  writeFile,
  executeCode,
  detectLanguage,
  getExecLanguage,
} from './editorService';

interface OpenFile {
  path: string;
  content: string;
  originalContent: string;
  language: string;
}

interface ExecOutput {
  stdout: string;
  stderr: string;
  exitCode: number;
  timeMs: number;
}

export default function EditorPage() {
  const [openFiles, setOpenFiles] = useState<OpenFile[]>([]);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [output, setOutput] = useState<ExecOutput | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const activeFile = openFiles.find((f) => f.path === activePath);

  const handleFileSelect = useCallback(
    async (path: string) => {
      // Already open?
      if (openFiles.some((f) => f.path === path)) {
        setActivePath(path);
        return;
      }

      try {
        const content = await readFile(path);
        const language = detectLanguage(path);
        setOpenFiles((prev) => [
          ...prev,
          {
            path,
            content,
            originalContent: content,
            language,
          },
        ]);
        setActivePath(path);
      } catch (e) {
        console.error('Error loading file:', e);
      }
    },
    [openFiles]
  );

  const handleContentChange = useCallback(
    (value: string) => {
      setOpenFiles((prev) =>
        prev.map((f) => (f.path === activePath ? { ...f, content: value } : f))
      );
    },
    [activePath]
  );

  const handleSave = useCallback(
    async (value: string) => {
      if (!activePath) return;
      try {
        await writeFile(activePath, value);
        setOpenFiles((prev) =>
          prev.map((f) =>
            f.path === activePath ? { ...f, originalContent: value } : f
          )
        );
      } catch (e) {
        console.error('Error saving file:', e);
      }
    },
    [activePath]
  );

  const handleCloseTab = useCallback(
    (path: string) => {
      setOpenFiles((prev) => prev.filter((f) => f.path !== path));
      if (activePath === path) {
        const remaining = openFiles.filter((f) => f.path !== path);
        setActivePath(remaining.length > 0 ? remaining[remaining.length - 1].path : null);
      }
    },
    [activePath, openFiles]
  );

  const handleExecute = useCallback(async () => {
    if (!activeFile) return;

    setIsExecuting(true);
    setOutput(null);

    try {
      const execLang = getExecLanguage(activeFile.language);
      const result = await executeCode(activeFile.content, execLang);
      setOutput({
        stdout: result.stdout,
        stderr: result.stderr,
        exitCode: result.exit_code,
        timeMs: result.execution_time_ms,
      });
    } catch (e) {
      setOutput({
        stdout: '',
        stderr: e instanceof Error ? e.message : 'Execution failed',
        exitCode: 1,
        timeMs: 0,
      });
    } finally {
      setIsExecuting(false);
    }
  }, [activeFile]);

  const hasChanges = activeFile && activeFile.content !== activeFile.originalContent;

  return (
    <div className="flex h-full bg-dark-900">
      {/* Sidebar: File Tree */}
      {sidebarOpen && (
        <div className="w-64 border-r border-gray-700 flex flex-col bg-dark-850">
          <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700">
            <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
              Files
            </span>
            <button
              onClick={() => setRefreshKey((k) => k + 1)}
              className="text-gray-500 hover:text-white text-xs"
            >
              ↻
            </button>
          </div>
          <div className="flex-1 overflow-y-auto">
            <FileTree onFileSelect={handleFileSelect} refreshKey={refreshKey} />
          </div>
        </div>
      )}

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-1.5 bg-dark-800 border-b border-gray-700">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="p-1 text-gray-400 hover:text-white"
            title="Toggle sidebar"
          >
            ☰
          </button>

          {activeFile && (
            <>
              <div className="flex items-center gap-1 ml-2">
                <span className="text-sm">{getFileIcon(activeFile.language)}</span>
                <span className="text-sm text-gray-300">{activePath?.split('/').pop()}</span>
                {hasChanges && <span className="text-yellow-400 text-xs">●</span>}
              </div>

              <div className="ml-auto flex items-center gap-2">
                <button
                  onClick={() => handleSave(activeFile.content)}
                  disabled={!hasChanges}
                  className={`px-3 py-1 text-xs rounded ${
                    hasChanges
                      ? 'bg-blue-600 text-white hover:bg-blue-500'
                      : 'bg-gray-700 text-gray-500 cursor-not-allowed'
                  }`}
                >
                  Save (Ctrl+S)
                </button>

                <button
                  onClick={handleExecute}
                  disabled={isExecuting}
                  className={`px-3 py-1 text-xs rounded ${
                    isExecuting
                      ? 'bg-gray-700 text-gray-500 cursor-not-allowed'
                      : 'bg-green-600 text-white hover:bg-green-500'
                  }`}
                >
                  {isExecuting ? '⏳ Running...' : '▶ Run'}
                </button>
              </div>
            </>
          )}
        </div>

        {/* Tabs */}
        {openFiles.length > 0 && (
          <div className="flex bg-dark-800 border-b border-gray-700 overflow-x-auto">
            {openFiles.map((f) => (
              <div
                key={f.path}
                className={`flex items-center gap-1 px-3 py-1.5 text-sm cursor-pointer border-r border-gray-700 whitespace-nowrap ${
                  f.path === activePath
                    ? 'bg-dark-900 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-dark-700'
                }`}
                onClick={() => setActivePath(f.path)}
              >
                <span>{getFileIcon(f.language)}</span>
                <span>{f.path.split('/').pop()}</span>
                {f.content !== f.originalContent && (
                  <span className="text-yellow-400 text-xs">●</span>
                )}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleCloseTab(f.path);
                  }}
                  className="ml-1 text-gray-500 hover:text-white text-xs"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Editor or empty state */}
        <div className="flex-1 min-h-0 flex">
          <div className="flex-1 min-h-0">
            {activeFile ? (
              <MonacoEditor
                value={activeFile.content}
                language={activeFile.language}
                onChange={handleContentChange}
                onSave={handleSave}
              />
            ) : (
              <div className="flex items-center justify-center h-full text-gray-500">
                <div className="text-center">
                  <div className="text-4xl mb-4">📝</div>
                  <div className="text-lg">Select a file to edit</div>
                  <div className="text-sm mt-2">
                    Or press <kbd className="px-2 py-0.5 bg-dark-700 rounded text-xs">Ctrl+N</kbd> to
                    create a new file
                  </div>
                </div>
              </div>
            )}
          </div>
          {activeFile && (
            <div className="w-72 border-l border-gray-700 overflow-y-auto p-2">
              <SuggestionsPanel
                code={activeFile.content}
                filePath={activeFile.path}
                onApplyFix={(s) => {
                  const newContent = activeFile.content.replace(
                    `// TODO: ${s.description}`,
                    ''
                  );
                  handleContentChange(newContent);
                }}
              />
            </div>
          )}
        </div>

        {/* Output / Terminal */}
        {output && (
          <div className="h-52 border-t border-gray-700 flex flex-col">
            <div className="flex items-center justify-between px-3 py-1 bg-dark-800 border-b border-gray-700">
              <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                Output
              </span>
              <div className="flex items-center gap-2">
                <span
                  className={`text-xs px-1.5 py-0.5 rounded ${
                    output.exitCode === 0
                      ? 'bg-green-900 text-green-300'
                      : 'bg-red-900 text-red-300'
                  }`}
                >
                  Exit: {output.exitCode}
                </span>
                <span className="text-xs text-gray-500">{output.timeMs}ms</span>
                <button
                  onClick={() => setOutput(null)}
                  className="text-gray-500 hover:text-white text-xs"
                >
                  ×
                </button>
              </div>
            </div>
            <div className="flex-1 min-h-0">
              <Terminal
                output={
                  output.stdout
                    ? output.stderr
                      ? `${output.stdout}\n\n${output.stderr}`
                      : output.stdout
                    : output.stderr || 'No output'
                }
                isRunning={false}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function getFileIcon(language: string): string {
  switch (language) {
    case 'rust':
      return '🦀';
    case 'python':
      return '🐍';
    case 'javascript':
      return '📜';
    case 'typescript':
      return '📘';
    case 'json':
      return '📋';
    case 'markdown':
      return '📝';
    default:
      return '📄';
  }
}
