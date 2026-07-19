import { RefactoringPanel } from '../components/RefactoringPanel';

export default function RefactoringPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Advanced Refactoring</h1>
        <p className="text-gray-400 text-sm mt-1">
          Analyze your code and get AI-powered refactoring suggestions with previews.
        </p>
      </div>

      <RefactoringPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">How It Works</h3>
        <div className="text-sm text-gray-300 space-y-2">
          <p>1. <strong>Paste code:</strong> Enter the code you want to analyze for refactoring.</p>
          <p>2. <strong>Analyze:</strong> The system detects functions, blocks, dependencies, and complexity.</p>
          <p>3. <strong>Review opportunities:</strong> See suggestions like Extract Function, Rename, Deduplicate.</p>
          <p>4. <strong>Preview:</strong> See a diff preview before applying any changes.</p>
        </div>
      </div>

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Refactoring Types</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Extract Function</div>
            <div className="text-gray-400 mt-1">Split large functions into smaller, reusable ones</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-green-400">Rename Symbol</div>
            <div className="text-gray-400 mt-1">Improve naming for clarity and consistency</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-yellow-400">Inline Function</div>
            <div className="text-gray-400 mt-1">Replace function calls with their implementation</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-purple-400">Simplify Conditional</div>
            <div className="text-gray-400 mt-1">Remove redundant conditions and simplify logic</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-red-400">Remove Dead Code</div>
            <div className="text-gray-400 mt-1">Clean up TODOs, FIXMEs, and unused code</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-orange-400">Deduplicate Code</div>
            <div className="text-gray-400 mt-1">Identify and merge duplicate code blocks</div>
          </div>
        </div>
      </div>
    </div>
  );
}
