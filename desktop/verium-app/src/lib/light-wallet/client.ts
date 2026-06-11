import { invoke } from '@tauri-apps/api/core';
import type { CoinId } from '@/lib/coin/profile';

export type WalletMode = 'full_node' | 'light';

export interface WalletModeStatus {
  mode: WalletMode;
  light_wallet_enabled: boolean;
  light_wallet_exists: boolean;
  electrum_servers: string[];
}

export interface LightServerStatus {
  connected: boolean;
  server_host: string | null;
  server_port: number | null;
  latency_ms: number | null;
  tip_height: number | null;
  banner: string | null;
  failover_index: number;
  servers_total: number;
}

export interface ConformanceCheck {
  method: string;
  ok: boolean;
  detail: string;
  optional?: boolean;
}

export interface ConformanceResult {
  server: string;
  passed: boolean;
  checks: ConformanceCheck[];
}

export async function walletModeGet(): Promise<WalletModeStatus> {
  return invoke<WalletModeStatus>('wallet_mode_get');
}

export async function walletModeSet(mode: WalletMode): Promise<void> {
  return invoke('wallet_mode_set', { mode });
}

/** Effective wallet mode for a single coin (per-coin override or app default). */
export async function walletModeGetForCoin(coin: CoinId): Promise<WalletModeStatus> {
  return invoke<WalletModeStatus>('wallet_mode_get_for_coin', { coin });
}

/** Set the wallet mode for one coin without affecting the other chain. */
export async function walletModeSetForCoin(coin: CoinId, mode: WalletMode): Promise<void> {
  return invoke('wallet_mode_set_for_coin', { coin, mode });
}

export async function electrumServersGet(coin: CoinId): Promise<string[]> {
  return invoke<string[]>('electrum_servers_get', { coin });
}

export async function electrumServersSet(coin: CoinId, servers: string[]): Promise<void> {
  return invoke('electrum_servers_set', { coin, servers });
}

export async function electrumTestConnection(
  coin: CoinId,
  server?: string
): Promise<ConformanceResult> {
  return invoke<ConformanceResult>('electrum_test_connection', { coin, server });
}

export async function electrumValidateDefaults(coin: CoinId): Promise<ConformanceResult[]> {
  return invoke<ConformanceResult[]>('electrum_validate_defaults', { coin });
}

export async function lightWalletCreate(
  coin: CoinId,
  mnemonic: string,
  passphrase: string,
  label?: string
): Promise<void> {
  return invoke('light_wallet_create', { coin, mnemonic, passphrase, label });
}

/** Replace the active-chain light wallet from a BIP39 phrase or xprv master key. */
export async function lightWalletImport(
  coin: CoinId,
  mnemonic: string,
  passphrase: string,
  label?: string,
  totpCode?: string
): Promise<void> {
  return invoke('light_wallet_import', {
    coin,
    mnemonic,
    passphrase,
    label,
    totpCode: totpCode?.trim() || null,
  });
}

export async function lightWalletUnlock(
  coin: CoinId,
  passphrase: string,
  seconds?: number
): Promise<void> {
  return invoke('light_wallet_unlock', { coin, passphrase, seconds });
}

export async function lightWalletLock(coin: CoinId): Promise<void> {
  return invoke('light_wallet_lock', { coin });
}

export async function lightWalletExists(coin: CoinId): Promise<boolean> {
  return invoke<boolean>('light_wallet_exists', { coin });
}

export async function lightServerStatus(coin: CoinId): Promise<LightServerStatus | null> {
  return invoke<LightServerStatus | null>('light_server_status', { coin });
}

export interface ServerTip {
  server: string;
  height: number | null;
  error: string | null;
}

export interface TipVerifyResult {
  consistent: boolean;
  tips: ServerTip[];
  max_drift_blocks: number;
}

export async function electrumCrossVerifyTip(coin: CoinId): Promise<TipVerifyResult> {
  return invoke<TipVerifyResult>('electrum_cross_verify_tip', { coin });
}
