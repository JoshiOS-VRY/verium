import { invoke } from '@tauri-apps/api/core';

/**
 * Opens a URL in the system browser (Safari on iOS). Routed through a Tauri
 * command that uses the opener plugin on mobile — the `open` crate does not
 * work on iOS.
 */
export async function openExternal(url: string): Promise<void> {
  const trimmed = url.trim();
  if (!trimmed) return;
  try {
    await invoke<void>('open_external_url', { url: trimmed });
  } catch (err) {
    console.error('openExternal failed:', err);
    throw err;
  }
}
