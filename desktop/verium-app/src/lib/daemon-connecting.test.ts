import { describe, expect, it } from "vitest";
import { isDaemonConnectingState } from "@/lib/daemon-connecting";

describe("isDaemonConnectingState", () => {
  const baseOpts = {
    isLoading: false,
    isFetching: false,
    startupGraceActive: false,
  };

  it("stays connecting while backend reports starting phase", () => {
    expect(
      isDaemonConnectingState(
        {
          connected: false,
          daemon_phase: "starting",
          error: "Starting node…",
        },
        baseOpts,
      ),
    ).toBe(true);
  });

  it("stays connecting for friendly starting error without startup grace", () => {
    expect(
      isDaemonConnectingState(
        {
          connected: false,
          error: "Starting node…",
        },
        baseOpts,
      ),
    ).toBe(true);
  });

  it("surfaces auth mismatch as not connecting", () => {
    expect(
      isDaemonConnectingState(
        {
          connected: false,
          error: "RPC unauthorized",
        },
        baseOpts,
      ),
    ).toBe(false);
  });
});
