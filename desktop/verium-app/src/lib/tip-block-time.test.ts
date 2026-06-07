import { describe, expect, it } from "vitest";
import { resolveTipBlockTime } from "./tip-block-time";

describe("resolveTipBlockTime", () => {
  const tipHeight = 1_101_412;
  const tipTime = 1_700_000_000;
  const staleMediantime = 1_699_998_000;

  it("prefers chain tip watcher when height matches", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight, hash: "abc", time: tipTime },
        explorerBlocks: [{ height: tipHeight, time: tipTime + 10, hash: "x" }],
        headerTime: staleMediantime,
      }),
    ).toBe(tipTime);
  });

  it("uses explorer block at tip height when watcher is stale", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight - 1, hash: "old", time: staleMediantime },
        explorerBlocks: [{ height: tipHeight, time: tipTime, hash: "new" }],
        headerTime: staleMediantime,
      }),
    ).toBe(tipTime);
  });

  it("falls back to getblockheader time", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: null,
        explorerBlocks: [],
        headerTime: tipTime,
      }),
    ).toBe(tipTime);
  });

  it("returns undefined instead of mediantime-like stale values", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight - 5, hash: "old", time: staleMediantime },
        explorerBlocks: [{ height: tipHeight - 3, time: staleMediantime, hash: "x" }],
        headerTime: null,
      }),
    ).toBeUndefined();
  });
});
