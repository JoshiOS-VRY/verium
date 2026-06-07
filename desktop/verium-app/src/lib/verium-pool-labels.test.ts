import { describe, expect, it } from "vitest";
import {
  isVeriumPoolMinerAddress,
  resolveVeriumMinerExplorerAddress,
  VERIUM_POOL_DISPLAY_NAME,
} from "./verium-pool-labels";
import { VERIUM_POOL_PAYOUT_ADDRESS } from "./verium-pool";

describe("verium pool miner labels", () => {
  it("recognizes payout address and display name", () => {
    expect(isVeriumPoolMinerAddress(VERIUM_POOL_PAYOUT_ADDRESS)).toBe(true);
    expect(isVeriumPoolMinerAddress(VERIUM_POOL_DISPLAY_NAME)).toBe(true);
    expect(isVeriumPoolMinerAddress("VLZEz6CBem7XpqbEm9tik9rLi7uQggccu5")).toBe(
      false,
    );
  });

  it("maps display name to payout address for explorer links", () => {
    expect(resolveVeriumMinerExplorerAddress(VERIUM_POOL_DISPLAY_NAME)).toBe(
      VERIUM_POOL_PAYOUT_ADDRESS,
    );
    expect(
      resolveVeriumMinerExplorerAddress(VERIUM_POOL_PAYOUT_ADDRESS),
    ).toBe(VERIUM_POOL_PAYOUT_ADDRESS);
  });
});
