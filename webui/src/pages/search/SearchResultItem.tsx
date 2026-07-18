/**
 * Individual search result with highlighting and metadata
 */

import { useState } from 'react';
import type { AdvancedSearchResult } from './searchService';

interface SearchResultItemProps {
  result: AdvancedSearchResult;
  index: number;
}

export function SearchResultItem({ result, index }: SearchResultItemProps) {
  const [expanded, setExpanded] = useState(false);

  // Determine source badge color
  const sourceColor = result.score_breakdown.vector > 0
    ? 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    : 'bg-purple-500/20 text-purple-400 border-purple-500/30';

  // Determine highest scoring source
  const dominantSource =
    result.score_breakdown.vector > result.score_breakdown.graph ? 'vector' : 'graph';

  return (
    <div className="bg-dark-700 rounded-lg border border-gray-600 hover:border-primary-500/50 transition">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-600/50">
        <div className="flex items-center gap-3">
          <span className="text-xs text-gray-400 font-mono">#{index + 1}</span>
          {result.path && (
            <span className="text-sm text-primary-400 font-mono truncate max-w-xs">
              {result.path}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <span className={`text-xs px-2 py-0.5 rounded border ${sourceColor}`}>
            {dominantSource}
          </span>
          <span className="text-xs text-gray-400">
            {(result.similarity * 100).toFixed(1)}%
          </span>
        </div>
      </div>

      {/* Content */}
      <div className="p-4">
        {result.highlighted ? (
          <div
            className="text-sm text-gray-300 leading-relaxed whitespace-pre-wrap"
            dangerouslySetInnerHTML={{ __html: result.highlighted }}
          />
        ) : (
          <p className="text-sm text-gray-300 leading-relaxed">
            {expanded ? result.content : result.content.slice(0, 300)}
            {result.content.length > 300 && !expanded && (
              <button
                onClick={() => setExpanded(true)}
                className="text-primary-400 hover:text-primary-300 ml-1"
              >
                ...ver más
              </button>
            )}
          </p>
        )}
      </div>

      {/* Footer - Score breakdown */}
      <div className="px-4 py-2 bg-dark-800/50 rounded-b-lg border-t border-gray-600/50">
        <div className="flex items-center gap-4 text-xs text-gray-500">
          <span>
            vector: {result.score_breakdown.vector.toFixed(3)}
          </span>
          <span>
            graph: {result.score_breakdown.graph.toFixed(3)}
          </span>
          <span>
            rrf: {result.score_breakdown.rrf.toFixed(3)}
          </span>
          <span className="text-gray-600">
            frag:{result.fragment_id} doc:{result.document_id}
          </span>
        </div>
      </div>
    </div>
  );
}
