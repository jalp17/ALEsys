/**
 * Native filesystem operations via Tauri
 */
import { isTauri } from '../utils/platform';

export async function readTextFile(path: string): Promise<string | null> {
  if (!isTauri()) return null;

  try {
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    return await readTextFile(path);
  } catch (e) {
    console.error('Error reading file:', e);
    return null;
  }
}

export async function writeTextFile(path: string, contents: string): Promise<boolean> {
  if (!isTauri()) return false;

  try {
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    await writeTextFile(path, contents);
    return true;
  } catch (e) {
    console.error('Error writing file:', e);
    return false;
  }
}

export async function readDir(path: string): Promise<string[]> {
  if (!isTauri()) return [];

  try {
    const { readDir } = await import('@tauri-apps/plugin-fs');
    const entries = await readDir(path);
    return entries.map((e) => e.name);
  } catch (e) {
    console.error('Error reading directory:', e);
    return [];
  }
}

export async function fileExists(path: string): Promise<boolean> {
  if (!isTauri()) return false;

  try {
    const { exists } = await import('@tauri-apps/plugin-fs');
    return await exists(path);
  } catch (e) {
    console.error('Error checking file existence:', e);
    return false;
  }
}
