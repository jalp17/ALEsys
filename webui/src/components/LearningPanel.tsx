import { useState, useEffect } from 'react';

interface Insight {
  insight_type: string;
  description: string;
  confidence: number;
  based_on_count: number;
}

interface FeedbackForm {
  suggestion_id: string;
  rating: 'Helpful' | 'Neutral' | 'Unhelpful';
  suggestion_type: string;
}

const learningService = {
  async submitFeedback(feedback: FeedbackForm): Promise<void> {
    await fetch('/api/v1/learning/feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(feedback),
    });
  },

  async getInsights(): Promise<Insight[]> {
    const res = await fetch('/api/v1/learning/insights');
    const data = await res.json();
    return data.insights || [];
  },
};

export function LearningPanel() {
  const [insights, setInsights] = useState<Insight[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadInsights();
  }, []);

  const loadInsights = async () => {
    try {
      const result = await learningService.getInsights();
      setInsights(result);
    } catch (err) {
      console.error('Failed to load insights:', err);
    }
    setLoading(false);
  };

  const confidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-green-400';
    if (confidence >= 0.5) return 'text-yellow-400';
    return 'text-red-400';
  };

  return (
    <div className="border rounded-lg bg-dark-800 p-4">
      <h3 className="font-semibold mb-3">Learning Insights</h3>
      {loading ? (
        <div className="text-gray-500 text-sm">Loading insights...</div>
      ) : insights.length === 0 ? (
        <div className="text-gray-500 text-sm">No insights yet. Provide feedback on suggestions to start learning.</div>
      ) : (
        <div className="space-y-3">
          {insights.map((insight, i) => (
            <div key={i} className="p-3 rounded border border-gray-700 bg-dark-900">
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-medium text-blue-400">{insight.insight_type}</span>
                <span className={`text-xs ${confidenceColor(insight.confidence)}`}>
                  {(insight.confidence * 100).toFixed(0)}% confidence
                </span>
              </div>
              <p className="text-sm text-gray-300">{insight.description}</p>
              <div className="text-xs text-gray-500 mt-1">
                Based on {insight.based_on_count} data points
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function FeedbackButtons({
  suggestionId,
  suggestionType,
  onFeedbackSubmitted,
}: {
  suggestionId: string;
  suggestionType: string;
  onFeedbackSubmitted?: () => void;
}) {
  const [submitted, setSubmitted] = useState(false);

  const handleFeedback = async (rating: 'Helpful' | 'Neutral' | 'Unhelpful') => {
    try {
      await learningService.submitFeedback({
        suggestion_id: suggestionId,
        rating,
        suggestion_type: suggestionType,
      });
      setSubmitted(true);
      onFeedbackSubmitted?.();
    } catch (err) {
      console.error('Failed to submit feedback:', err);
    }
  };

  if (submitted) {
    return <span className="text-xs text-green-400">Thanks for your feedback!</span>;
  }

  return (
    <div className="flex items-center gap-1">
      <button
        onClick={() => handleFeedback('Helpful')}
        className="text-xs px-2 py-0.5 rounded bg-green-900 text-green-300 hover:bg-green-800"
      >
        👍
      </button>
      <button
        onClick={() => handleFeedback('Neutral')}
        className="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-300 hover:bg-gray-600"
      >
        –
      </button>
      <button
        onClick={() => handleFeedback('Unhelpful')}
        className="text-xs px-2 py-0.5 rounded bg-red-900 text-red-300 hover:bg-red-800"
      >
        👎
      </button>
    </div>
  );
}
