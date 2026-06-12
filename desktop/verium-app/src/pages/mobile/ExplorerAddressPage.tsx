import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useParams } from 'react-router-dom';
import { AddressActivityChart } from '@/components/mobile/explorer/AddressActivityChart';
import { CumulativeBalanceChart } from '@/components/mobile/explorer/CumulativeBalanceChart';
import { AddressActivitySection } from '@/components/mobile/explorer/AddressActivitySection';
import { AddressBalanceHero } from '@/components/mobile/explorer/AddressBalanceHero';
import { AddressCompositionCard } from '@/components/mobile/explorer/AddressCompositionCard';
import { AddressStatsStrip } from '@/components/mobile/explorer/AddressStatsStrip';
import { ExplorerDetailShell } from '@/components/mobile/explorer/ExplorerDetailShell';
import { shortExplorerAddress } from '@/components/mobile/explorer/tx-detail-utils';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { buildActivityBars, buildAddressComposition } from '@/lib/address-activity';
import { cumulativeSeriesCaption, indexerPointsToChart } from '@/lib/cumulative-balance';
import { buildAddressExplorerUrl, effectiveAddressExplorerTemplate } from '@/lib/explorer-links';
import { fetchIndexerAddress, fetchIndexerAddressCumulativeSeries } from '@/lib/indexer-api';
import { indexerTicker, parseIndexerAmountCoins } from '@/lib/indexer-amount';
import { useUserPreferences } from '@/lib/user-preferences';

const PAGE_SIZE = 20;
const CHART_SAMPLE = 36;

export function ExplorerAddressPage() {
  const { address: rawAddress } = useParams();
  const address = rawAddress?.trim() ?? '';
  const coin = useActiveCoin();
  const { prefs } = useUserPreferences();
  const addressTemplate = effectiveAddressExplorerTemplate(
    coin,
    prefs.explorer_address_url_template
  );
  const [offset, setOffset] = useState(0);

  useEffect(() => {
    setOffset(0);
  }, [address]);

  const detail = useQuery({
    queryKey: coinQueryKey(coin, 'indexer-address', address, offset),
    queryFn: () => fetchIndexerAddress(coin, address, PAGE_SIZE, offset),
    enabled: address.length > 0,
  });

  const chartSample = useQuery({
    queryKey: coinQueryKey(coin, 'indexer-address-chart', address),
    queryFn: () => fetchIndexerAddress(coin, address, CHART_SAMPLE, 0),
    enabled: address.length > 0,
    staleTime: 60_000,
  });

  const cumulative = useQuery({
    queryKey: coinQueryKey(coin, 'indexer-address-cumulative', address),
    queryFn: () => fetchIndexerAddressCumulativeSeries(coin, address),
    enabled: address.length > 0,
    staleTime: 120_000,
  });

  if (!address) {
    return (
      <ExplorerDetailShell title="Address" subtitle="Missing address.">
        <p className="px-3 text-sm text-fg-muted">
          Go back and open an address from a transaction.
        </p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isLoading) {
    return (
      <ExplorerDetailShell title="Address" subtitle="Loading from explorer index…">
        <p className="px-3 text-sm text-fg-muted">Fetching address details…</p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isError || !detail.data) {
    return (
      <ExplorerDetailShell
        title="Address"
        subtitle="Could not load this address."
        externalUrl={buildAddressExplorerUrl(coin, addressTemplate, address)}
      >
        <p className="px-3 text-sm text-danger">
          {detail.error instanceof Error ? detail.error.message : 'Indexer request failed.'}
        </p>
      </ExplorerDetailShell>
    );
  }

  const data = detail.data;
  const balance = data.balance;
  const externalUrl = buildAddressExplorerUrl(coin, addressTemplate, address);
  const txs = data.transactions ?? [];
  const paging = data.paging;
  const chartTxs = chartSample.data?.transactions ?? txs;
  const ticker = indexerTicker(balance?.balance) || indexerTicker(balance?.totalReceived) || 'VRM';

  if (!data.found) {
    return (
      <ExplorerDetailShell
        title="Address not indexed"
        subtitle="No indexed activity for this address yet."
        externalUrl={externalUrl}
      >
        <p className="px-3 font-mono text-xs break-all text-fg-muted">{address}</p>
      </ExplorerDetailShell>
    );
  }

  const composition = buildAddressComposition(
    parseIndexerAmountCoins(balance?.totalReceived),
    parseIndexerAmountCoins(balance?.totalSent),
    parseIndexerAmountCoins(balance?.balance),
    ticker
  );
  const activityBars = buildActivityBars(chartTxs, CHART_SAMPLE);
  const cumulativePoints = indexerPointsToChart(cumulative.data?.points ?? []);
  const cumulativeCaption = cumulative.data
    ? cumulativeSeriesCaption(
        cumulative.data.complete,
        cumulative.data.txCountUsed,
        cumulative.data.txCountTotal
      )
    : cumulative.isLoading
      ? 'Loading full address history…'
      : undefined;

  return (
    <ExplorerDetailShell
      title="Address"
      subtitle={shortExplorerAddress(address, 16, 12)}
      externalUrl={externalUrl}
    >
      <AddressBalanceHero address={address} balance={balance?.balance} />

      <AddressStatsStrip balance={balance} richlist={data.richlist} />

      <AddressCompositionCard composition={composition} />

      <CumulativeBalanceChart
        points={cumulativePoints}
        ticker={ticker}
        coin={coin}
        anchorBalance={parseIndexerAmountCoins(balance?.balance)}
        caption={cumulativeCaption}
      />

      <AddressActivityChart bars={activityBars} sampleBalance={balance?.balance} />

      <AddressActivitySection
        transactions={txs}
        paging={paging}
        offset={offset}
        pageSize={PAGE_SIZE}
        onOffsetChange={setOffset}
      />

      {data.source?.label && (
        <p className="px-2 text-center text-[10px] text-fg-subtle">
          Index source: {data.source.label}
          {data.trusted ? ' · trusted' : ''}
        </p>
      )}
    </ExplorerDetailShell>
  );
}
