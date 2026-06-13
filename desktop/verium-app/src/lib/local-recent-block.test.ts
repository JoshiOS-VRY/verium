import { describe, expect, it } from 'vitest';
import { mergeRecentBlocks, parseRpcBlock } from '@/lib/local-recent-block';
import type { ExplorerBlock } from '@/lib/explorer-api';

function block(height: number, hash: string): ExplorerBlock {
  return { id: height, hash, height, time: height };
}

describe('mergeRecentBlocks', () => {
  it('prepends a local block not yet on the explorer feed', () => {
    const explorer = [block(100, 'a'), block(99, 'b')];
    const local = [
      {
        ...block(101, 'new'),
        miner_address: 'Vabc',
        output_total: '1.5',
      },
    ];
    const merged = mergeRecentBlocks(explorer, local, 10);
    expect(merged[0]?.height).toBe(101);
    expect(merged[0]?.miner_address).toBe('Vabc');
  });

  it('enriches an explorer row at the same height with local miner metadata', () => {
    const explorer = [{ ...block(100, 'a'), miner_address: undefined }];
    const local = [{ ...block(100, 'a'), miner_address: 'Vyours' }];
    const merged = mergeRecentBlocks(explorer, local, 10);
    expect(merged[0]?.miner_address).toBe('Vyours');
  });

  it('merges placeholder and indexed rows at the same height', () => {
    const stub = {
      id: 1103326,
      hash: 'light-pending-1103326',
      height: 1103326,
      time: 1_700_000_000,
    };
    const indexed = {
      id: 1103326,
      hash: 'abc123realhash',
      height: 1103326,
      time: 1_700_001_000,
      miner_address: 'VMiner',
      output_total: '1.5',
      n_tx: 2,
    };
    const merged = mergeRecentBlocks([stub], [indexed], 10);
    expect(merged).toHaveLength(1);
    expect(merged[0]?.hash).toBe('abc123realhash');
    expect(merged[0]?.miner_address).toBe('VMiner');
    expect(merged[0]?.time).toBe(1_700_001_000);
  });

  it('dedupes when height is numeric in one feed and string-like in another', () => {
    const explorer = [{ ...block(100, 'a'), height: 100 as number }];
    const local = [{ ...block(100, 'b'), height: '100' as unknown as number }];
    const merged = mergeRecentBlocks(explorer, local, 10);
    expect(merged).toHaveLength(1);
  });

  it('parses coinbase reward and miner from getblock verbosity 2', () => {
    const row = parseRpcBlock('verium', 100, 'abc', {
      height: 100,
      hash: 'abc',
      time: 1_700_000_000,
      nTx: 1,
      size: 177,
      difficulty: 0.00008,
      tx: [
        {
          vin: [{ coinbase: '00' }],
          vout: [
            {
              value: 1.3135,
              scriptPubKey: { address: 'VMinerAddress123456789' },
            },
          ],
        },
      ],
    });

    expect(row.output_total).toBe('1.3135');
    expect(row.miner_address).toBe('VMinerAddress123456789');
    expect(row.n_tx).toBe(1);
    expect(row.size).toBe(177);
  });
});
