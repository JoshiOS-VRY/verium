import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { WalletUnlockForm } from "@/components/WalletUnlockForm";
import { coinQueryKey } from "@/lib/coin/profile";
import { useActiveCoin } from "@/lib/coin/context";
import { rpcGetWalletInfo } from "@/lib/rpc/client";
import { lightWalletExists } from "@/lib/light-wallet/client";
import { useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { COIN_PROFILES } from "@/lib/coin/profile";
import { isWalletLocked } from "@/lib/wallet-unlock";

interface WalletUnlockGateProps {
  children: ReactNode;
  title?: string;
  description?: string;
  mintingOnly?: boolean;
}

export function WalletUnlockGate({
  children,
  title,
  description,
  mintingOnly,
}: WalletUnlockGateProps) {
  const coin = useActiveCoin();
  const profile = COIN_PROFILES[coin];
  const { isLight } = useWalletMode();
  const storedLightWallet = useQuery({
    queryKey: coinQueryKey(coin, "light-wallet-exists"),
    queryFn: () => lightWalletExists(coin),
  });
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  if (wallet.isLoading) {
    return (
      <Card>
        <CardContent className="py-10 text-center text-sm text-fg-muted">
          Loading wallet…
        </CardContent>
      </Card>
    );
  }

  if (!wallet.data) {
    const hasStoredLightWallet = storedLightWallet.data === true;
    const modeMismatch = !isLight && hasStoredLightWallet;
    return (
      <Card>
        <CardHeader>
          <CardTitle>
            {modeMismatch
              ? lightWalletCopy.modeMismatchTitle
              : isLight
                ? lightWalletCopy.unlockUnavailableTitle
                : "Wallet unavailable"}
          </CardTitle>
          <CardDescription>
            {modeMismatch
              ? lightWalletCopy.modeMismatchDescription.replace(
                  "{coin}",
                  profile.displayName,
                )
              : isLight
                ? lightWalletCopy.unlockUnavailableDescription
                : "Connect to your node and ensure a wallet is loaded before using this page."}
          </CardDescription>
        </CardHeader>
        {(modeMismatch || (isLight && !hasStoredLightWallet)) && (
          <CardContent>
            <Link to="/settings" className="text-sm text-accent underline">
              {modeMismatch
                ? lightWalletCopy.modeMismatchCta
                : lightWalletCopy.unlockUnavailableCta}
            </Link>
          </CardContent>
        )}
      </Card>
    );
  }

  if (isWalletLocked(wallet.data)) {
    return (
      <Card>
        <CardContent className="py-6">
          <WalletUnlockForm
            title={title}
            description={description}
            mintingOnly={mintingOnly}
          />
        </CardContent>
      </Card>
    );
  }

  return <>{children}</>;
}
