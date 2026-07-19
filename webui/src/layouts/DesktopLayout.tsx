import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { isTauri, getPlatform } from '../utils/platform';

interface DesktopLayoutProps {
  children: React.ReactNode;
}

export function DesktopLayout({ children }: DesktopLayoutProps) {
  const [platform] = useState(getPlatform);
  const [focused, setFocused] = useState(true);

  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;

    const setup = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<boolean>('window-focused', (event) => {
          setFocused(event.payload);
        });
      } catch (e) {
        console.error('Error setting up event listener:', e);
      }
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return (
    <div
      className={`h-screen bg-dark-900 text-white flex flex-col ${focused ? '' : 'opacity-95'}`}
    >
      {/* Title bar for Tauri (draggable) */}
      <div
        data-tauri-drag-region
        className="h-9 bg-dark-800 flex items-center justify-between px-4 select-none border-b border-gray-800"
      >
        <div className="flex items-center gap-2">
          <span className="text-xs font-bold text-primary-400">ALEsys</span>
          <span className="text-[10px] text-gray-500 bg-dark-700 px-1.5 py-0.5 rounded">
            GraphRAG-PG
          </span>
        </div>
        <div className="flex items-center gap-3 text-xs text-gray-500" data-tauri-drag-region>
          {platform === 'desktop' ? 'Desktop' : 'Web'}
        </div>
        <div className="flex gap-1.5">
          <button className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400" />
          <button className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400" />
          <button className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400" />
        </div>
      </div>

      {/* Menu bar */}
      <nav className="h-7 bg-dark-800 border-b border-gray-800 flex items-center px-1 text-xs">
        <MenuItem label="File">
          <MenuItemOption to="/chat">New Tab</MenuItemOption>
          <MenuItemOption to="/editor">New File</MenuItemOption>
          <div className="border-t border-gray-700 my-1" />
          <MenuItemOption action="open_file">Open File...</MenuItemOption>
          <MenuItemOption action="open_folder">Open Folder...</MenuItemOption>
          <div className="border-t border-gray-700 my-1" />
          <MenuItemOption to="/settings">Preferences</MenuItemOption>
          <div className="border-t border-gray-700 my-1" />
          <MenuItemOption action="quit">Quit</MenuItemOption>
        </MenuItem>
        <MenuItem label="Edit">
          <MenuItemOption action="undo">Undo</MenuItemOption>
          <MenuItemOption action="redo">Redo</MenuItemOption>
          <div className="border-t border-gray-700 my-1" />
          <MenuItemOption action="cut">Cut</MenuItemOption>
          <MenuItemOption action="copy">Copy</MenuItemOption>
          <MenuItemOption action="paste">Paste</MenuItemOption>
        </MenuItem>
        <MenuItem label="View">
          <MenuItemOption to="/graph">Graph</MenuItemOption>
          <MenuItemOption to="/search">Search</MenuItemOption>
          <MenuItemOption to="/agents">Agents</MenuItemOption>
          <MenuItemOption to="/plugins">Plugins</MenuItemOption>
          <MenuItemOption to="/orchestrator">Orchestrator</MenuItemOption>
          <div className="border-t border-gray-700 my-1" />
          <MenuItemOption action="toggle_sidebar">Toggle Sidebar</MenuItemOption>
          <MenuItemOption action="toggle_terminal">Toggle Terminal</MenuItemOption>
        </MenuItem>
        <MenuItem label="Run">
          <MenuItemOption action="run_code">Run Code</MenuItemOption>
        </MenuItem>
        <MenuItem label="Help">
          <MenuItemOption action="shortcuts">Keyboard Shortcuts</MenuItemOption>
          <MenuItemOption to="/settings">Settings</MenuItemOption>
        </MenuItem>
      </nav>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  );
}

// =============================================================================
// Sub-components
// =============================================================================

function MenuItem({ label, children }: { label: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);

  return (
    <div
      className="relative"
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      <button
        className={`px-3 py-1 rounded ${open ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'}`}
      >
        {label}
      </button>
      {open && (
        <div className="absolute top-full left-0 mt-0 bg-dark-800 border border-gray-700 rounded shadow-lg py-1 min-w-[180px] z-50">
          {children}
        </div>
      )}
    </div>
  );
}

function MenuItemOption({
  to,
  action,
  children,
}: {
  to?: string;
  action?: string;
  children: React.ReactNode;
}) {
  const handleClick = () => {
    if (action && isTauri()) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('plugin:event', { name: 'menu-action', payload: action }).catch(() => {});
      });
    }
  };

  if (to) {
    return (
      <Link
        to={to}
        className="block px-4 py-1.5 text-xs text-gray-300 hover:bg-gray-700 hover:text-white"
      >
        {children}
      </Link>
    );
  }

  return (
    <button
      onClick={handleClick}
      className="block w-full text-left px-4 py-1.5 text-xs text-gray-300 hover:bg-gray-700 hover:text-white"
    >
      {children}
    </button>
  );
}
