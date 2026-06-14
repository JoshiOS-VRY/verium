import type { CoinId } from './profile';
import sdkProfiles from './coin-profiles.json';

/** SDK-generated chain constants (regenerate via vericonomy-sdk/scripts/generate-coin-profiles.ps1). */
export interface SdkCoinProfile {
  id: string;
  symbol: string;
  display_name: string;
  p2pkh_version: number;
  wif_secret_prefix: number;
  bech32_hrp: string;
  bip44_coin_type: number;
  maturity_confirmations: number;
  default_rpc_port: number;
  default_p2p_port: number;
  earn_mode: string;
  p2pkh_address_prefix: string;
  p2pkh_address_length: number;
}
const SDK_BY_ID = Object.fromEntries(
  (sdkProfiles as SdkCoinProfile[]).map((p) => [p.id, p])
) as Record<CoinId, SdkCoinProfile>;

export function sdkProfile(coin: CoinId): SdkCoinProfile {
  return SDK_BY_ID[coin];
}

export function sdkMaturityConfirmations(coin: CoinId): number {
  return SDK_BY_ID[coin].maturity_confirmations;
}

export function sdkDefaultRpcPort(coin: CoinId): number {
  return SDK_BY_ID[coin].default_rpc_port;
}
