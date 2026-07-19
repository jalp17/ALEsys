import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import {
  fetchAgents,
  fetchAgentStats,
  executeOnAgent,
  AgentInfo,
} from '../services/agentService';

export function Agents() {
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [cmdInput, setCmdInput] = useState('');
  const [cmdOutput, setCmdOutput] = useState('');

  const { data: agents = [], isLoading } = useQuery({
    queryKey: ['agents'],
    queryFn: fetchAgents,
    refetchInterval: 5000,
  });

  const { data: stats } = useQuery({
    queryKey: ['agents-stats'],
    queryFn: fetchAgentStats,
    refetchInterval: 5000,
  });

  const execMutation = useMutation({
    mutationFn: ({ agentId, command }: { agentId: string; command: string }) => {
      const parts = command.split(/\s+/);
      return executeOnAgent(agentId, parts[0], parts.slice(1));
    },
  });

  const handleExecute = () => {
    if (!selectedAgent || !cmdInput.trim()) return;
    execMutation.mutate(
      { agentId: selectedAgent, command: cmdInput },
      {
        onSuccess: (result) => {
          setCmdOutput(
            `Exit: ${result.exit_code}\n\n${result.stdout}${
              result.stderr ? `\n\nSTDERR:\n${result.stderr}` : ''
            }`
          );
        },
        onError: (err: Error) => {
          setCmdOutput(`Error: ${err.message}`);
        },
      }
    );
  };

  return (
    <div className="h-full flex">
      {/* Sidebar - Agent List */}
      <div className="w-64 bg-dark-800 border-r border-gray-700 flex flex-col">
        <div className="p-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold">Agentes Remotos</h2>
          {stats && (
            <p className="text-xs text-gray-400 mt-1">
              {stats.connected} de {stats.total} conectados
            </p>
          )}
        </div>

        <div className="flex-1 overflow-auto p-2 space-y-1">
          {isLoading && (
            <p className="text-sm text-gray-500 p-2">Cargando agentes...</p>
          )}
          {agents.length === 0 && !isLoading && (
            <p className="text-sm text-gray-500 p-2">
              No hay agentes conectados.
            </p>
          )}
          {agents.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              selected={selectedAgent === agent.id}
              onSelect={() => setSelectedAgent(agent.id)}
            />
          ))}
        </div>
      </div>

      {/* Main - Terminal */}
      <div className="flex-1 flex flex-col">
        {selectedAgent ? (
          <>
            <div className="p-3 bg-dark-800 border-b border-gray-700 flex items-center gap-3">
              <span className="text-sm text-gray-300">
                Ejecutar en:{' '}
                <span className="font-mono text-primary-400">
                  {agents.find((a) => a.id === selectedAgent)?.name}
                </span>
              </span>
            </div>

            <div className="flex-1 p-4 overflow-auto">
              <pre className="font-mono text-sm text-green-400 whitespace-pre-wrap bg-dark-900 p-4 rounded h-full overflow-auto border border-gray-800">
                {cmdOutput || 'Escribe un comando y presiona Enter.'}
              </pre>
            </div>

            <div className="p-3 bg-dark-800 border-t border-gray-700">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={cmdInput}
                  onChange={(e) => setCmdInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleExecute()}
                  placeholder="Comando (ej: ls -la)"
                  className="flex-1 px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white font-mono text-sm"
                />
                <button
                  onClick={handleExecute}
                  disabled={execMutation.isPending}
                  className="px-4 py-2 bg-primary-600 rounded hover:bg-primary-700 transition text-sm font-semibold disabled:opacity-50"
                >
                  {execMutation.isPending ? 'Ejecutando...' : 'Ejecutar'}
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            Selecciona un agente para ejecutar comandos.
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Sub-components
// =============================================================================

function AgentCard({
  agent,
  selected,
  onSelect,
}: {
  agent: AgentInfo;
  selected: boolean;
  onSelect: () => void;
}) {
  const statusColor: Record<string, string> = {
    Connected: 'bg-green-500',
    Idle: 'bg-yellow-500',
    Busy: 'bg-orange-500',
    Disconnected: 'bg-red-500',
  };

  return (
    <button
      onClick={onSelect}
      className={`w-full text-left p-3 rounded transition ${
        selected
          ? 'bg-primary-600/20 border border-primary-600'
          : 'bg-dark-700 hover:bg-dark-600 border border-transparent'
      }`}
    >
      <div className="flex items-center gap-2">
        <div
          className={`w-2 h-2 rounded-full ${
            statusColor[agent.status] || 'bg-gray-500'
          }`}
        />
        <span className="text-sm font-medium text-white">{agent.name}</span>
      </div>
      <div className="text-xs text-gray-400 mt-1 font-mono">
        {agent.os}/{agent.arch}
      </div>
    </button>
  );
}
