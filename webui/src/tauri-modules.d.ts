declare module '@tauri-apps/plugin-dialog' {
  export function open(options?: {
    title?: string;
    filters?: { name: string; extensions: string[] }[];
    multiple?: boolean;
    directory?: boolean;
    defaultPath?: string;
  }): Promise<string | string[] | null>;

  export function save(options?: {
    title?: string;
    filters?: { name: string; extensions: string[] }[];
    defaultPath?: string;
  }): Promise<string | null>;

  export function message(
    message: string,
    options?: { title?: string; kind?: 'info' | 'warning' | 'error' },
  ): Promise<void>;
}

declare module '@tauri-apps/plugin-fs' {
  export function readTextFile(path: string): Promise<string>;
  export function writeTextFile(path: string, contents: string): Promise<void>;
  export function readDir(path: string): Promise<{ name: string; isFile: boolean; isDirectory: boolean }[]>;
  export function exists(path: string): Promise<boolean>;
}

declare module '@tauri-apps/plugin-global-shortcut' {
  export function register(shortcut: string, handler: () => void): Promise<void>;
  export function unregister(shortcut: string): Promise<void>;
}

declare module '@tauri-apps/api/event' {
  export function listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void>;
}

declare module '@tauri-apps/api/core' {
  export function invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown>;
}
