import type { CoinId } from "@/lib/coin/profile";
import { COIN_PROFILES } from "@/lib/coin/profile";
import type { LightServerStatus } from "@/lib/light-wallet/client";

/** Short label for the top bar (e.g. "VRM primary"). */
export function electrumServerShortLabel(
  coin: CoinId,
  status: Pick<
    LightServerStatus,
    "server_host" | "server_port" | "failover_index" | "servers_total"
  >,
): string {
  const symbol = COIN_PROFILES[coin].symbol;
  const host = status.server_host?.toLowerCase() ?? "";

  if (host.includes("vrm1") || host.includes("vrc1")) {
    return `${symbol} primary`;
  }
  if (host.includes("vrm2") || host.includes("vrc2")) {
    return `${symbol} backup`;
  }
  if (status.servers_total > 1) {
    return status.failover_index === 0
      ? `${symbol} primary`
      : `${symbol} backup`;
  }
  return `${symbol} Electrum`;
}

/** Longer label for settings / tooltips. */
export function electrumServerLongLabel(
  coin: CoinId,
  status: Pick<
    LightServerStatus,
    "server_host" | "server_port" | "failover_index" | "servers_total"
  >,
): string {
  const { displayName } = COIN_PROFILES[coin];
  const short = electrumServerShortLabel(coin, status);
  const role = short.includes("primary")
    ? "primary server"
    : short.includes("backup")
      ? "backup server"
      : "server";
  const host = status.server_host ?? "unknown";
  const port = status.server_port != null ? `:${status.server_port}` : "";
  return `${displayName} light wallet · ${role} (${host}${port})`;
}

export function formatChainHeight(height: number): string {
  return height.toLocaleString();
}

/** Friendly label for a configured server URI in settings. */
export function electrumUriFriendlyLabel(
  uri: string,
  coin: CoinId,
  index: number,
): string {
  const host = uri
    .replace(/^tls:\/\//i, "")
    .replace(/^ssl:\/\//i, "")
    .replace(/^tcp:\/\//i, "")
    .split(":")[0]
    .toLowerCase();
  const status = {
    server_host: host,
    server_port: null,
    failover_index: index,
    servers_total: 2,
  };
  return electrumServerShortLabel(coin, status);
}

export function electrumStatusTitle(
  coin: CoinId,
  status: LightServerStatus,
): string {
  const lines = [
    electrumServerLongLabel(coin, status),
    status.tip_height != null
      ? `Chain height: ${formatChainHeight(status.tip_height)}`
      : null,
    status.latency_ms != null ? `Latency: ${status.latency_ms} ms` : null,
    status.banner ? status.banner : null,
  ].filter(Boolean);
  return lines.join("\n");
}
