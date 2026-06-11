const COIN = 100_000_000;

export function fmtVrm(sat: string | number | bigint | null | undefined, dp = 4): string {
  if (sat === null || sat === undefined) return '0';
  const value = Number(typeof sat === 'bigint' ? sat.toString() : sat) / COIN;
  return value.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: dp,
  });
}

/** Internal metrics are H/s; Verium display uses H/m (cpuminer convention). */
export function hsToHm(hps: number | null | undefined): number {
  return Number(hps ?? 0) * 60;
}

export function fmtHashrate(hps: number | null | undefined): string {
  const hm = hsToHm(hps);
  if (!isFinite(hm) || hm <= 0) return '0 H/m';
  if (hm >= 10_000) return `${(hm / 1000).toFixed(2)} kH/m`;
  if (hm >= 1000) return `${Math.round(hm).toLocaleString()} H/m`;
  return `${hm.toFixed(1)} H/m`;
}

export function fmtHashrateAxis(hps: number | null | undefined): string {
  const hm = hsToHm(hps);
  if (!isFinite(hm) || hm <= 0) return '0';
  if (hm >= 10_000) return `${(hm / 1000).toFixed(1)}k`;
  if (hm >= 1000) return `${Math.round(hm / 1000)}k`;
  return `${Math.round(hm)}`;
}

export function fmtRejectPct(
  accepted: number,
  rejected: number,
  stale = 0
): { label: string; tone: 'good' | 'warn' | 'bad' | 'muted' } {
  const total = accepted + rejected + stale;
  if (total === 0) return { label: '—', tone: 'muted' };
  const pct = (rejected / total) * 100;
  const tone = pct >= 15 ? 'bad' : pct >= 2 ? 'warn' : 'good';
  return { label: `${pct.toFixed(1)}%`, tone };
}

export function timeAgo(iso: string | null | undefined): string {
  if (!iso) return 'never';
  const then = new Date(iso).getTime();
  const sec = Math.floor((Date.now() - then) / 1000);
  if (sec < 0) return 'just now';
  if (sec < 60) return `${sec}s ago`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m ago`;
  if (sec < 86400) return `${Math.floor(sec / 3600)}h ago`;
  return `${Math.floor(sec / 86400)}d ago`;
}

export function shortHash(hash: string | null | undefined, head = 10, tail = 8): string {
  if (!hash) return '';
  if (hash.length <= head + tail + 1) return hash;
  return `${hash.slice(0, head)}…${hash.slice(-tail)}`;
}
