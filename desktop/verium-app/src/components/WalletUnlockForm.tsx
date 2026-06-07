import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, Loader2, Lock } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { coinQueryKey } from "@/lib/coin/profile";
import { useActiveCoin } from "@/lib/coin/context";
import { useInvalidateWalletMode, useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletExists, lightWalletUnlock } from "@/lib/light-wallet/client";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { rpcGetWalletInfo, rpcWalletUnlock } from "@/lib/rpc/client";
import { passkeyStatus } from "@/lib/security/client";
import {
  optimisticLightWalletUnlockPatch,
  rpcUnlockTimeoutSeconds,
  shouldUnlockMintingOnly,
} from "@/lib/wallet-unlock";
import { cn } from "@/lib/utils";

interface WalletUnlockFormProps {
  title?: string;
  description?: string;
  onUnlocked?: () => void;
  mintingOnly?: boolean;
  className?: string;
  submitDisabled?: boolean;
  submitDisabledMessage?: string;
}

function formatUnlockError(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  const text = String(error).trim();
  return text && text !== "[object Object]"
    ? text
    : "Unlock failed — check your light-wallet passphrase (from import), not your full-node wallet passphrase.";
}

export function WalletUnlockForm({
  title = "Unlock wallet",
  description = "Enter your wallet passphrase to continue. Your passphrase is never stored.",
  onUnlocked,
  mintingOnly = false,
  className,
  submitDisabled = false,
  submitDisabledMessage,
}: WalletUnlockFormProps) {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();
  const invalidateWalletMode = useInvalidateWalletMode();
  const queryClient = useQueryClient();
  const passkey = useQuery({ queryKey: ["passkey"], queryFn: passkeyStatus });
  const storedLightWallet = useQuery({
    queryKey: coinQueryKey(coin, "light-wallet-exists"),
    queryFn: () => lightWalletExists(coin),
  });
  const [passphrase, setPassphrase] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<"idle" | "verifying" | "applied">("idle");

  const useLightUnlock =
    isLight || storedLightWallet.data === true;

  const unlock = useMutation({
    mutationFn: async () => {
      setStatus("verifying");
      if (useLightUnlock) {
        await lightWalletUnlock(coin, passphrase, rpcUnlockTimeoutSeconds());
        return;
      }
      await rpcWalletUnlock(
        coin,
        passphrase,
        rpcUnlockTimeoutSeconds(),
        shouldUnlockMintingOnly(coin, mintingOnly) ? true : undefined,
      );
    },
    onSuccess: async () => {
      setPassphrase("");
      setError(null);

      if (useLightUnlock) {
        setStatus("applied");
        queryClient.setQueryData(
          coinQueryKey(coin, "getwalletinfo"),
          (prev) =>
            optimisticLightWalletUnlockPatch(
              prev as Awaited<ReturnType<typeof rpcGetWalletInfo>>,
              coin,
              rpcUnlockTimeoutSeconds(),
            ),
        );
        invalidateWalletMode();
      } else {
        setStatus("idle");
      }

      if (!useLightUnlock) {
        void queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, "getwalletinfo"),
        });
      }
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "light-wallet-exists"),
      });
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "light-server-status"),
      });
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "listtransactions"),
      });
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "listunspent"),
      });
      void queryClient.invalidateQueries({ queryKey: ["wallet-mode-status"] });

      if (useLightUnlock) {
        setTimeout(() => setStatus("idle"), 1500);
      }

      onUnlocked?.();
    },
    onError: (e) => {
      setStatus("idle");
      setError(formatUnlockError(e));
    },
  });

  const busy = unlock.isPending || status === "applied";

  return (
    <div className={cn("flex flex-col gap-4", className)}>
      <div className="flex items-start gap-3">
        <div className="rounded-lg border border-border bg-bg-subtle p-2.5">
          <Lock className="h-5 w-5 text-accent" />
        </div>
        <div>
          <h2 className="text-lg font-semibold">{title}</h2>
          <p className="mt-1 text-sm text-fg-muted">{description}</p>
          {mintingOnly && coin === "vericoin" && (
            <p className="mt-1 text-xs text-fg-subtle">
              Stake-only unlock — coins stay locked for sending until you unlock
              fully.
            </p>
          )}
        </div>
      </div>

      <form
        className="flex flex-col gap-4"
        onSubmit={(e) => {
          e.preventDefault();
          if (submitDisabled || busy) return;
          if (passphrase) unlock.mutate();
        }}
      >
        {passkey.data?.enabled && (
          <p className="text-xs text-fg-subtle">
            App PIN is enrolled — use the PIN gate at launch. Enter your wallet
            passphrase here to unlock signing and sending.
          </p>
        )}

        <div className="flex flex-col gap-1 text-sm">
          <label className="text-fg-muted">Passphrase</label>
          <input
            type="password"
            autoComplete="off"
            spellCheck={false}
            autoFocus
            disabled={busy}
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            className="h-10 rounded-md border border-border bg-bg-subtle px-3 text-sm text-fg outline-none focus:border-accent disabled:opacity-60"
            placeholder="Wallet passphrase"
          />
        </div>

        {status === "applied" && (
          <div className="flex items-start gap-2 rounded-md border border-success/30 bg-success/10 px-3 py-2 text-xs text-fg-muted">
            <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 shrink-0 text-success" />
            <span>
              <strong className="font-medium text-fg">
                {lightWalletCopy.unlockApplied}
              </strong>
              {" — "}
              {lightWalletCopy.unlockRefreshingBalance}
            </span>
          </div>
        )}

        {unlock.isPending && useLightUnlock && status === "verifying" && (
          <div className="flex items-center gap-2 text-xs text-fg-muted">
            <Loader2 className="h-3.5 w-3.5 animate-spin text-accent" />
            {lightWalletCopy.unlockVerifying}
          </div>
        )}

        {error && <div className="text-xs text-danger">{error}</div>}
        {submitDisabled && submitDisabledMessage && (
          <div className="text-xs text-fg-muted">{submitDisabledMessage}</div>
        )}

        <Button
          type="submit"
          disabled={!passphrase || busy || submitDisabled}
          className="self-start"
        >
          {unlock.isPending
            ? useLightUnlock
              ? lightWalletCopy.unlockVerifying
              : "Unlocking…"
            : status === "applied"
              ? lightWalletCopy.unlockApplied
              : "Unlock wallet"}
        </Button>
      </form>
    </div>
  );
}
