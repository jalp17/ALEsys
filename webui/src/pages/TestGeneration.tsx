import { TestGeneratorPanel } from '../components/TestGeneratorPanel';

export default function TestGenerationPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Test Generation</h1>
        <p className="text-gray-400 text-sm mt-1">
          Automatically generate unit tests, edge cases, and integration tests for your functions.
        </p>
      </div>

      <TestGeneratorPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">How It Works</h3>
        <div className="text-sm text-gray-300 space-y-2">
          <p>1. <strong>Define function:</strong> Enter the function name, parameters, and return type.</p>
          <p>2. <strong>Select language:</strong> Choose between Rust, Python, TypeScript, or JavaScript.</p>
          <p>3. <strong>Generate:</strong> Click generate to create basic, edge case, and integration tests.</p>
          <p>4. <strong>Copy & use:</strong> Copy the generated tests to your test file.</p>
        </div>
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Test Types Generated</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-green-400">Unit Tests</div>
            <div className="text-gray-400 mt-1">Basic functionality with sample inputs</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-yellow-400">Edge Cases</div>
            <div className="text-gray-400 mt-1">Empty strings, zero values, optional params</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Integration</div>
            <div className="text-gray-400 mt-1">Tests with mocked dependencies</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-purple-400">Error Handling</div>
            <div className="text-gray-400 mt-1">Invalid inputs and error conditions</div>
          </div>
        </div>
      </div>
    </div>
  );
}
