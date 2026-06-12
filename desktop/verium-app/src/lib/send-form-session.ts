import { create } from 'zustand';
import type { SelectedUtxoSet } from '@/components/CoinControlDialog';
import type { CoinId } from '@/lib/coin/profile';

export interface SendFormRecipient {
  id: string;
  address: string;
  label: string;
  amount: string;
}

export interface SendFormSessionSnapshot {
  recipients: SendFormRecipient[];
  subtractFee: boolean;
  feeRate: number;
  coinControl: SelectedUtxoSet;
}

function recipientIsEmpty(recipient: SendFormRecipient): boolean {
  return !recipient.address.trim() && !recipient.label.trim() && !recipient.amount.trim();
}

/** True when the user has entered anything worth restoring after leaving the send view. */
export function sendFormHasUserInput(
  snapshot: SendFormSessionSnapshot,
  baselineFeeRate?: number
): boolean {
  if (snapshot.subtractFee) return true;
  if (snapshot.coinControl.length > 0) return true;
  if (baselineFeeRate != null && snapshot.feeRate !== baselineFeeRate) return true;
  return snapshot.recipients.some((row) => !recipientIsEmpty(row));
}

interface SendFormSessionState {
  byCoin: Partial<Record<CoinId, SendFormSessionSnapshot>>;
  get: (coin: CoinId) => SendFormSessionSnapshot | undefined;
  save: (coin: CoinId, snapshot: SendFormSessionSnapshot) => void;
  clear: (coin: CoinId) => void;
}

export const useSendFormSession = create<SendFormSessionState>((set, get) => ({
  byCoin: {},
  get: (coin) => get().byCoin[coin],
  save: (coin, snapshot) =>
    set((state) => ({
      byCoin: { ...state.byCoin, [coin]: snapshot },
    })),
  clear: (coin) =>
    set((state) => {
      const next = { ...state.byCoin };
      delete next[coin];
      return { byCoin: next };
    }),
}));
