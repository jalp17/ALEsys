import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { isDesktop } from './utils/platform';
import { DesktopLayout } from './layouts/DesktopLayout';
import { WebLayout } from './layouts/WebLayout';
import { Chat } from './pages/Chat';
import { Generate } from './pages/Generate';
import { Settings } from './pages/Settings';

const queryClient = new QueryClient();

function App() {
  const Layout = isDesktop ? DesktopLayout : WebLayout;

  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<Chat />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/generate" element={<Generate />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Layout>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;