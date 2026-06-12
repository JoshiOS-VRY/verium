import { useQuery } from '@tanstack/react-query';
import { Calendar, Hash, Layers } from 'lucide-react';
import { useParams } from 'react-router-dom';
import { ExplorerDetailShell } from '@/components/mobile/explorer/ExplorerDetailShell';
import { ExplorerInternalLink } from '@/components/mobile/explorer/ExplorerInternalLink';
import { TransactionFlowCard } from '@/components/mobile/explorer/TransactionFlowCard';
import {
  TransactionInputsSection,
  TransactionOutputsSection,
} from '@/components/mobile/explorer/TransactionIoSection';
import { WalletImpactCard } from '@/components/mobile/explorer/WalletImpactCard';
import { shortTxid } from '@/components/mobile/explorer/tx-detail-utils';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { buildTxExplorerUrl, effectiveTxExplorerTemplate } from '@/lib/explorer-links';
import { fetchIndexerTransaction } from '@/lib/indexer-api';
import { explorerBlockPath } from '@/lib/explorer-nav';
import { useUserPreferences } from '@/lib/user-preferences';

export function ExplorerTransactionPage() {
  const { txid: rawTxid } = useParams();
  const txid = rawTxid?.trim() ?? '';
  const coin = useActiveCoin();
  const { prefs } = useUserPreferences();
  const txTemplate = effectiveTxExplorerTemplate(coin, prefs.explorer_tx_url_template);

  const detail = useQuery({
    queryKey: coinQueryKey(coin, 'indexer-tx', txid),
    queryFn: () => fetchIndexerTransaction(coin, txid),
    enabled: txid.length > 0,
  });

  if (!txid) {
    return (
      <ExplorerDetailShell title="Transaction" subtitle="Missing transaction id.">
        <p className="px-3 text-sm text-fg-muted">
          Go back and open a transaction from your history.
        </p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isLoading) {
    return (
      <ExplorerDetailShell title="Transaction" subtitle="Loading from explorer index…">
        <p className="px-3 text-sm text-fg-muted">Fetching transaction details…</p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isError || !detail.data) {
    return (
      <ExplorerDetailShell
        title="Transaction"
        subtitle="Could not load this transaction."
        externalUrl={buildTxExplorerUrl(coin, txTemplate, txid)}
      >
        <p className="px-3 text-sm text-danger">
          {detail.error instanceof Error ? detail.error.message : 'Indexer request failed.'}
        </p>
      </ExplorerDetailShell>
    );
  }

  const data = detail.data;
  const tx = data.transaction;
  const externalUrl = buildTxExplorerUrl(coin, txTemplate, txid);
  const inputs = data.inputs ?? [];
  const outputs = data.outputs ?? [];
  const addressEvents = data.addressEvents ?? [];

  if (!data.found || !tx) {
    return (
      <ExplorerDetailShell
        title="Transaction not indexed"
        subtitle="This txid is not in the explorer index yet."
        externalUrl={externalUrl}
      >
        <p className="px-3 font-mono text-xs break-all text-fg-muted">{txid}</p>
      </ExplorerDetailShell>
    );
  }

  const when = tx.time ? new Date(tx.time * 1000).toLocaleString() : '—';
  const flags = [tx.isCoinbase ? 'Coinbase' : null, tx.isCoinstake ? 'Coinstake' : null]
    .filter(Boolean)
    .join(' · ');

  return (
    <ExplorerDetailShell
      title="Transaction"
      subtitle={flags || shortTxid(txid)}
      externalUrl={externalUrl}
    >
      <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
        <dl className="space-y-3 text-xs">
          <div className="flex items-start gap-3">
            <Calendar className="mt-0.5 h-4 w-4 shrink-0 text-fg-muted" aria-hidden />
            <div>
              <dt className="text-fg-muted">Confirmed</dt>
              <dd className="mt-0.5 font-medium text-fg">{when}</dd>
            </div>
          </div>
          {tx.blockHeight != null && (
            <div className="flex items-start gap-3">
              <Layers className="mt-0.5 h-4 w-4 shrink-0 text-fg-muted" aria-hidden />
              <div>
                <dt className="text-fg-muted">Block</dt>
                <dd className="mt-0.5">
                  <ExplorerInternalLink to={explorerBlockPath(tx.blockHeight)}>
                    #{tx.blockHeight.toLocaleString()}
                  </ExplorerInternalLink>
                </dd>
              </div>
            </div>
          )}
          <div className="flex items-start gap-3">
            <Hash className="mt-0.5 h-4 w-4 shrink-0 text-fg-muted" aria-hidden />
            <div className="min-w-0">
              <dt className="text-fg-muted">Transaction id</dt>
              <dd className="mt-0.5 font-mono text-[11px] leading-relaxed break-all text-fg">
                {txid}
              </dd>
            </div>
          </div>
        </dl>
      </section>

      <WalletImpactCard events={addressEvents} />

      <TransactionFlowCard
        inputs={inputs}
        outputs={outputs}
        isCoinbase={tx.isCoinbase}
        isCoinstake={tx.isCoinstake}
      />

      <TransactionInputsSection inputs={inputs} />
      <TransactionOutputsSection outputs={outputs} />
    </ExplorerDetailShell>
  );
}
