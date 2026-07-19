import { MultiAgentPanel } from '../components/MultiAgentPanel';

export default function MultiAgentPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Multi-Agent Collaboration</h1>
        <p className="text-gray-400 text-sm mt-1">
          Coordinate multiple AI agents working together on complex tasks with shared task boards and consensus.
        </p>
      </div>

      <MultiAgentPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">How Multi-Agent Works</h3>
        <div className="text-sm text-gray-300 space-y-2">
          <p>1. <strong>Register agents:</strong> Each agent has capabilities (code, test, review, etc.)</p>
          <p>2. <strong>Create tasks:</strong> Tasks have priorities and dependencies</p>
          <p>3. <strong>Coordinate:</strong> The coordinator assigns agents based on capabilities</p>
          <p>4. <strong>Consensus:</strong> For critical decisions, agents vote and reach consensus</p>
        </div>
      </div>
    </div>
  );
}
