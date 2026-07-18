import { useState } from 'react';
import type { DiffResult } from '../../pages/editor/editorService';

interface DiffViewerProps {
  diff: DiffResult;
  onApply?: (newContent: string) => void;
  onDiscard?: () => void;
}

export function DiffViewer({ diff, onApply, onDiscard }: DiffViewerProps) {
  const [showFull, setShowFull] = useState(false);

  const lines = diff.diff.split('\n');
  const displayLines = showFull ? lines : lines.slice(0, 50);

  return (
    <div className="bg-dark-850 border border-gray-700 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-dark-800 border-b border-gray-700">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-gray-300">Changes</span>
          <span className="text-xs text-green-400">+{diff.lines_added}</span>
          <span className="text-xs text-red-400">-{diff.lines_removed}</span>
        </div>
        <div className="flex items-center gap-2">
          {onDiscard && (
            <button
              onClick={onDiscard}
              className="px-3 py-1 text-xs text-gray-400 hover:text-white bg-dark-700 hover:bg-dark-600 rounded"
            >
              Discard
            </button>
          )}
          {onApply && (
            <button
              onClick={() => onApply(diff.new_content)}
              className="px-3 py-1 text-xs text-white bg-green-600 hover:bg-green-500 rounded"
            >
              Apply
            </button>
          )}
        </div>
      </div>

      {/* Diff content */}
      <div className="overflow-auto max-h-96 font-mono text-sm">
        {displayLines.map((line, i) => {
          const firstChar = line[0];
          let bgColor = 'bg-transparent';
          let textColor = 'text-gray-400';

          if (firstChar === '+') {
            bgColor = 'bg-green-900/30';
            textColor = 'text-green-400';
          } else if (firstChar === '-') {
            bgColor = 'bg-red-900/30';
            textColor = 'text-red-400';
          } else if (firstChar === ' ') {
            textColor = 'text-gray-500';
          }

          return (
            <div
              key={i}
              className={`${bgColor} ${textColor} px-4 py-0.5 hover:bg-dark-700`}
            >
              <span className="inline-block w-6 text-gray-600 select-none">{firstChar}</span>
              <span className="whitespace-pre-wrap">{line.slice(1)}</span>
            </div>
          );
        })}
      </div>

      {/* Show more/less */}
      {lines.length > 50 && (
        <div className="px-4 py-2 bg-dark-800 border-t border-gray-700">
          <button
            onClick={() => setShowFull(!showFull)}
            className="text-xs text-gray-400 hover:text-white"
          >
            {showFull ? 'Show less' : `Show all ${lines.length} lines`}
          </button>
        </div>
      )}
    </div>
  );
}
