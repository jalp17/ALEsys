import { useState, useEffect } from 'react';

interface UsageStats {
  total_events: number;
  unique_users: number;
  events_by_type: Record<string, number>;
}

interface PerformanceSummary {
  name: string;
  avg: number;
  min: number;
  max: number;
  count: number;
}

interface BehaviorPattern {
  name: string;
  frequency: number;
  description: string;
}

const analyticsService = {
  async getUsage(): Promise<UsageStats> {
    const res = await fetch('/api/v1/analytics/usage');
    return res.json();
  },

  async getPerformance(): Promise<{ summaries: PerformanceSummary[] }> {
    const res = await fetch('/api/v1/analytics/performance');
    return res.json();
  },

  async getUsers(): Promise<{ total_actions: number; unique_users: number; patterns: BehaviorPattern[] }> {
    const res = await fetch('/api/v1/analytics/users');
    return res.json();
  },
};

export function AnalyticsPanel() {
  const [usage, setUsage] = useState<UsageStats | null>(null);
  const [performance, setPerformance] = useState<PerformanceSummary[]>([]);
  const [userStats, setUserStats] = useState<{ total_actions: number; unique_users: number; patterns: BehaviorPattern[] } | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const [u, p, ub] = await Promise.all([
        analyticsService.getUsage(),
        analyticsService.getPerformance(),
        analyticsService.getUsers(),
      ]);
      setUsage(u);
      setPerformance(p.summaries);
      setUserStats(ub);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Usage Overview</h3>
        <div className="grid grid-cols-2 gap-4 text-center">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-blue-400">{usage?.total_events ?? 0}</div>
            <div className="text-gray-400 text-sm">Total Events</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-green-400">{usage?.unique_users ?? 0}</div>
            <div className="text-gray-400 text-sm">Unique Users</div>
          </div>
        </div>

        {usage && Object.keys(usage.events_by_type).length > 0 && (
          <div className="mt-3">
            <h4 className="text-sm font-medium mb-2">Events by Type</h4>
            <div className="space-y-1">
              {Object.entries(usage.events_by_type).map(([type, count]) => (
                <div key={type} className="flex items-center justify-between text-sm p-1">
                  <span className="text-gray-300">{type}</span>
                  <span className="text-gray-500">{count}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Performance Metrics</h3>
        {performance.length > 0 ? (
          <div className="space-y-2">
            {performance.map((m) => (
              <div key={m.name} className="flex items-center justify-between p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span className="text-gray-300">{m.name}</span>
                <div className="flex gap-4 text-xs">
                  <span className="text-gray-500">avg: {m.avg.toFixed(1)}</span>
                  <span className="text-green-400">min: {m.min.toFixed(1)}</span>
                  <span className="text-red-400">max: {m.max.toFixed(1)}</span>
                  <span className="text-gray-500">n={m.count}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-gray-500">No performance data yet</div>
        )}
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">User Behavior</h3>
        <div className="grid grid-cols-2 gap-4 text-center mb-3">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-yellow-400">{userStats?.total_actions ?? 0}</div>
            <div className="text-gray-400 text-sm">Total Actions</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="text-2xl font-bold text-purple-400">{userStats?.patterns?.length ?? 0}</div>
            <div className="text-gray-400 text-sm">Patterns</div>
          </div>
        </div>

        {userStats?.patterns && userStats.patterns.length > 0 && (
          <div className="space-y-1">
            {userStats.patterns.map((p) => (
              <div key={p.name} className="p-2 rounded bg-dark-900 border border-gray-700 text-sm">
                <span className="text-blue-400">{p.name}</span>
                <span className="text-gray-500 ml-2">({p.frequency}x)</span>
                <div className="text-xs text-gray-400 mt-1">{p.description}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      <button
        onClick={loadData}
        disabled={loading}
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
      >
        {loading ? 'Loading...' : 'Refresh Data'}
      </button>
    </div>
  );
}
