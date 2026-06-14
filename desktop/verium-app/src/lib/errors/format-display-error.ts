/**
 * User-facing copy for errors surfaced in the UI.
 *
 * Tauri `invoke` rejections are already mapped in the Rust `AppError` serializer;
 * this helper covers client-side errors and any legacy/raw strings still in flight.
 */

const RPC_CODE_MESSAGES: Record<number, string> = {
  [-2]: "The node is in safe mode, so this action isn't available right now.",
  [-3]: 'One of the values sent to the node has the wrong type.',
  [-5]: "That address or key doesn't look valid.",
  [-6]: 'Not enough spendable balance for this amount and fee.',
  [-7]: 'The node ran out of memory while handling this request.',
  [-9]: "The node isn't connected to the network yet.",
  [-10]: 'The node is still downloading the blockchain.',
  [-11]: "That label name isn't valid.",
  [-12]: 'The wallet ran out of unused addresses. Try again after it refills.',
  [-13]: 'Unlock your wallet first to continue.',
  [-14]: 'Incorrect wallet passphrase.',
  [-15]: "This action doesn't match the wallet's encryption state.",
  [-16]: "Couldn't encrypt the wallet.",
  [-17]: 'Wallet is already unlocked.',
  [-18]: "That wallet isn't loaded on the node.",
  [-19]: 'No wallet is selected on the node.',
  [-20]: 'A local database error occurred. Try restarting the app.',
  [-27]: 'This transaction is already on the blockchain.',
  [-28]: 'The node is still starting up. Try again in a moment.',
  [-29]: "That peer isn't connected.",
  [-31]: 'Peer networking is disabled on this node.',
  [-32]: 'That RPC command is deprecated on this node.',
  [-32601]: "That command isn't available on the connected node.",
  [-32603]: 'The node hit an internal error. Check Logs for details.',
};

function refineOperationalMessage(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Something went wrong. Try again.';

  const lower = trimmed.toLowerCase();

  if (
    lower.includes('incorrect wallet passphrase') ||
    lower.includes('decrypt failed') ||
    lower.includes('passphrase entered was incorrect')
  ) {
    return 'Incorrect wallet passphrase.';
  }
  if (
    lower.includes('wallet passphrase is required') ||
    lower.includes('passphrase required to send')
  ) {
    return 'Enter your wallet passphrase to send.';
  }
  if (lower.includes('insufficient funds')) {
    return 'Not enough spendable balance for this amount and fee.';
  }
  if (lower.includes('fee too low') || lower.includes('min relay fee')) {
    return 'The network fee is too low. Raise the fee and try again.';
  }
  if (lower.includes('dust')) {
    return 'That amount is too small to send after fees.';
  }
  if (lower.includes('absurdly-high-fee') || lower.includes('absurdly high fee')) {
    return 'The fee looks unusually high. Lower the fee and try again.';
  }
  if (lower.includes('transaction too large')) {
    return 'This transaction is too large. Try sending a smaller amount.';
  }
  if (lower.includes('invalid address')) {
    return "That address doesn't look valid.";
  }
  if (lower.includes('wallet is locked') || lower.includes('unlock needed')) {
    return 'Unlock your wallet first to continue.';
  }
  if (lower.includes('rescanning')) {
    return 'The wallet is rescanning. Wait for it to finish, then try again.';
  }
  if (lower.includes('warming up') || lower.includes('loading block index')) {
    return 'The node is still starting up. Try again in a moment.';
  }
  if (lower.includes('work queue depth exceeded')) {
    return 'The node is busy starting up. Try again in a moment.';
  }
  if (
    lower.includes('electrum connect') ||
    lower.includes('electrum connection closed') ||
    lower.includes('electrum read:') ||
    lower.includes('electrum write:') ||
    lower.includes('electrum tls') ||
    lower.includes('electrum call timed out')
  ) {
    return "Couldn't reach the light-wallet server. Check your connection or try another server.";
  }
  if (lower.includes('excessive resource') || lower.includes('error -101')) {
    return 'The light-wallet server is busy. Wait a moment and try again.';
  }
  if (lower.includes('indexing batch limit reached')) {
    return 'Address lookup is taking longer than expected. Try again in a few minutes.';
  }
  if (lower.includes('manual rescan cooldown') || lower.includes('rescan cooldown')) {
    return 'You can run another address rescan in about an hour.';
  }
  if (lower.includes('unauthorized')) {
    return "Couldn't connect to the node — RPC credentials are missing or incorrect. Check Settings → Daemon connection.";
  }
  if (
    lower.includes('connection refused') ||
    lower.includes('error sending request') ||
    lower.includes('timed out')
  ) {
    return "Couldn't reach the node. It may still be starting — try again shortly.";
  }

  return trimmed;
}

function parsePrefixedRpcError(raw: string): { code: number; message: string } | null {
  const match = /^rpc error (-?\d+): (.+)$/i.exec(raw.trim());
  if (!match) return null;
  return { code: Number(match[1]), message: match[2] };
}

function parsePrefixedElectrumError(raw: string): { code: number; message: string } | null {
  const match = /^electrum error (-?\d+): (.+)$/i.exec(raw.trim());
  if (!match) return null;
  return { code: Number(match[1]), message: match[2] };
}

function mapRpcError(code: number, message: string): string {
  return RPC_CODE_MESSAGES[code] ?? refineOperationalMessage(message);
}

function mapElectrumError(code: number, message: string): string {
  if (code === -101 || message.toLowerCase().includes('excessive resource')) {
    return 'The light-wallet server is busy. Wait a moment and try again.';
  }
  return refineOperationalMessage(message);
}

/** Normalize any thrown/rejected value into copy safe to show users. */
export function formatDisplayError(
  error: unknown,
  fallback = 'Something went wrong. Try again.'
): string {
  const raw =
    error instanceof Error
      ? error.message.trim()
      : typeof error === 'string'
        ? error.trim()
        : String(error).trim();

  if (!raw || raw === '[object Object]') return fallback;

  if (raw.startsWith('daemon not reachable:')) {
    return refineOperationalMessage(raw.slice('daemon not reachable:'.length));
  }

  const rpc = parsePrefixedRpcError(raw);
  if (rpc) return mapRpcError(rpc.code, rpc.message);

  const electrum = parsePrefixedElectrumError(raw);
  if (electrum) return mapElectrumError(electrum.code, electrum.message);

  return refineOperationalMessage(raw);
}
