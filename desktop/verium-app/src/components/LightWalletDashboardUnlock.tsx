import { useQuery } from "@tanstack/react-query";
import { Lock } from "lucide-react";
import { Card, CardContent } from "@/components/ui/Card";
import { WalletUnlockForm } from "@/components/WalletUnlockForm";
import { useActiveCoin } from "@/lib/coin/context";
import { coinQueryKey } from "@/lib/coin/profile";
import { useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletExists } from "@/lib/light-wallet/client";
import { rpcGetWalletInfo } from "@/lib/rpc/client";
import { isWalletLocked } from "@/lib/wallet-unlock";

/** Prompt to unlock the light wallet so balance and history can load from Electrum. */
export function LightWalletDashboardUnlock() {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();
  const exists = useQuery({
    queryKey: coinQueryKey(coin, "light-wallet-exists"),
    queryFn: () => lightWalletExists(coin),
    enabled: isLight,
  });
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: isLight && exists.data === true,
    refetchInterval: false,
  });

  if (!isLight || exists.data !== true) return null;
  if (wallet.isLoading) return null;

  const locked = !wallet.data || isWalletLocked(wallet.data);
  if (!locked) return null;

  return (
    <Card className="border-accent/30 bg-accent/5">
      <CardContent className="flex flex-col gap-3 py-4">
        <div className="flex items-center gap-2 text-sm font-medium text-fg">
          <Lock className="h-4 w-4 text-accent" />
          Unlock your light wallet to send and view transactions
        </div>
        <p className="text-xs text-fg-muted">
          Your keys stay on this device. Unlocking is instant; balance may
          already be visible. A background scan on Vericonomy servers refreshes
          history after you unlock.
        </p>
        <WalletUnlockForm
          title="Unlock light wallet"
          description="Enter your wallet passphrase."
          className="gap-3"
        />
      </CardContent>
    </Card>
  );
}
