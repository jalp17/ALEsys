import { DebugAssistant } from '../components/DebugAssistantPanel';

export default function DebugPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Debug Assistant</h1>
        <p className="text-gray-400 text-sm mt-1">
          Paste your logs and get AI-powered analysis with actionable suggestions.
        </p>
      </div>

      <DebugAssistant />
    </div>
  );
}
