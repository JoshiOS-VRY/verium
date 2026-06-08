import { useState } from "react";
import type { CoinId } from "@/lib/coin/profile";
import { Users } from "lucide-react";
import { Button } from "@/components/ui/Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { ExplorerLink } from "@/components/ExplorerLink";
import { ExplorerPeersPanel } from "@/components/ExplorerPeersPanel";
import { NetworkLocalPeersCard } from "@/components/NetworkLocalPeersCard";
import {
  explorerExtractionHash,
  explorerPeersHash,
  explorerRichlistHash,
} from "@/lib/explorer-links";
import type { PeerInfo } from "@/lib/rpc/client";
import { cn } from "@/lib/utils";

type PeersTab = "connected" | "discover";

export function NetworkPeersSection({
  coin,
  peers,
  explorerEnabled,
}: {
  coin: CoinId;
  peers?: PeerInfo[];
  explorerEnabled: boolean;
}) {
  const [tab, setTab] = useState<PeersTab>("connected");
  const peerCount = peers?.length ?? 0;

  return (
    <Card>
      <CardHeader className="gap-4 space-y-0">
        <div className="flex min-w-0 flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <CardTitle className="flex items-center gap-2 normal-case">
              <Users className="h-4 w-4 text-accent" aria-hidden />
              Peers
            </CardTitle>
            <CardDescription className="mt-1">
              {peerCount > 0
                ? `${peerCount} live connection${peerCount === 1 ? "" : "s"} from your node.`
                : "No live connections — discover peers below or check firewall / P2P port."}
            </CardDescription>
          </div>
          <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs">
            <ExplorerLink
              coin={coin}
              target={{ kind: "raw", url: explorerPeersHash(coin) }}
              label="Explorer peers"
            />
            {coin === "verium" && (
              <>
                <ExplorerLink
                  coin={coin}
                  target={{ kind: "raw", url: explorerExtractionHash(coin) }}
                  label="Extraction"
                />
                <ExplorerLink
                  coin={coin}
                  target={{ kind: "raw", url: explorerRichlistHash(coin) }}
                  label="Rich list"
                />
              </>
            )}
          </div>
        </div>

        {explorerEnabled && (
          <div
            role="tablist"
            aria-label="Peer views"
            className="inline-flex rounded-lg border border-border bg-bg-subtle p-1"
          >
            <Button
              type="button"
              role="tab"
              aria-selected={tab === "connected"}
              variant={tab === "connected" ? "primary" : "ghost"}
              className={cn("h-8 px-3 text-sm")}
              onClick={() => setTab("connected")}
            >
              Your connections
              {peerCount > 0 ? ` (${peerCount})` : ""}
            </Button>
            <Button
              type="button"
              role="tab"
              aria-selected={tab === "discover"}
              variant={tab === "discover" ? "primary" : "ghost"}
              className={cn("h-8 px-3 text-sm")}
              onClick={() => setTab("discover")}
            >
              Discover peers
            </Button>
          </div>
        )}
      </CardHeader>

      <CardContent className="p-0 pt-0">
        {tab === "connected" || !explorerEnabled ? (
          <NetworkLocalPeersCard coin={coin} peers={peers} embedded />
        ) : (
          <ExplorerPeersPanel embedded />
        )}
      </CardContent>
    </Card>
  );
}
