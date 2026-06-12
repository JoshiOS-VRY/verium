import { Check, Copy, Wallet } from 'lucide-react';
import { QRCodeSVG } from 'qrcode.react';
import { Button } from '@/components/ui/Button';
import { useCopyToClipboard } from '@/hooks/useCopyToClipboard';
import { useCoinProfile } from '@/lib/coin/context';
import { formatIndexerAmount, type IndexerAmount } from '@/lib/indexer-api';
import { cn } from '@/lib/utils';
import { shortExplorerAddress } from './tx-detail-utils';

export function AddressBalanceHero({
  address,
  balance,
}: {
  address: string;
  balance?: IndexerAmount | null;
}) {
  const profile = useCoinProfile();
  const { copied, copy } = useCopyToClipboard();

  return (
    <section className="mobile-panel overflow-hidden rounded-2xl border border-border bg-gradient-to-br from-bg-panel via-bg-panel to-accent/5 p-4 shadow-sm">
      <div className="flex items-start gap-4">
        <div className="shrink-0 rounded-xl border border-border bg-white p-2 shadow-sm">
          <QRCodeSVG value={address} size={88} level="M" includeMargin={false} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <Wallet className="h-4 w-4 text-accent" aria-hidden />
            <p className="text-[11px] font-medium uppercase tracking-wide text-fg-subtle">
              Indexed balance
            </p>
          </div>
          <p className="mt-1 text-2xl font-bold tabular-nums tracking-tight text-fg">
            {formatIndexerAmount(balance ?? undefined)}
          </p>
          <p className="text-xs text-fg-muted">{profile.displayName} address</p>
        </div>
      </div>

      <div className="mt-4 flex items-center gap-2 rounded-xl border border-border/70 bg-bg-subtle/40 px-3 py-2">
        <p className="min-w-0 flex-1 font-mono text-[11px] leading-relaxed break-all text-fg-muted">
          {shortExplorerAddress(address, 14, 10)}
        </p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className={cn(
            'h-8 shrink-0 rounded-lg px-2.5',
            copied && 'border-success/40 text-success'
          )}
          onClick={() => void copy(address)}
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
          <span className="text-[11px]">{copied ? 'Copied' : 'Copy'}</span>
        </Button>
      </div>
    </section>
  );
}
