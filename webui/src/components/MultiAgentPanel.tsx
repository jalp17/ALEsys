import { useState, useEffect } from 'react';

interface Agent {
  id: string;
  name: string;
  status: string;
  capabilities: string[];
}

interface Task {
  id: string;
  title: string;
  status: string;
  priority: string;
}

interface ConsensusResult {
  proposal_id: string;
  passed: boolean;
  approval_rate: number;
  consensus_reached: boolean;
  final_decision: string;
  weighted_score: number;
  votes_count?: number;
}

const collabService = {
  async getStatus(): Promise<{ total_agents: number; idle_agents: number; busy_agents: number }> {
    const res = await fetch('/api/v1/collab/status');
    return res.json();
  },

  async getTasks(): Promise<{ tasks: Task[]; total: number; done: number; in_progress: number; pending: number }> {
    const res = await fetch('/api/v1/collab/tasks');
    return res.json();
  },

  async createTask(task: { id: string; title: string; description: string; priority: string }): Promise<Task> {
    const res = await fetch('/api/v1/collab/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(task),
    });
    return res.json();
  },

  async runConsensus(proposal: { proposal_id: string; votes: { agent_id: string; vote: string; confidence: number }[] }): Promise<ConsensusResult> {
    const res = await fetch('/api/v1/collab/consensus', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(proposal),
    });
    return res.json();
  },
};

export function MultiAgentPanel() {
  const [status, setStatus] = useState<{ total_agents: number; idle_agents: number; busy_agents: number } | null>(null);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskStats, setTaskStats] = useState({ total: 0, done: 0, in_progress: 0, pending: 0 });
  const [newTaskTitle, setNewTaskTitle] = useState('');
  const [newTaskPriority, setNewTaskPriority] = useState('medium');
  const [consensusResult, setConsensusResult] = useState<ConsensusResult | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(false);

  const loadAgents = async () => {
    try {
      const res = await fetch('/api/v1/agents');
      const data = await res.json();
      setAgents(
        (data.agents || []).map((a: any) => ({
          id: a.id,
          name: a.name,
          status: a.status,
          capabilities: [],
        }))
      );
    } catch (e) { console.error(e); }
  };

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [s, t] = await Promise.all([collabService.getStatus(), collabService.getTasks()]);
      setStatus(s);
      setTasks(t.tasks);
      setTaskStats({ total: t.total, done: t.done, in_progress: t.in_progress, pending: t.pending });
      await loadAgents();
    } catch (err) { console.error(err); }
  };

  const handleCreateTask = async () => {
    if (!newTaskTitle.trim()) return;
    setLoading(true);
    try {
      await collabService.createTask({
        id: `task-${Date.now()}`,
        title: newTaskTitle,
        description: '',
        priority: newTaskPriority,
      });
      setNewTaskTitle('');
      await loadData();
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleConsensus = async () => {
    setLoading(true);
    try {
      const result = await collabService.runConsensus({
        proposal_id: `proposal-${Date.now()}`,
        votes: [
          { agent_id: 'agent-1', vote: 'approve', confidence: 0.9 },
          { agent_id: 'agent-2', vote: 'approve', confidence: 0.7 },
          { agent_id: 'agent-3', vote: 'reject', confidence: 0.6 },
        ],
      });
      setConsensusResult(result);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Agent Status</h3>
        <div className="grid grid-cols-3 gap-4 text-center">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-blue-400">{status?.total_agents ?? 0}</div>
            <div className="text-gray-400 text-sm">Total</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-green-400">{status?.idle_agents ?? 0}</div>
            <div className="text-gray-400 text-sm">Idle</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-yellow-400">{status?.busy_agents ?? 0}</div>
            <div className="text-gray-400 text-sm">Busy</div>
          </div>
        </div>
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Agents</h3>
        {agents.length === 0 ? (
          <p className="text-sm text-gray-400">No agents loaded.</p>
        ) : (
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {agents.map((agent) => (
              <div key={agent.id} className="flex items-center justify-between p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span className="font-medium">{agent.name}</span>
                <span className={`text-xs px-2 py-1 rounded ${
                  agent.status === 'Connected' ? 'bg-green-900 text-green-300' :
                  agent.status === 'Busy' ? 'bg-yellow-900 text-yellow-300' :
                  agent.status === 'Idle' ? 'bg-blue-900 text-blue-300' :
                  'bg-gray-700 text-gray-300'
                }`}>{agent.status}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Task Board</h3>
        <div className="grid grid-cols-4 gap-2 mb-3 text-center text-sm">
          <div className="p-2 rounded bg-dark-900 border border-gray-700">
            <div className="font-bold">{taskStats.total}</div>
            <div className="text-gray-400">Total</div>
          </div>
          <div className="p-2 rounded bg-dark-900 border border-gray-700">
            <div className="font-bold text-gray-300">{taskStats.pending}</div>
            <div className="text-gray-400">Pending</div>
          </div>
          <div className="p-2 rounded bg-dark-900 border border-gray-700">
            <div className="font-bold text-yellow-400">{taskStats.in_progress}</div>
            <div className="text-gray-400">In Progress</div>
          </div>
          <div className="p-2 rounded bg-dark-900 border border-gray-700">
            <div className="font-bold text-green-400">{taskStats.done}</div>
            <div className="text-gray-400">Done</div>
          </div>
        </div>

        <div className="flex gap-2 mb-3">
          <input
            value={newTaskTitle}
            onChange={(e) => setNewTaskTitle(e.target.value)}
            placeholder="New task title..."
            className="flex-1 bg-dark-900 border border-gray-700 rounded p-2 text-sm"
          />
          <select
            value={newTaskPriority}
            onChange={(e) => setNewTaskPriority(e.target.value)}
            className="bg-dark-900 border border-gray-700 rounded px-2 text-sm"
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
          <button
            onClick={handleCreateTask}
            disabled={loading}
            className="px-3 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50"
          >
            Add
          </button>
        </div>

        {tasks.length > 0 && (
          <div className="space-y-1">
            {tasks.map((task) => (
              <div key={task.id} className="flex items-center justify-between p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span>{task.title}</span>
                <span className="text-xs text-gray-400">{task.status}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Consensus Engine</h3>
        <button
          onClick={handleConsensus}
          disabled={loading}
          className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50 mb-3"
        >
          {loading ? 'Evaluating...' : 'Run Consensus (3 agents)'}
        </button>

        {consensusResult && (
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="flex items-center gap-3 mb-2">
              <span className={`text-lg font-bold ${consensusResult.passed ? 'text-green-400' : 'text-red-400'}`}>
                {consensusResult.passed ? 'APPROVED' : 'REJECTED'}
              </span>
              <span className="text-sm text-gray-400">{(consensusResult.approval_rate * 100).toFixed(0)}% approval</span>
            </div>
            <div className="text-sm text-gray-400">{consensusResult.final_decision}</div>
            <div className="text-xs text-gray-500 mt-1">
              Weighted score: {consensusResult.weighted_score.toFixed(2)} | Votes: {consensusResult.votes_count}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
