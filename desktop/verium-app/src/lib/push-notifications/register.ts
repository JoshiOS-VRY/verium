import { invoke } from '@tauri-apps/api/core';

export async function pushSyncDevice(deviceToken: string): Promise<void> {
  await invoke('push_sync_device', { deviceToken });
}

export async function pushHeartbeatDevice(deviceToken: string): Promise<void> {
  await invoke('push_heartbeat_device', { deviceToken });
}

export async function pushUnregisterDevice(deviceToken: string): Promise<void> {
  await invoke('push_unregister_device', { deviceToken });
}

export async function pushRegistrationConfigured(): Promise<boolean> {
  return invoke<boolean>('push_registration_configured');
}
