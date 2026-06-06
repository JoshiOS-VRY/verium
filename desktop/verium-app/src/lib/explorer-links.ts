import type { CoinId } from "@/lib/coin/profile";

/** V2 explorer web UI (external links only — API fetches use `/v1/:chain/wallet`). */
export const EXPLORER_LINK_BASE = "https://explorer.vericonomy.com";

const LEGACY_EXPLORER_HOSTS = [
  "https://explorer-vrm.vericonomy.com",
  "https://explorer-vrc.vericonomy.com",
  "https://staging-explorer.vericonomy.com",
];

export function explorerChainPath(coin: CoinId): "vrm" | "vrc" {
  return coin === "verium" ? "vrm" : "vrc";
}

function chainBase(coin: CoinId): string {
  return `${EXPLORER_LINK_BASE}/${explorerChainPath(coin)}`;
}

export function explorerHome(coin: CoinId): string {
  return `${chainBase(coin)}/`;
}

/** Logo asset URL — served from the production explorer web app. */
export function explorerLogoUrl(coin: CoinId): string {
  const slug = coin === "verium" ? "verium" : "vericoin";
  return `${EXPLORER_LINK_BASE}/img/vericonomy/${slug}-logo.svg`;
}

export function defaultTxExplorerTemplate(coin: CoinId): string {
  return `${chainBase(coin)}/tx/%s`;
}

export function defaultBlockExplorerTemplate(coin: CoinId): string {
  return `${chainBase(coin)}/block/%s`;
}

export function defaultAddressExplorerTemplate(coin: CoinId): string {
  return `${chainBase(coin)}/address/%s`;
}

export function explorerBlocksHash(coin: CoinId): string {
  return `${chainBase(coin)}/`;
}

export function explorerPeersHash(coin: CoinId): string {
  return `${chainBase(coin)}/`;
}

export function explorerExtractionHash(coin: CoinId): string {
  return coin === "verium"
    ? `${EXPLORER_LINK_BASE}/vrm/miners`
    : `${chainBase(coin)}/`;
}

export function explorerRichlistHash(coin: CoinId): string {
  return `${chainBase(coin)}/richlist`;
}

export function explorerProfitabilityHash(_coin: CoinId): string {
  return `${EXPLORER_LINK_BASE}/insights`;
}

function isLegacyExplorerUrl(url: string): boolean {
  return LEGACY_EXPLORER_HOSTS.some((host) => url.startsWith(host));
}

/** Rewrite alpha staging explorer URLs to production (one-time prefs migration). */
export function migrateStagingExplorerTemplate(template: string): string {
  if (!template.includes("staging-explorer.vericonomy.com")) return template;
  return template.replaceAll(
    "staging-explorer.vericonomy.com",
    "explorer.vericonomy.com",
  );
}

/** Apply staging → production migration to all explorer URL prefs. */
export function migrateExplorerPrefs(
  prefs: Partial<{
    explorer_tx_url_template?: string;
    explorer_block_url_template?: string;
    explorer_address_url_template?: string;
  }>,
): Partial<{
  explorer_tx_url_template: string;
  explorer_block_url_template?: string;
  explorer_address_url_template?: string;
}> | null {
  const tx = prefs.explorer_tx_url_template;
  const block = prefs.explorer_block_url_template;
  const addr = prefs.explorer_address_url_template;
  const nextTx = tx ? migrateStagingExplorerTemplate(tx) : undefined;
  const nextBlock = block ? migrateStagingExplorerTemplate(block) : undefined;
  const nextAddr = addr ? migrateStagingExplorerTemplate(addr) : undefined;
  if (
    nextTx === tx &&
    nextBlock === block &&
    nextAddr === addr
  ) {
    return null;
  }
  return {
    ...(nextTx && nextTx !== tx ? { explorer_tx_url_template: nextTx } : {}),
    ...(nextBlock && nextBlock !== block
      ? { explorer_block_url_template: nextBlock }
      : {}),
    ...(nextAddr && nextAddr !== addr
      ? { explorer_address_url_template: nextAddr }
      : {}),
  };
}

function otherCoin(coin: CoinId): CoinId {
  return coin === "verium" ? "vericoin" : "verium";
}

function legacyHashTemplate(
  coin: CoinId,
  fragment: "tx" | "block" | "address",
): string {
  const host =
    coin === "verium"
      ? "https://explorer-vrm.vericonomy.com"
      : "https://explorer-vrc.vericonomy.com";
  return `${host}/#${fragment}/%s`;
}

/** True when a saved template targets the other chain (vrm vs vrc path or legacy host). */
export function explorerTemplateTargetsOtherChain(
  coin: CoinId,
  stored: string,
): boolean {
  const lower = stored.toLowerCase();
  const otherPath = explorerChainPath(otherCoin(coin));
  if (lower.includes(`/${otherPath}/`) || lower.includes(`/${otherPath}#`)) {
    return true;
  }
  if (coin === "verium" && lower.includes("explorer-vrc")) return true;
  if (coin === "vericoin" && lower.includes("explorer-vrm")) return true;
  return false;
}

function effectiveExplorerTemplate(
  coin: CoinId,
  stored: string | undefined,
  fragment: "tx" | "block" | "address",
  defaultFor: (c: CoinId) => string,
): string {
  const coinDefault = defaultFor(coin);
  if (!stored) return coinDefault;
  if (isLegacyExplorerUrl(stored)) return coinDefault;
  if (stored === legacyHashTemplate(coin, fragment)) return coinDefault;
  if (stored === legacyHashTemplate(otherCoin(coin), fragment)) return coinDefault;
  if (stored === defaultFor(otherCoin(coin))) return coinDefault;
  if (explorerTemplateTargetsOtherChain(coin, stored)) return coinDefault;
  return stored;
}

/** Prefer the active coin's explorer when prefs still hold a legacy or other-chain default. */
export function effectiveTxExplorerTemplate(
  coin: CoinId,
  stored: string | undefined,
): string {
  return effectiveExplorerTemplate(
    coin,
    stored,
    "tx",
    defaultTxExplorerTemplate,
  );
}

export function effectiveBlockExplorerTemplate(
  coin: CoinId,
  stored: string | undefined,
): string {
  return effectiveExplorerTemplate(
    coin,
    stored,
    "block",
    defaultBlockExplorerTemplate,
  );
}

export function effectiveAddressExplorerTemplate(
  coin: CoinId,
  stored: string | undefined,
): string {
  return effectiveExplorerTemplate(
    coin,
    stored,
    "address",
    defaultAddressExplorerTemplate,
  );
}

export function buildTxExplorerUrl(
  coin: CoinId,
  template: string,
  txid: string,
): string {
  const resolved = effectiveTxExplorerTemplate(coin, template);
  const safe =
    resolved && resolved.includes("%s")
      ? resolved
      : defaultTxExplorerTemplate(coin);
  return safe.replace("%s", encodeURIComponent(txid));
}

export function buildBlockExplorerUrl(
  coin: CoinId,
  template: string,
  blockHashOrHeight: string | number,
): string {
  const resolved = effectiveBlockExplorerTemplate(coin, template);
  const safe =
    resolved && resolved.includes("%s")
      ? resolved
      : defaultBlockExplorerTemplate(coin);
  return safe.replace("%s", encodeURIComponent(String(blockHashOrHeight)));
}

export function buildAddressExplorerUrl(
  coin: CoinId,
  template: string,
  address: string,
): string {
  const resolved = effectiveAddressExplorerTemplate(coin, template);
  const safe =
    resolved && resolved.includes("%s")
      ? resolved
      : defaultAddressExplorerTemplate(coin);
  return safe.replace("%s", encodeURIComponent(address));
}
