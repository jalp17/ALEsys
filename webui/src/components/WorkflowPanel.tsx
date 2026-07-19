import { useState, useEffect } from 'react';

interface Workflow {
  id: string;
  title: string;
  status: string;
  steps: number;
}

interface WorkflowResult {
  workflow_id: string;
  success: boolean;
  logs: { step_id: string; step_name: string; success: boolean; output: string; duration_ms: number }[];
  total_duration_ms: number;
}

const workflowService = {
  async list(): Promise<{ workflows: Workflow[]; total: number }> {
    const res = await fetch('/api/v1/workflows');
    return res.json();
  },

  async create(wf: { id: string; name: string; description: string }): Promise<Workflow> {
    const res = await fetch('/api/v1/workflows', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(wf),
    });
    return res.json();
  },

  async run(id: string): Promise<WorkflowResult> {
    const res = await fetch(`/api/v1/workflows/${id}/run`, { method: 'POST' });
    return res.json();
  },
};

export function WorkflowPanel() {
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [name, setName] = useState('');
  const [result, setResult] = useState<WorkflowResult | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => { loadWorkflows(); }, []);

  const loadWorkflows = async () => {
    try {
      const data = await workflowService.list();
      setWorkflows(data.workflows);
    } catch (err) { console.error(err); }
  };

  const handleCreate = async () => {
    if (!name.trim()) return;
    setLoading(true);
    try {
      await workflowService.create({ id: `wf-${Date.now()}`, name, description: '' });
      setName('');
      await loadWorkflows();
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleRun = async (id: string) => {
    setLoading(true);
    try {
      const r = await workflowService.run(id);
      setResult(r);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Workflow Engine</h3>
        <div className="flex gap-2 mb-3">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Workflow name..."
            className="flex-1 bg-dark-900 border border-gray-700 rounded p-2 text-sm"
          />
          <button onClick={handleCreate} disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 text-sm">
            Create
          </button>
        </div>

        {workflows.length > 0 ? (
          <div className="space-y-2">
            {workflows.map((wf) => (
              <div key={wf.id} className="flex items-center justify-between p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span>{wf.title}</span>
                <button onClick={() => handleRun(wf.id)} disabled={loading}
                  className="text-xs px-2 py-1 bg-green-600 text-white rounded hover:bg-green-700">
                  Run
                </button>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-gray-500">No workflows yet. Create one above or run the demo.</div>
        )}

        <button onClick={() => handleRun('demo')} disabled={loading}
          className="mt-3 px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50 text-sm">
          {loading ? 'Running...' : 'Run Demo Workflow'}
        </button>
      </div>

      {result && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <div className="flex items-center gap-3 mb-3">
            <span className={`text-lg font-bold ${result.success ? 'text-green-400' : 'text-red-400'}`}>
              {result.success ? 'SUCCESS' : 'FAILED'}
            </span>
            <span className="text-sm text-gray-400">{result.total_duration_ms}ms</span>
          </div>
          <div className="space-y-1">
            {result.logs.map((log) => (
              <div key={log.step_id} className="flex items-center gap-2 p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span className={log.success ? 'text-green-400' : 'text-red-400'}>
                  {log.success ? '✓' : '✗'}
                </span>
                <span>{log.step_name}</span>
                <span className="text-gray-500 ml-auto">{log.duration_ms}ms</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
