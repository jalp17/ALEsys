import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AppShell } from './layouts/AppShell';
import { Chat } from './pages/Chat';
import { Generate } from './pages/Generate';
import { Sessions } from './pages/Sessions';
import { Settings } from './pages/Settings';
import { GraphViewer } from './pages/graph';
import { AdvancedSearch } from './pages/search';
import { EditorPage } from './pages/editor';
import { Agents } from './pages/Agents';
import Plugins from './pages/Plugins';
import OrchestratorDashboard from './pages/Orchestrator';
import Collaboration from './pages/Collaboration';
import LearningPage from './pages/Learning';
import DebugPage from './pages/Debug';
import TestGenerationPage from './pages/TestGeneration';
import RefactoringPage from './pages/Refactoring';
import KnowledgeCurationPage from './pages/KnowledgeCuration';
import MultiAgentPage from './pages/MultiAgent';
import AnalyticsPage from './pages/Analytics';
import WorkflowsPage from './pages/Workflows';
import AdvancedSearchPage from './pages/AdvancedSearch';
import { IngestionPanel, BatchIngestion, IngestionHistory, MarkdownIngestionPanel } from './pages/ingestion';
import ResearchLayout from './layouts/ResearchLayout/ResearchLayout';
import WorkspaceLayout from './layouts/WorkspaceLayout/WorkspaceLayout';

const queryClient = new QueryClient();

const API_BASE = import.meta.env.VITE_API_URL || '';

// Auto-login with secure credentials from build-time env
async function ensureAuthToken() {
  try {
    const existing = localStorage.getItem('alesys_token');
    if (existing) return;
    const resp = await fetch(`${API_BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        username: 'admin', 
        password: import.meta.env.VITE_ADMIN_PASSWORD || 'alesys_admin_2026_secure' 
      }),
    });
    if (resp.ok) {
      const data = await resp.json();
      if (data.token) localStorage.setItem('alesys_token', data.token);
    }
  } catch { /* noop */ }
}
ensureAuthToken();

/** Padded wrapper for standard content pages */
function Padded({ children }: { children: React.ReactNode }) {
  return <div className="mx-auto h-full max-w-6xl p-4 lg:p-6">{children}</div>
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<Navigate to="/workspaces" replace />} />
            <Route path="/chat" element={<Padded><Chat /></Padded>} />

            {/* Investigación */}
            <Route path="/workspaces" element={<WorkspaceLayout />} />
            <Route path="/workspaces/:wsId" element={<WorkspaceLayout />} />
            <Route path="/ingestion" element={<Padded><IngestionPanel /></Padded>} />
            <Route path="/ingestion/batch" element={<Padded><BatchIngestion /></Padded>} />
            <Route path="/ingestion/history" element={<Padded><IngestionHistory /></Padded>} />
            <Route path="/ingestion/markdown" element={<Padded><MarkdownIngestionPanel /></Padded>} />
            <Route path="/research" element={<ResearchLayout />} />
            <Route path="/research/:projectId" element={<ResearchLayout />} />
            <Route path="/sessions" element={<Padded><Sessions /></Padded>} />

            {/* Desarrollo */}
            <Route path="/editor" element={<EditorPage />} />
            <Route path="/generate" element={<Padded><Generate /></Padded>} />
            <Route path="/test-generation" element={<Padded><TestGenerationPage /></Padded>} />
            <Route path="/debug" element={<Padded><DebugPage /></Padded>} />
            <Route path="/refactoring" element={<Padded><RefactoringPage /></Padded>} />
            <Route path="/agents" element={<Padded><Agents /></Padded>} />
            <Route path="/orchestrator" element={<Padded><OrchestratorDashboard /></Padded>} />
            <Route path="/collaboration" element={<Padded><Collaboration /></Padded>} />

            {/* Conocimiento */}
            <Route path="/graph" element={<GraphViewer />} />
            <Route path="/search" element={<Padded><AdvancedSearch /></Padded>} />
            <Route path="/advanced-search" element={<Padded><AdvancedSearchPage /></Padded>} />
            <Route path="/kb-curation" element={<Padded><KnowledgeCurationPage /></Padded>} />
            <Route path="/analytics" element={<Padded><AnalyticsPage /></Padded>} />
            <Route path="/workflows" element={<Padded><WorkflowsPage /></Padded>} />

            {/* Sistema */}
            <Route path="/plugins" element={<Padded><Plugins /></Padded>} />
            <Route path="/learning" element={<Padded><LearningPage /></Padded>} />
            <Route path="/multi-agent" element={<Padded><MultiAgentPage /></Padded>} />
            <Route path="/settings" element={<Padded><Settings /></Padded>} />
          </Routes>
        </AppShell>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;