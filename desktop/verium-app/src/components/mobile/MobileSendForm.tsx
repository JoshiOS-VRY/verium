import { useState } from 'react';
import {
  BookUser,
  ChevronDown,
  ClipboardPaste,
  Coins,
  Loader2,
  Plus,
  QrCode,
  SendHorizontal,
  X,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { ExplorerLink } from '@/components/ExplorerLink';
import type { CoinId } from '@/lib/coin/profile';
import { getCoinProfile } from '@/lib/coin/profile';
import { formatCoinAmount } from '@/lib/units';
import { validateSendAddress } from '@/lib/address-validation';
import { cn } from '@/lib/utils';

interface SendRecipient {
  id: string;
  address: string;
  label: string;
  amount: string;
}

interface SendSuccessResult {
  txids: string[];
  recipients: { address: string; label?: string; amount: number }[];
  totalAmount: number;
  completedAt: number;
}

function MobileSendSuccess({
  result,
  coin,
  onDismiss,
}: {
  result: SendSuccessResult;
  coin: CoinId;
  onDismiss: () => void;
}) {
  const symbol = getCoinProfile(coin).symbol;
  const primaryTxid = result.txids[0];

  return (
    <div className="rounded-2xl border border-success/30 bg-success/10 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-sm font-semibold text-fg">Payment sent</p>
          <p className="mt-1 text-sm tabular-nums text-fg">
            {formatCoinAmount(result.totalAmount, coin, 4)} {symbol}
          </p>
        </div>
        <button
          type="button"
          onClick={onDismiss}
          className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-fg-muted active:bg-bg-subtle"
          aria-label="Dismiss"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      {primaryTxid && (
        <ExplorerLink
          coin={coin}
          target={{ kind: 'tx', txid: primaryTxid }}
          label="View on explorer"
          className="mt-3 text-sm text-accent"
        />
      )}
      <p className="mt-2 text-xs text-fg-muted">Awaiting network confirmation.</p>
    </div>
  );
}

export function MobileSendForm({
  coin,
  profile,
  symbol,
  exampleAddress,
  balance,
  feeRate,
  subtractFee,
  onSubtractFeeChange,
  recipients,
  updateRecipient,
  removeRecipient,
  addRecipient,
  pasteAddress,
  useAvailableBalance,
  openQrScan,
  openAddressBook,
  openFeeDialog,
  openCoinControl,
  coinControlCount,
  coinControlTotal,
  clearCoinControl,
  showCoinControl,
  canSend,
  preparingConfirm,
  sendPending,
  onSend,
  onClear,
  clipboardGuardError,
  spendWarning,
  sendError,
  lastSend,
  onDismissSuccess,
}: {
  coin: CoinId;
  profile: { displayName: string; symbol: string };
  symbol: string;
  exampleAddress?: string;
  balance: number;
  feeRate: number;
  subtractFee: boolean;
  onSubtractFeeChange: (value: boolean) => void;
  recipients: SendRecipient[];
  updateRecipient: (id: string, patch: Partial<SendRecipient>) => void;
  removeRecipient: (id: string) => void;
  addRecipient: () => void;
  pasteAddress: (id: string) => void;
  useAvailableBalance: (id: string) => void;
  openQrScan: (id: string) => void;
  openAddressBook: (id: string) => void;
  openFeeDialog: () => void;
  openCoinControl: () => void;
  coinControlCount: number;
  coinControlTotal: number;
  clearCoinControl: () => void;
  showCoinControl: boolean;
  canSend: boolean;
  preparingConfirm: boolean;
  sendPending: boolean;
  onSend: () => void;
  onClear: () => void;
  clipboardGuardError: string | null;
  spendWarning: string | null;
  sendError: string | null;
  lastSend: SendSuccessResult | null;
  onDismissSuccess: () => void;
}) {
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [expandedNotes, setExpandedNotes] = useState<Set<string>>(() => new Set());

  const toggleNote = (id: string) => {
    setExpandedNotes((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="mobile-send-available rounded-2xl border border-border bg-gradient-to-br from-bg-panel to-bg-subtle/50 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <span className="text-sm text-fg-muted">Available to send</span>
          <span className="text-lg font-bold tabular-nums tracking-tight text-fg">
            {formatCoinAmount(balance, coin, 4)}
            <span className="ml-1 text-sm font-medium text-fg-muted">{profile.symbol}</span>
          </span>
        </div>
      </div>

      {lastSend && (
        <MobileSendSuccess result={lastSend} coin={coin} onDismiss={onDismissSuccess} />
      )}

      {recipients.map((row, index) => {
        const addressError = row.address.trim() ? validateSendAddress(row.address.trim()) : null;
        const showNote = expandedNotes.has(row.id) || row.label.trim().length > 0;

        return (
          <section
            key={row.id}
            className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4"
          >
            {recipients.length > 1 && (
              <div className="mb-3 flex items-center justify-between gap-2">
                <p className="text-xs font-semibold uppercase tracking-wide text-fg-subtle">
                  Recipient {index + 1}
                </p>
                <button
                  type="button"
                  onClick={() => removeRecipient(row.id)}
                  className="flex h-8 w-8 items-center justify-center rounded-lg text-fg-muted active:bg-bg-subtle"
                  aria-label="Remove recipient"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
            )}

            <div className="flex flex-col gap-4">
              <div>
                <label htmlFor={`mobile-pay-to-${row.id}`} className="mobile-field-label">
                  To
                </label>
                <input
                  id={`mobile-pay-to-${row.id}`}
                  type="text"
                  spellCheck={false}
                  autoCapitalize="off"
                  autoCorrect="off"
                  enterKeyHint="next"
                  value={row.address}
                  onChange={(e) => updateRecipient(row.id, { address: e.target.value })}
                  placeholder={
                    exampleAddress
                      ? `Paste or scan a ${profile.displayName} address`
                      : `${profile.displayName} address`
                  }
                  className={cn(
                    'mobile-input mt-1.5 w-full font-mono text-[15px]',
                    addressError && 'border-danger'
                  )}
                />
                {addressError && <p className="mt-1.5 text-xs text-danger">{addressError}</p>}
                <div className="mt-2 grid grid-cols-3 gap-2">
                  <Button
                    type="button"
                    variant="secondary"
                    size="md"
                    className="mobile-send-action h-11 min-h-[44px] rounded-xl text-xs"
                    onClick={() => openQrScan(row.id)}
                  >
                    <QrCode className="h-4 w-4 shrink-0" />
                    Scan
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    size="md"
                    className="mobile-send-action h-11 min-h-[44px] rounded-xl text-xs"
                    onClick={() => void pasteAddress(row.id)}
                  >
                    <ClipboardPaste className="h-4 w-4 shrink-0" />
                    Paste
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    size="md"
                    className="mobile-send-action h-11 min-h-[44px] rounded-xl text-xs"
                    onClick={() => openAddressBook(row.id)}
                  >
                    <BookUser className="h-4 w-4 shrink-0" />
                    Contacts
                  </Button>
                </div>
              </div>

              <div>
                <label htmlFor={`mobile-amount-${row.id}`} className="mobile-field-label">
                  Amount
                </label>
                <div className="relative mt-1.5">
                  <input
                    id={`mobile-amount-${row.id}`}
                    type="text"
                    inputMode="decimal"
                    enterKeyHint="done"
                    autoComplete="off"
                    value={row.amount}
                    onChange={(e) => updateRecipient(row.id, { amount: e.target.value })}
                    placeholder="0.00"
                    className="mobile-input mobile-send-amount min-h-[52px] w-full pr-[4.75rem] text-2xl font-semibold tabular-nums"
                  />
                  <button
                    type="button"
                    className="absolute right-2 top-1/2 flex h-9 min-w-[2.75rem] -translate-y-1/2 items-center justify-center rounded-lg px-2 text-sm font-semibold text-accent active:bg-accent/10 disabled:opacity-40"
                    onClick={() => useAvailableBalance(row.id)}
                    disabled={balance <= 0}
                    aria-label="Use maximum available balance"
                  >
                    Max
                  </button>
                </div>
                {index === 0 && (
                  <label className="mobile-checkbox-row mt-2 text-fg-muted">
                    <input
                      type="checkbox"
                      checked={subtractFee}
                      onChange={(e) => onSubtractFeeChange(e.target.checked)}
                      className="h-5 w-5 shrink-0 rounded accent-accent"
                    />
                    <span>Subtract fee from amount</span>
                  </label>
                )}
              </div>

              {showNote ? (
                <div>
                  <label htmlFor={`mobile-label-${row.id}`} className="mobile-field-label">
                    Note
                  </label>
                  <input
                    id={`mobile-label-${row.id}`}
                    type="text"
                    enterKeyHint="done"
                    value={row.label}
                    onChange={(e) => updateRecipient(row.id, { label: e.target.value })}
                    placeholder="Saved to your address book"
                    className="mobile-input mt-1.5 w-full"
                  />
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => toggleNote(row.id)}
                  className="text-sm font-medium text-accent active:opacity-80"
                >
                  Add a note
                </button>
              )}
            </div>
          </section>
        );
      })}

      <Button
        type="button"
        variant="secondary"
        size="md"
        className="h-11 min-h-[44px] w-full rounded-xl"
        onClick={addRecipient}
      >
        <Plus className="h-4 w-4" />
        Add another recipient
      </Button>

      <button
        type="button"
        onClick={() => setShowAdvanced((v) => !v)}
        className="flex min-h-[44px] w-full items-center justify-between rounded-xl border border-border bg-bg-subtle/60 px-4 py-3 text-sm font-medium text-fg active:bg-bg-subtle"
        aria-expanded={showAdvanced}
      >
        <span>Fee &amp; advanced</span>
        <ChevronDown
          className={cn('h-4 w-4 text-fg-muted transition-transform', showAdvanced && 'rotate-180')}
        />
      </button>

      {showAdvanced && (
        <div className="flex flex-col gap-3 rounded-2xl border border-border bg-bg-subtle/40 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span className="text-fg-muted">
              Fee rate{' '}
              <span className="font-medium tabular-nums text-fg">
                {feeRate.toFixed(4)} {symbol}/kB
              </span>
            </span>
            <Button type="button" variant="secondary" size="sm" onClick={openFeeDialog}>
              Change
            </Button>
          </div>
          {showCoinControl && (
            <div className="flex flex-wrap items-center justify-between gap-2 text-sm">
              <span className="text-fg-muted">
                Coin control
                {coinControlCount > 0 && (
                  <span className="ml-1 font-medium tabular-nums text-fg">
                    · {formatCoinAmount(coinControlTotal, coin, 4)}
                  </span>
                )}
              </span>
              <Button type="button" variant="secondary" size="sm" onClick={openCoinControl}>
                <Coins className="h-4 w-4" />
                {coinControlCount > 0 ? `${coinControlCount} inputs` : 'Select'}
              </Button>
            </div>
          )}
          {coinControlCount > 0 && (
            <button
              type="button"
              className="text-sm text-accent underline"
              onClick={clearCoinControl}
            >
              Clear coin selection
            </button>
          )}
        </div>
      )}

      {clipboardGuardError && (
        <div className="rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {clipboardGuardError}
        </div>
      )}
      {spendWarning && (
        <div className="rounded-xl border border-warning/30 bg-warning/10 px-4 py-3 text-sm text-warning">
          {spendWarning}
        </div>
      )}
      {sendError && (
        <div className="rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {sendError}
        </div>
      )}

      <div className="mobile-send-sticky-bar">
        <div className="flex flex-col gap-2">
          <Button
            type="button"
            size="lg"
            className="h-12 min-h-[48px] w-full rounded-xl text-base font-semibold"
            disabled={!canSend}
            onClick={onSend}
          >
            {preparingConfirm || sendPending ? (
              <>
                <Loader2 className="h-5 w-5 animate-spin" aria-hidden />
                {preparingConfirm ? 'Checking…' : 'Sending…'}
              </>
            ) : (
              <>
                <SendHorizontal className="h-5 w-5" />
                Send {profile.symbol}
              </>
            )}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="md"
            className="h-11 min-h-[44px] w-full rounded-xl text-fg-muted"
            onClick={onClear}
          >
            Clear form
          </Button>
        </div>
      </div>
    </div>
  );
}
