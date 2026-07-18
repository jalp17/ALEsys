/**
 * Native file dialogs via Tauri
 */
import { isTauri } from '../utils/platform';

export interface FileFilter {
  name: string;
  extensions: string[];
}

export async function openFileDialog(
  title?: string,
  filters?: FileFilter[],
  multiple?: boolean,
): Promise<string | string[] | null> {
  if (!isTauri()) return null;

  try {
    const { open } = await import('@tauri-apps/plugin-dialog');

    const selected = await open({
      title,
      filters: filters?.map((f) => ({
        name: f.name,
        extensions: f.extensions,
      })),
      multiple: multiple ?? false,
    });

    return selected;
  } catch (e) {
    console.error('Error opening file dialog:', e);
    return null;
  }
}

export async function openFolderDialog(title?: string): Promise<string | null> {
  if (!isTauri()) return null;

  try {
    const { open } = await import('@tauri-apps/plugin-dialog');

    const selected = await open({
      title,
      directory: true,
      multiple: false,
    });

    return selected as string | null;
  } catch (e) {
    console.error('Error opening folder dialog:', e);
    return null;
  }
}

export async function saveFileDialog(
  title?: string,
  filters?: FileFilter[],
  defaultPath?: string,
): Promise<string | null> {
  if (!isTauri()) return null;

  try {
    const { save } = await import('@tauri-apps/plugin-dialog');

    const selected = await save({
      title,
      filters: filters?.map((f) => ({
        name: f.name,
        extensions: f.extensions,
      })),
      defaultPath,
    });

    return selected;
  } catch (e) {
    console.error('Error saving file dialog:', e);
    return null;
  }
}

export async function showMessageDialog(
  title: string,
  message: string,
  kind?: 'info' | 'warning' | 'error',
): Promise<void> {
  if (!isTauri()) return;

  try {
    const { message: showMessage } = await import('@tauri-apps/plugin-dialog');
    await showMessage(message, { title, kind });
  } catch (e) {
    console.error('Error showing dialog:', e);
  }
}
