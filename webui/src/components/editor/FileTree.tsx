import { useState, useEffect, useCallback } from 'react';
import type { FileTreeEntry } from '../../pages/editor/editorService';
import { listFiles } from '../../pages/editor/editorService';

interface FileTreeProps {
  onFileSelect: (path: string) => void;
  refreshKey?: number;
}

export function FileTree({ onFileSelect, refreshKey }: FileTreeProps) {
  const [entries, setEntries] = useState<FileTreeEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());

  const loadFiles = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listFiles('');
      setEntries(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Error loading files');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadFiles();
  }, [loadFiles, refreshKey]);

  const toggleDir = (path: string) => {
    setExpandedDirs((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  if (loading) {
    return (
      <div className="p-3 text-gray-500 text-sm">
        <div className="animate-pulse">Loading files...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-3 text-red-400 text-sm">
        <p>{error}</p>
        <button onClick={loadFiles} className="mt-2 text-xs text-gray-400 hover:text-white">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="py-2 text-sm">
      {entries.length === 0 ? (
        <div className="px-3 text-gray-500">No files found</div>
      ) : (
        entries.map((entry) => (
          <FileTreeItem
            key={entry.path}
            entry={entry}
            depth={0}
            expandedDirs={expandedDirs}
            onToggleDir={toggleDir}
            onFileSelect={onFileSelect}
          />
        ))
      )}
    </div>
  );
}

// =============================================================================
// FileTreeItem (recursive)
// =============================================================================

interface FileTreeItemProps {
  entry: FileTreeEntry;
  depth: number;
  expandedDirs: Set<string>;
  onToggleDir: (path: string) => void;
  onFileSelect: (path: string) => void;
}

function FileTreeItem({
  entry,
  depth,
  expandedDirs,
  onToggleDir,
  onFileSelect,
}: FileTreeItemProps) {
  const isExpanded = expandedDirs.has(entry.path);
  const paddingLeft = `${depth * 12 + 8}px`;

  const handleClick = () => {
    if (entry.is_dir) {
      onToggleDir(entry.path);
    } else {
      onFileSelect(entry.path);
    }
  };

  const getFileIcon = (name: string, isDir: boolean) => {
    if (isDir) {
      return isExpanded ? '📂' : '📁';
    }
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'rs':
        return '🦀';
      case 'py':
        return '🐍';
      case 'js':
      case 'jsx':
        return '📜';
      case 'ts':
      case 'tsx':
        return '📘';
      case 'json':
        return '📋';
      case 'md':
        return '📝';
      case 'toml':
      case 'yaml':
      case 'yml':
        return '⚙️';
      case 'sql':
        return '🗃️';
      default:
        return '📄';
    }
  };

  return (
    <>
      <div
        className="flex items-center py-1 px-2 hover:bg-dark-700 cursor-pointer group"
        style={{ paddingLeft }}
        onClick={handleClick}
      >
        {entry.is_dir && (
          <span className="mr-1 text-xs text-gray-500 w-3">
            {isExpanded ? '▼' : '▶'}
          </span>
        )}
        {!entry.is_dir && <span className="mr-1 w-3" />}
        <span className="mr-2 text-sm">{getFileIcon(entry.name, entry.is_dir)}</span>
        <span className="truncate text-gray-300 group-hover:text-white">{entry.name}</span>
        {!entry.is_dir && (
          <span className="ml-auto text-xs text-gray-600 hidden group-hover:block">
            {formatSize(entry.size)}
          </span>
        )}
      </div>
      {entry.is_dir && isExpanded && entry.children && (
        <div>
          {entry.children.map((child) => (
            <FileTreeItem
              key={child.path}
              entry={child}
              depth={depth + 1}
              expandedDirs={expandedDirs}
              onToggleDir={onToggleDir}
              onFileSelect={onFileSelect}
            />
          ))}
        </div>
      )}
    </>
  );
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}
