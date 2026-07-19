import { AnalyticsPanel } from '../components/AnalyticsPanel';

export default function AnalyticsPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Advanced Analytics</h1>
        <p className="text-gray-400 text-sm mt-1">
          System usage, performance metrics, and user behavior analytics.
        </p>
      </div>

      <AnalyticsPanel />
    </div>
  );
}
