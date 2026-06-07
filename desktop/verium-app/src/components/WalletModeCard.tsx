import { useState } from "react";
import { Link } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Cloud, HardDrive, Loader2, Server } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Badge } from "@/components/ui/Badge";
import { useActiveCoin } from "@/lib/coin/context";
import { useInvalidateWalletMode, useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { electrumUriFriendlyLabel } from "@/lib/light-wallet/labels";
import { coinQueryKey } from "@/lib/coin/profile";
import {
  electrumServersGet,
  electrumServersSet,
  electrumTestConnection,
  type WalletMode,
  walletModeGet,
  walletModeSet,
} from "@/lib/light-wallet/client";

export function WalletModeCard() {
  const { lightWalletEnabled } = useWalletMode();

  if (!lightWalletEnabled) {
    return null;
  }

  return <WalletModeCardInner />;
}

function WalletModeCardInner() {
  const activeCoin = useActiveCoin();
  const queryClient = useQueryClient();
  const invalidateWalletMode = useInvalidateWalletMode();
  const [customServers, setCustomServers] = useState("");
  const [confirmSwitch, setConfirmSwitch] = useState<WalletMode | null>(null);

  const modeStatus = useQuery({
    queryKey: ["wallet-mode-status"],
    queryFn: walletModeGet,
  });

  const servers = useQuery({
    queryKey: ["electrum-servers", activeCoin],
    queryFn: () => electrumServersGet(activeCoin),
    enabled: modeStatus.data?.mode === "light",
  });

  const setMode = useMutation({
    mutationFn: walletModeSet,
    onSuccess: () => {
      invalidateWalletMode();
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(activeCoin, "getwalletinfo"),
      });
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(activeCoin, "light-server-status"),
      });
      setConfirmSwitch(null);
    },
  });

  const saveServers = useMutation({
    mutationFn: (list: string[]) => electrumServersSet(activeCoin, list),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["electrum-servers", activeCoin] });
    },
  });

  const testConn = useMutation({
    mutationFn: () => electrumTestConnection(activeCoin),
  });

  const activeMode = modeStatus.data?.mode ?? "full_node";
  const isLight = activeMode === "light";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Wallet mode
          <Badge tone={isLight ? "accent" : "neutral"}>
            {isLight ? "Light" : "Full node"}
          </Badge>
        </CardTitle>
        <CardDescription>
          Full node mode (recommended) validates the chain locally. Light mode is
          a convenience tier that trusts Electrum index servers for balance data;
          keys still sign on this device.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {isLight && (
          <p className="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-xs text-warning">
            {lightWalletCopy.lightModeBanner}
          </p>
        )}
        <div className="grid gap-3 sm:grid-cols-2">
          <button
            type="button"
            className={`rounded-lg border p-3 text-left text-sm transition-colors ${
              !isLight
                ? "border-accent bg-accent/10"
                : "border-border hover:border-accent/50"
            }`}
            onClick={() => setConfirmSwitch("full_node")}
          >
            <div className="mb-1 flex items-center gap-2 font-medium">
              <HardDrive className="h-4 w-4" />
              Full node (recommended)
            </div>
            <p className="text-xs text-fg-muted">
              {lightWalletCopy.setupFullNodeRecommended}
            </p>
          </button>
          <button
            type="button"
            className={`rounded-lg border p-3 text-left text-sm transition-colors ${
              isLight
                ? "border-warning bg-warning/10"
                : "border-border hover:border-accent/50"
            }`}
            onClick={() => setConfirmSwitch("light")}
          >
            <div className="mb-1 flex items-center gap-2 font-medium">
              <Cloud className="h-4 w-4" />
              Light wallet (convenience)
            </div>
            <p className="text-xs text-fg-muted">
              {lightWalletCopy.lightConvenienceWarning}
            </p>
          </button>
        </div>

        {confirmSwitch && confirmSwitch !== activeMode && (
          <div className="rounded-md border border-warning/40 bg-warning/10 p-3 text-xs">
            <p className="mb-1 font-medium text-fg">
              {confirmSwitch === "light"
                ? lightWalletCopy.migrationFullToLightTitle
                : lightWalletCopy.migrationLightToFullTitle}
            </p>
            <p className="mb-2 text-fg-muted">
              {confirmSwitch === "light"
                ? lightWalletCopy.migrationFullToLightBody
                : lightWalletCopy.migrationLightToFullBody}
            </p>
            <p className="mb-2 text-fg-muted">
              <Link to="/security" className="text-accent underline">
                Open Security
              </Link>{" "}
              to export or import your recovery phrase before confirming.
            </p>
            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={() => setMode.mutate(confirmSwitch)}
                disabled={setMode.isPending}
              >
                {setMode.isPending && (
                  <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                )}
                Confirm switch
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setConfirmSwitch(null)}
              >
                Cancel
              </Button>
            </div>
          </div>
        )}

        {isLight && (
          <div className="flex flex-col gap-3 rounded-md border border-border bg-bg-subtle p-3 text-xs">
            <div className="flex items-center gap-2 font-medium text-fg">
              <Server className="h-3.5 w-3.5" />
              Light wallet servers
            </div>
            <ul className="flex flex-col gap-1.5 text-fg-muted">
              {(servers.data ?? []).map((s, i) => (
                <li key={s} className="flex flex-col gap-0.5">
                  <span className="font-medium text-fg">
                    {electrumUriFriendlyLabel(s, activeCoin, i)}
                  </span>
                  <span className="break-all font-mono text-[11px] text-fg-subtle">
                    {s}
                  </span>
                </li>
              ))}
            </ul>
            <textarea
              className="min-h-[72px] w-full rounded border border-border bg-bg px-2 py-1 font-mono text-[11px]"
              placeholder="Custom servers (comma-separated tls://host:port)"
              value={customServers}
              onChange={(e) => setCustomServers(e.target.value)}
            />
            <div className="flex flex-wrap gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() => {
                  const list = customServers
                    .split(",")
                    .map((s) => s.trim())
                    .filter(Boolean);
                  if (list.length > 0) saveServers.mutate(list);
                }}
                disabled={saveServers.isPending}
              >
                Save custom servers
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => testConn.mutate()}
                disabled={testConn.isPending}
              >
                {testConn.isPending ? "Testing…" : "Test connection"}
              </Button>
            </div>
            {testConn.data && (
              <div className="flex flex-col gap-2">
                {(() => {
                  const blocking = testConn.data.checks.filter(
                    (c) => !c.ok && !c.optional,
                  );
                  return (
                <p
                  className={
                    testConn.data.passed ? "text-success" : "text-danger"
                  }
                >
                  {testConn.data.server}:{" "}
                  {testConn.data.passed ? "OK" : "Failed"}
                  {blocking.length > 0
                    ? ` (${blocking.length} issue${blocking.length === 1 ? "" : "s"})`
                    : ""}
                </p>
                  );
                })()}
                {!testConn.data.passed && (
                  <ul className="list-inside list-disc space-y-1 text-fg-muted">
                    {testConn.data.checks
                      .filter((c) => !c.ok && !c.optional)
                      .map((c) => (
                        <li key={c.method}>
                          <span className="font-mono text-fg">{c.method}</span>
                          {c.detail && c.detail !== "ok" ? (
                            <span className="text-danger"> — {c.detail}</span>
                          ) : null}
                        </li>
                      ))}
                  </ul>
                )}
                <details className="text-fg-subtle">
                  <summary className="cursor-pointer text-[11px]">
                    All checks ({testConn.data.checks.length})
                  </summary>
                  <ul className="mt-1 list-inside list-disc space-y-0.5 font-mono text-[11px]">
                    {testConn.data.checks.map((c) => (
                      <li
                        key={c.method}
                        className={
                          c.ok
                            ? "text-success"
                            : c.optional
                              ? "text-warning"
                              : "text-danger"
                        }
                      >
                        {c.ok ? "✓" : c.optional ? "○" : "✗"} {c.method}
                        {c.detail && c.detail !== "ok" ? ` (${c.detail})` : ""}
                      </li>
                    ))}
                  </ul>
                </details>
              </div>
            )}
            {testConn.error && (
              <p className="text-danger text-xs">
                {String(testConn.error)}
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
