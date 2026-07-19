import { LearningPanel } from '../components/LearningPanel';

export default function LearningPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Learning</h1>
        <p className="text-gray-400 text-sm mt-1">
          Track how the AI learns from your feedback and improves suggestions over time.
        </p>
      </div>

      <LearningPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">How It Works</h3>
        <div className="text-sm text-gray-300 space-y-2">
          <p>1. <strong>Rate suggestions:</strong> When you see a suggestion in the editor, click 👍 or 👎 to rate it.</p>
          <p>2. <strong>Collect feedback:</strong> The system records your preferences per suggestion type.</p>
          <p>3. <strong>Generate insights:</strong> After enough data, the system learns which suggestions you find helpful.</p>
          <p>4. <strong>Improve suggestions:</strong> Future suggestions are weighted based on your feedback history.</p>
        </div>
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Data Sources</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Feedback History</div>
            <div className="text-gray-400 mt-1">Your ratings on suggestions (helpful/neutral/unhelpful)</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Context Memory</div>
            <div className="text-gray-400 mt-1">File patterns, languages, project structure</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Suggestion Scores</div>
            <div className="text-gray-400 mt-1">Internal scoring per suggestion type</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Session History</div>
            <div className="text-gray-400 mt-1">Past interactions and decisions</div>
          </div>
        </div>
      </div>
    </div>
  );
}
