import { useEffect } from 'react';
import { isTauri } from '../utils/platform';

interface ShortcutDef {
  key: string;
  modifiers?: string[];
  handler: () => void;
}

export function useGlobalShortcuts(shortcuts: ShortcutDef[]) {
  useEffect(() => {
    if (!isTauri()) return;

    let cleanup: (() => void) | undefined;

    const setup = async () => {
      try {
        const { register, unregister: unreg } = await import('@tauri-apps/plugin-global-shortcut');

        for (const shortcut of shortcuts) {
          const combo = [...(shortcut.modifiers || []), shortcut.key].join('+');
          await register(combo, () => {
            shortcut.handler();
          });
        }

        cleanup = () => {
          shortcuts.forEach((s) => {
            const combo = [...(s.modifiers || []), s.key].join('+');
            unreg(combo).catch(() => {});
          });
        };
      } catch (e) {
        console.error('Error registering shortcuts:', e);
      }
    };

    setup();

    return () => {
      if (cleanup) cleanup();
    };
  }, [shortcuts]);
}
