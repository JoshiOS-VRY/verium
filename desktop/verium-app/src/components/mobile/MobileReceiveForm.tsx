import { useState } from 'react';
import { ChevronDown, Copy, Check, Loader2, QrCode, Radio, Trash2, X } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { ExplorerLink } from '@/components/ExplorerLink';
import { QrCodeDisplay } from '@/components/QrCodeDisplay';
import type { ReceiveRequest } from '@/lib/security/client';
import type { CoinId } from '@/lib/coin/profile';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';

export function MobileReceiveForm({
  coin,
  profile,
  label,
  amount,
  message,
  onLabelChange,
  onAmountChange,
  onMessageChange,
  onClearForm,
  onCreate,
  onCreateAddressOnly,
  creating,
  creatingAddress,
  createError,
  plainAddress,
  showPlainQr,
  onShowPlainQr,
  onHidePlainQr,
  onDismissPlainAddress,
  requests,
  selected,
  onSelectRequest,
  onCloseDetail,
  onDeleteRequest,
  deleting,
  addressCopied,
  onCopyAddress,
  isLight,
  warnings,
}: {
  coin: CoinId;
  profile: { displayName: string; symbol: string };
  label: string;
  amount: string;
  message: string;
  onLabelChange: (value: string) => void;
  onAmountChange: (value: string) => void;
  onMessageChange: (value: string) => void;
  onClearForm: () => void;
  onCreate: () => void;
  onCreateAddressOnly: () => void;
  creating: boolean;
  creatingAddress: boolean;
  createError: string | null;
  plainAddress: string | null;
  showPlainQr: boolean;
  onShowPlainQr: () => void;
  onHidePlainQr: () => void;
  onDismissPlainAddress: () => void;
  selected: ReceiveRequest | null;
  requests: ReceiveRequest[];
  onSelectRequest: (id: string) => void;
  onCloseDetail: () => void;
  onDeleteRequest: (id: string) => void;
  deleting: boolean;
  addressCopied: boolean;
  onCopyAddress: (address: string) => void;
  isLight: boolean;
  warnings: { id: string; title: string; body: string; tone?: 'warning' | 'neutral' }[];
}) {
  const [showDetails, setShowDetails] = useState(false);
  const [showWarnings, setShowWarnings] = useState(false);

  const active = selected;
  const busy = creating || creatingAddress;

  return (
    <div className="mobile-send-form-root flex flex-col gap-4">
      {isLight && (
        <div className="flex items-center gap-2 rounded-xl border border-accent/25 bg-accent/5 px-3 py-2.5 text-sm text-fg-muted">
          <Radio className="h-4 w-4 shrink-0 animate-pulse text-accent" aria-hidden />
          <span>Watching for incoming payments on this screen</span>
        </div>
      )}

      {warnings.length > 0 && (
        <div className="rounded-2xl border border-border bg-bg-subtle/50">
          <button
            type="button"
            onClick={() => setShowWarnings((v) => !v)}
            className="flex min-h-[44px] w-full items-center justify-between px-4 py-3 text-sm font-medium text-fg"
          >
            <span>Network notices ({warnings.length})</span>
            <ChevronDown
              className={cn(
                'h-4 w-4 text-fg-muted transition-transform',
                showWarnings && 'rotate-180'
              )}
            />
          </button>
          {showWarnings && (
            <div className="space-y-2 border-t border-border px-4 pb-3 pt-2">
              {warnings.map((w) => (
                <div
                  key={w.id}
                  className={cn(
                    'rounded-xl px-3 py-2.5 text-xs',
                    w.tone === 'warning'
                      ? 'border border-warning/30 bg-warning/10 text-fg'
                      : 'border border-border bg-bg-panel text-fg-muted'
                  )}
                >
                  <p className="font-medium">{w.title}</p>
                  <p className="mt-1">{w.body}</p>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
        {active ? (
          <>
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <p className="text-xs font-semibold uppercase tracking-wide text-fg-subtle">
                  {active.label || 'Payment request'}
                </p>
                {active.amount != null && (
                  <p className="mt-1 text-2xl font-bold tabular-nums tracking-tight text-fg">
                    {formatCoinAmount(active.amount, coin, 4)}
                  </p>
                )}
              </div>
              <button
                type="button"
                onClick={onCloseDetail}
                className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-fg-muted active:bg-bg-subtle"
                aria-label="Close QR"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            {active.message && <p className="mt-2 text-sm text-fg-muted">{active.message}</p>}

            <div className="mt-4 flex justify-center">
              <QrCodeDisplay
                coin={coin}
                address={active.address}
                amount={active.amount}
                label={active.label || undefined}
                message={active.message || undefined}
                size={240}
              />
            </div>

            <div
              className={cn(
                'mt-4 flex items-center gap-2 rounded-xl border px-3 py-2.5 text-sm transition-colors',
                addressCopied ? 'border-success/40 bg-success/5' : 'border-border bg-bg-subtle'
              )}
            >
              <span className="min-w-0 flex-1 break-all font-mono text-xs">{active.address}</span>
              <button
                type="button"
                aria-label={addressCopied ? 'Address copied' : 'Copy address'}
                onClick={() => onCopyAddress(active.address)}
                className={cn(
                  'flex h-10 w-10 shrink-0 items-center justify-center rounded-lg transition-colors',
                  addressCopied ? 'text-success' : 'text-fg-muted active:bg-bg-panel'
                )}
              >
                {addressCopied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
              </button>
            </div>

            <div className="mt-3 flex flex-wrap items-center gap-3 text-xs text-fg-muted">
              <ExplorerLink
                coin={coin}
                target={{ kind: 'address', address: active.address }}
                label="View on explorer"
                className="text-accent"
              />
              <span>Created {new Date(active.created_at * 1000).toLocaleString()}</span>
            </div>

            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="mt-3 h-10 min-h-[40px] text-danger"
              disabled={deleting}
              onClick={() => onDeleteRequest(active.id)}
            >
              <Trash2 className="h-4 w-4" />
              Remove request
            </Button>
          </>
        ) : plainAddress ? (
          <>
            <div className="flex items-start justify-between gap-3">
              <p className="text-xs font-semibold uppercase tracking-wide text-fg-subtle">
                Receive address
              </p>
              <button
                type="button"
                onClick={onDismissPlainAddress}
                className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-fg-muted active:bg-bg-subtle"
                aria-label="Dismiss address"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            {showPlainQr && (
              <div className="mt-4 flex justify-center">
                <QrCodeDisplay coin={coin} address={plainAddress} size={240} />
              </div>
            )}

            <div
              className={cn(
                'mt-4 flex items-center gap-2 rounded-xl border px-3 py-2.5 text-sm transition-colors',
                addressCopied ? 'border-success/40 bg-success/5' : 'border-border bg-bg-subtle'
              )}
            >
              <span className="min-w-0 flex-1 break-all font-mono text-xs">{plainAddress}</span>
              <button
                type="button"
                aria-label={addressCopied ? 'Address copied' : 'Copy address'}
                onClick={() => onCopyAddress(plainAddress)}
                className={cn(
                  'flex h-10 w-10 shrink-0 items-center justify-center rounded-lg transition-colors',
                  addressCopied ? 'text-success' : 'text-fg-muted active:bg-bg-panel'
                )}
              >
                {addressCopied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
              </button>
            </div>

            <div className="mt-3 flex flex-wrap items-center gap-3">
              <Button
                type="button"
                variant="secondary"
                size="sm"
                className="h-10 min-h-[40px] rounded-xl"
                onClick={showPlainQr ? onHidePlainQr : onShowPlainQr}
              >
                <QrCode className="h-4 w-4" />
                {showPlainQr ? 'Hide QR code' : 'Show QR code'}
              </Button>
              <ExplorerLink
                coin={coin}
                target={{ kind: 'address', address: plainAddress }}
                label="View on explorer"
                className="text-xs text-accent"
              />
            </div>
          </>
        ) : (
          <div className="py-6 text-center">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/10 text-accent">
              <QrCode className="h-8 w-8" />
            </div>
            <p className="mt-4 text-sm font-medium text-fg">
              Receive {profile.symbol}
            </p>
            <p className="mt-1 text-xs text-fg-muted">
              Get an address to copy, or create a QR payment request below.
            </p>
          </div>
        )}
      </section>

      <button
        type="button"
        onClick={() => setShowDetails((v) => !v)}
        className="flex min-h-[44px] w-full items-center justify-between gap-2 rounded-xl border border-border bg-bg-subtle/60 px-4 py-3 text-sm font-medium text-fg active:bg-bg-subtle"
        aria-expanded={showDetails}
      >
        <span>Payment request details</span>
        <span className="flex items-center gap-2 text-xs text-fg-muted">
          Optional
          <ChevronDown
            className={cn('h-4 w-4 transition-transform', showDetails && 'rotate-180')}
          />
        </span>
      </button>

      {showDetails && (
        <section className="mobile-panel space-y-4 rounded-2xl border border-border bg-bg-panel/60 p-4">
          <div>
            <label htmlFor="mobile-receive-label" className="mobile-field-label">
              Label
            </label>
            <input
              id="mobile-receive-label"
              type="text"
              value={label}
              onChange={(e) => onLabelChange(e.target.value)}
              placeholder="Who is this for?"
              className="mobile-input mt-1.5 w-full"
            />
          </div>
          <div>
            <label htmlFor="mobile-receive-amount" className="mobile-field-label">
              Amount ({profile.symbol})
            </label>
            <input
              id="mobile-receive-amount"
              type="text"
              inputMode="decimal"
              value={amount}
              onChange={(e) => onAmountChange(e.target.value)}
              placeholder="Optional fixed amount"
              className="mobile-input mt-1.5 w-full tabular-nums"
            />
          </div>
          <div>
            <label htmlFor="mobile-receive-message" className="mobile-field-label">
              Message
            </label>
            <textarea
              id="mobile-receive-message"
              rows={2}
              value={message}
              onChange={(e) => onMessageChange(e.target.value)}
              placeholder="Shown when the QR is scanned"
              className="mobile-textarea mt-1.5 w-full"
            />
          </div>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="text-fg-muted"
            onClick={onClearForm}
          >
            Clear fields
          </Button>
        </section>
      )}

      {requests.length > 0 && (
        <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/40 p-4">
          <h3 className="text-sm font-semibold text-fg">Recent requests</h3>
          <p className="mt-0.5 text-xs text-fg-muted">Tap to show QR again</p>
          <ul className="mt-3 flex flex-col gap-2">
            {requests.slice(0, 8).map((row) => {
              const isActive = selected?.id === row.id;
              return (
                <li key={row.id}>
                  <button
                    type="button"
                    onClick={() => onSelectRequest(row.id)}
                    className={cn(
                      'flex min-h-[52px] w-full items-center justify-between gap-3 rounded-xl border px-3 py-2.5 text-left transition-colors active:opacity-90',
                      isActive ? 'border-accent/40 bg-accent/10' : 'border-border bg-bg-subtle/50'
                    )}
                  >
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium">
                        {row.label || 'Payment request'}
                      </p>
                      <p className="mt-0.5 truncate font-mono text-[11px] text-fg-muted">
                        {row.address}
                      </p>
                    </div>
                    <span className="shrink-0 text-sm font-semibold tabular-nums">
                      {row.amount != null ? formatCoinAmount(row.amount, coin, 4) : '—'}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        </section>
      )}

      {createError && (
        <div className="rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {createError}
        </div>
      )}

      <div className="mobile-send-sticky-bar flex flex-col gap-2">
        <Button
          type="button"
          variant="secondary"
          size="lg"
          className="h-12 min-h-[48px] w-full rounded-xl text-base font-semibold"
          disabled={busy}
          onClick={onCreateAddressOnly}
        >
          {creatingAddress ? (
            <>
              <Loader2 className="h-5 w-5 animate-spin" aria-hidden />
              Generating…
            </>
          ) : (
            <>
              <Copy className="h-5 w-5" />
              Get address only
            </>
          )}
        </Button>
        <Button
          type="button"
          size="lg"
          className="h-12 min-h-[48px] w-full rounded-xl text-base font-semibold"
          disabled={busy}
          onClick={onCreate}
        >
          {creating ? (
            <>
              <Loader2 className="h-5 w-5 animate-spin" aria-hidden />
              Generating…
            </>
          ) : (
            <>
              <QrCode className="h-5 w-5" />
              {active ? 'New QR code' : 'Show QR code'}
            </>
          )}
        </Button>
      </div>
    </div>
  );
}
