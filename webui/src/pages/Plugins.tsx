import { useState, useEffect } from 'react';

interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  permissions: string[];
  hooks: string[];
}

interface MarketplacePlugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  installed: boolean;
}

const pluginService = {
  async listPlugins(): Promise<Plugin[]> {
    const res = await fetch('/api/v1/plugins');
    const data = await res.json();
    return data.plugins || [];
  },

  async executePlugin(pluginId: string, command: string, args: string[] = []): Promise<any> {
    const res = await fetch(`/api/v1/plugins/${pluginId}/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command, args }),
    });
    return res.json();
  },

  async enablePlugin(pluginId: string): Promise<any> {
    const res = await fetch(`/api/v1/plugins/${pluginId}/enable`, {
      method: 'POST',
    });
    return res.json();
  },

  async disablePlugin(pluginId: string): Promise<any> {
    const res = await fetch(`/api/v1/plugins/${pluginId}/disable`, {
      method: 'POST',
    });
    return res.json();
  },

  async listMarketplace(): Promise<MarketplacePlugin[]> {
    const res = await fetch('/api/v1/marketplace/plugins');
    const data = await res.json();
    return data.plugins || [];
  },

  async installPlugin(pluginId: string): Promise<any> {
    const res = await fetch(`/api/v1/marketplace/install/${pluginId}`, {
      method: 'POST',
    });
    return res.json();
  },

  async uninstallPlugin(pluginId: string): Promise<any> {
    const res = await fetch(`/api/v1/marketplace/uninstall/${pluginId}`, {
      method: 'DELETE',
    });
    return res.json();
  },
};

export default function Plugins() {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [marketplace, setMarketplace] = useState<MarketplacePlugin[]>([]);
  const [activeTab, setActiveTab] = useState<'installed' | 'marketplace'>('installed');
  const [loading, setLoading] = useState(true);
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [output, setOutput] = useState<string>('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const [installed, available] = await Promise.all([
        pluginService.listPlugins(),
        pluginService.listMarketplace(),
      ]);
      setPlugins(installed);
      setMarketplace(available);
    } catch (err) {
      console.error('Failed to load plugins:', err);
    }
    setLoading(false);
  };

  const handleExecute = async (pluginId: string, command: string) => {
    try {
      const result = await pluginService.executePlugin(pluginId, command);
      setOutput(JSON.stringify(result, null, 2));
    } catch (err) {
      setOutput(`Error: ${err}`);
    }
  };

  const handleToggle = async (pluginId: string, enabled: boolean) => {
    try {
      if (enabled) {
        await pluginService.enablePlugin(pluginId);
      } else {
        await pluginService.disablePlugin(pluginId);
      }
      loadData();
    } catch (err) {
      console.error('Failed to toggle plugin:', err);
    }
  };

  const handleInstall = async (pluginId: string) => {
    try {
      await pluginService.installPlugin(pluginId);
      loadData();
    } catch (err) {
      console.error('Failed to install plugin:', err);
    }
  };

  const handleUninstall = async (pluginId: string) => {
    try {
      await pluginService.uninstallPlugin(pluginId);
      loadData();
    } catch (err) {
      console.error('Failed to uninstall plugin:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading plugins...</div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Plugins</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('installed')}
            className={`px-4 py-2 rounded ${
              activeTab === 'installed'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 text-gray-700'
            }`}
          >
            Installed ({plugins.length})
          </button>
          <button
            onClick={() => setActiveTab('marketplace')}
            className={`px-4 py-2 rounded ${
              activeTab === 'marketplace'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 text-gray-700'
            }`}
          >
            Marketplace ({marketplace.length})
          </button>
        </div>
      </div>

      {activeTab === 'installed' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {plugins.map((plugin) => (
            <div
              key={plugin.id}
              className="border rounded-lg p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-semibold text-lg">{plugin.name}</h3>
                  <p className="text-sm text-gray-500">v{plugin.version} by {plugin.author}</p>
                  <p className="mt-2 text-gray-700">{plugin.description}</p>
                </div>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={true}
                    onChange={(e) => handleToggle(plugin.id, e.target.checked)}
                    className="w-5 h-5"
                  />
                  <span className="text-sm">Enabled</span>
                </label>
              </div>
              <div className="mt-4 flex gap-2">
                {plugin.hooks.map((hook) => (
                  <span
                    key={hook}
                    className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded"
                  >
                    {hook}
                  </span>
                ))}
              </div>
              <div className="mt-4 flex gap-2">
                <button
                  onClick={() => setSelectedPlugin(plugin)}
                  className="px-3 py-1 bg-gray-200 rounded text-sm"
                >
                  Configure
                </button>
                <button
                  onClick={() => handleUninstall(plugin.id)}
                  className="px-3 py-1 bg-red-100 text-red-700 rounded text-sm"
                >
                  Uninstall
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === 'marketplace' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {marketplace.map((plugin) => (
            <div
              key={plugin.id}
              className="border rounded-lg p-4 hover:shadow-md transition-shadow"
            >
              <h3 className="font-semibold text-lg">{plugin.name}</h3>
              <p className="text-sm text-gray-500">v{plugin.version} by {plugin.author}</p>
              <p className="mt-2 text-gray-700">{plugin.description}</p>
              <div className="mt-4">
                {plugin.installed ? (
                  <span className="px-3 py-1 bg-green-100 text-green-700 rounded text-sm">
                    Installed
                  </span>
                ) : (
                  <button
                    onClick={() => handleInstall(plugin.id)}
                    className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
                  >
                    Install
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {selectedPlugin && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-lg w-full">
            <h2 className="text-xl font-bold mb-4">{selectedPlugin.name} Configuration</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Plugin ID</label>
                <input
                  type="text"
                  value={selectedPlugin.id}
                  disabled
                  className="w-full p-2 border rounded bg-gray-50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Version</label>
                <input
                  type="text"
                  value={selectedPlugin.version}
                  disabled
                  className="w-full p-2 border rounded bg-gray-50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Test Command</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    placeholder="e.g., git.status"
                    id="test-command"
                    className="flex-1 p-2 border rounded"
                  />
                  <button
                    onClick={() => {
                      const input = document.getElementById('test-command') as HTMLInputElement;
                      if (input.value) {
                        handleExecute(selectedPlugin.id, input.value);
                      }
                    }}
                    className="px-4 py-2 bg-blue-600 text-white rounded"
                  >
                    Run
                  </button>
                </div>
              </div>
              {output && (
                <div>
                  <label className="block text-sm font-medium mb-1">Output</label>
                  <pre className="p-2 bg-gray-100 rounded text-sm overflow-auto max-h-40">
                    {output}
                  </pre>
                </div>
              )}
            </div>
            <div className="mt-6 flex justify-end">
              <button
                onClick={() => {
                  setSelectedPlugin(null);
                  setOutput('');
                }}
                className="px-4 py-2 bg-gray-200 rounded"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
