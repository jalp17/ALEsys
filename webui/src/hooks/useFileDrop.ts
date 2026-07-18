import { useEffect, useState } from 'react';
import { isTauri } from '../utils/platform';

interface FileDropState {
  isDragging: boolean;
  paths: string[];
}

export function useFileDrop(): FileDropState {
  const [state, setState] = useState<FileDropState>({ isDragging: false, paths: [] });

  useEffect(() => {
    if (!isTauri()) return;

    let cleanup: (() => void) | undefined;

    const setup = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        const unlistenDrag = await listen<string[]>('tauri://drag-enter', () => {
          setState((prev) => ({ ...prev, isDragging: true }));
        });

        const unlistenDrop = await listen<string[]>('tauri://drag-drop', (event) => {
          setState({ isDragging: false, paths: event.payload });
        });

        const unlistenLeave = await listen<string[]>('tauri://drag-leave', () => {
          setState((prev) => ({ ...prev, isDragging: false }));
        });

        cleanup = () => {
          unlistenDrag();
          unlistenDrop();
          unlistenLeave();
        };
      } catch (e) {
        console.error('Error setting up file drop listener:', e);
      }
    };

    setup();

    return () => {
      if (cleanup) cleanup();
    };
  }, []);

  return state;
}
