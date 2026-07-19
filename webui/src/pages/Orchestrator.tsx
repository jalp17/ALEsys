import { useState, useEffect } from 'react';

interface OrchestratorTask {
  id: string;
  description: string;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'PartiallyCompleted';
  subtasks: Subtask[];
}

interface Subtask {
  id: string;
  agent_type: string;
  command: string;
  args: string[];
  timeout_secs: number;
  retries: number;
  depends_on: string[];
}

interface PoolStats {
  total: number;
  idle: number;
  busy: number;
  total_completed: number;
}

interface CompletedTask {
  subtask_id: string;
  success: boolean;
  output: string | null;
  error: string | null;
  duration_ms: number;
}

const orchestratorService = {
  async submitTask(description: string): Promise<string> {
    const res = await fetch('/api/v1/orchestrator/submit', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ description }),
    });
    const data = await res.json();
    return data.task_id;
  },

  async getPoolStats(): Promise<PoolStats> {
    const res = await fetch('/api/v1/orchestrator/pool/stats');
    return res.json();
  },

  async getActiveTasks(): Promise<OrchestratorTask[]> {
    const res = await fetch('/api/v1/orchestrator/tasks');
    const data = await res.json();
    return data.tasks || [];
  },

  async getCompletedTasks(): Promise<CompletedTask[]> {
    const res = await fetch('/api/v1/orchestrator/completed');
    const data = await res.json();
    return data.tasks || [];
  },

  async registerAgent(id: string, agentType: string): Promise<void> {
    await fetch('/api/v1/orchestrator/pool/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, agent_type: agentType }),
    });
  },
};

export default function OrchestratorDashboard() {
  const [tasks, setTasks] = useState<OrchestratorTask[]>([]);
  const [completed, setCompleted] = useState<CompletedTask[]>([]);
  const [stats, setStats] = useState<PoolStats | null>(null);
  const [newTask, setNewTask] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const [activeTasks, completedTasks, poolStats] = await Promise.all([
        orchestratorService.getActiveTasks(),
        orchestratorService.getCompletedTasks(),
        orchestratorService.getPoolStats(),
      ]);
      setTasks(activeTasks);
      setCompleted(completedTasks);
      setStats(poolStats);
    } catch (err) {
      console.error('Failed to load orchestrator data:', err);
    }
    setLoading(false);
  };

  const handleSubmitTask = async () => {
    if (!newTask.trim()) return;
    try {
      await orchestratorService.submitTask(newTask);
      setNewTask('');
      loadData();
    } catch (err) {
      console.error('Failed to submit task:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading orchestrator...</div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Multi-Agent Orchestrator</h1>

      {/* Pool Stats */}
      {stats && (
        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="border rounded-lg p-4">
            <div className="text-sm text-gray-500">Total Agents</div>
            <div className="text-2xl font-bold">{stats.total}</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-gray-500">Idle</div>
            <div className="text-2xl font-bold text-green-600">{stats.idle}</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-gray-500">Busy</div>
            <div className="text-2xl font-bold text-yellow-600">{stats.busy}</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-gray-500">Completed</div>
            <div className="text-2xl font-bold text-blue-600">{stats.total_completed}</div>
          </div>
        </div>
      )}

      {/* Submit Task */}
      <div className="border rounded-lg p-4 mb-6">
        <h2 className="text-lg font-semibold mb-3">Submit New Task</h2>
        <div className="flex gap-2">
          <input
            type="text"
            value={newTask}
            onChange={(e) => setNewTask(e.target.value)}
            placeholder="Describe a complex task (e.g., 'Implement and test a new API endpoint')"
            className="flex-1 p-2 border rounded"
            onKeyDown={(e) => e.key === 'Enter' && handleSubmitTask()}
          />
          <button
            onClick={handleSubmitTask}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Submit
          </button>
        </div>
      </div>

      {/* Active Tasks */}
      <div className="border rounded-lg p-4 mb-6">
        <h2 className="text-lg font-semibold mb-3">Active Tasks ({tasks.length})</h2>
        {tasks.length === 0 ? (
          <p className="text-gray-500">No active tasks</p>
        ) : (
          <div className="space-y-3">
            {tasks.map((task) => (
              <div key={task.id} className="border rounded p-3">
                <div className="flex items-center justify-between">
                  <span className="font-medium">{task.description}</span>
                  <span
                    className={`px-2 py-1 rounded text-xs ${
                      task.status === 'Completed'
                        ? 'bg-green-100 text-green-800'
                        : task.status === 'Failed'
                        ? 'bg-red-100 text-red-800'
                        : task.status === 'Running'
                        ? 'bg-yellow-100 text-yellow-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {task.status}
                  </span>
                </div>
                <div className="mt-2 text-sm text-gray-600">
                  Subtasks: {task.subtasks.length}
                </div>
                <div className="mt-2 flex flex-wrap gap-1">
                  {task.subtasks.map((st) => (
                    <span
                      key={st.id}
                      className="px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded"
                    >
                      {st.agent_type}: {st.command}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Completed Tasks */}
      <div className="border rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-3">Completed Tasks ({completed.length})</h2>
        {completed.length === 0 ? (
          <p className="text-gray-500">No completed tasks yet</p>
        ) : (
          <div className="space-y-2">
            {completed.slice(0, 10).map((task) => (
              <div
                key={task.subtask_id}
                className={`flex items-center justify-between p-2 rounded ${
                  task.success ? 'bg-green-50' : 'bg-red-50'
                }`}
              >
                <span className="text-sm">
                  Task {task.subtask_id.slice(0, 8)}...
                </span>
                <div className="flex items-center gap-4 text-sm">
                  <span className={task.success ? 'text-green-600' : 'text-red-600'}>
                    {task.success ? 'Success' : 'Failed'}
                  </span>
                  <span className="text-gray-500">{task.duration_ms}ms</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
