import { WorkflowPanel } from '../components/WorkflowPanel';

export default function WorkflowsPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Workflow Automation</h1>
        <p className="text-gray-400 text-sm mt-1">
          Create, manage, and run automated workflows with multi-step actions and triggers.
        </p>
      </div>

      <WorkflowPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Workflow Features</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Multi-Step</div>
            <div className="text-gray-400 mt-1">Chain actions together with dependencies</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-green-400">Triggers</div>
            <div className="text-gray-400 mt-1">Manual, cron, webhook, or event-based</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-yellow-400">Actions</div>
            <div className="text-gray-400 mt-1">Run commands, call APIs, send notifications</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-purple-400">Logging</div>
            <div className="text-gray-400 mt-1">Track execution with detailed logs</div>
          </div>
        </div>
      </div>
    </div>
  );
}
