import { describe, expect, it } from "vitest";
import { isPlaceholderTipHash, resolveTipBlockTime } from "./tip-block-time";

describe("isPlaceholderTipHash", () => {
  it("detects light and local pending stubs", () => {
    expect(isPlaceholderTipHash("light-tip-1101462")).toBe(true);
    expect(isPlaceholderTipHash("abc123")).toBe(false);
  });
});

describe("resolveTipBlockTime", () => {
  const tipHeight = 1_101_412;
  const tipTime = 1_700_000_000;
  const staleMediantime = 1_699_998_000;

  it("prefers explorer at tip height (matches recent-blocks table)", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight, hash: "abc", time: tipTime },
        explorerBlocks: [{ id: tipHeight, height: tipHeight, time: tipTime + 10, hash: "x" }],
        headerTime: staleMediantime,
      }),
    ).toBe(tipTime + 10);
  });

  it("ignores placeholder watcher times when explorer has the tip row", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: {
          height: tipHeight,
          hash: "light-tip-1101412",
          time: tipTime + 24,
        },
        explorerBlocks: [{ id: tipHeight, height: tipHeight, time: tipTime, hash: "x" }],
      }),
    ).toBe(tipTime);
  });

  it("uses chain tip watcher when explorer has not indexed tip height yet", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight, hash: "abc", time: tipTime },
        explorerBlocks: [{ id: tipHeight - 1, height: tipHeight - 1, time: staleMediantime, hash: "old" }],
        headerTime: staleMediantime,
      }),
    ).toBe(tipTime);
  });

  it("uses explorer block at tip height when watcher is stale", () => {
    expect(
      resolveTipBlockTime(tipHeight, {
        chainTip: { height: tipHeight - 1, hash: "old", time: staleMediantime },
        explorerBlocks: [{ id: tipHeight, height: tipHeight, time: tipTime, hash: "new" }],
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
        explorerBlocks: [{ id: tipHeight - 3, height: tipHeight - 3, time: staleMediantime, hash: "x" }],
        headerTime: null,
      }),
    ).toBeUndefined();
  });
});
