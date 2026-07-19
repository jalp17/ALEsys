import { useQuery } from '@tanstack/react-query';
import { fetchAgentStats } from '../services/agentService';

export function AgentStatusIndicator() {
  const { data: stats } = useQuery({
    queryKey: ['agents-stats'],
    queryFn: fetchAgentStats,
    refetchInterval: 10000,
  });

  if (!stats) return null;

  return (
    <div className="flex items-center gap-1.5 text-xs text-gray-400" title={`${stats.connected} agentes conectados`}>
      <div className={`w-1.5 h-1.5 rounded-full ${stats.connected > 0 ? 'bg-green-500' : 'bg-gray-600'}`} />
      <span>Agents</span>
    </div>
  );
}
