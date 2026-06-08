import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  HardDriveUpload,
  Loader2,
  ShieldCheck,
  FolderInput,
  Wallet as WalletIcon,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { COIN_PROFILES, coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { rpcSetConfig, tauriRestartDaemon } from "@/lib/rpc/client";
import { recoveryApplyHdSeed } from "@/lib/security/client";
import {
  legacyDatadirCandidate,
  legacyRequestHdUpgrade,
} from "@/lib/wallet-profile";
import { WalletUnlockForm } from "@/components/WalletUnlockForm";
import { WalletImportForm } from "@/components/WalletImportForm";
import { RecoveryPhraseWizard } from "@/components/RecoveryPhraseWizard";

type LegacyStep =
  | "explain"
  | "choose_source"
  | "unlock"
  | "secure"
  | "recovery";

interface LegacyUpgradeWizardProps {
  coin: CoinId;
  /** Detected legacy wallet.dat path (from the wallet profile). */
  legacyPath?: string | null;
  /** True once the node RPC is reachable so wallet RPCs can run. */
  connected: boolean;
  /** Called when the wallet is upgraded (or the user defers) and onboarding can continue. */
  onComplete: () => void;
  /** User chose to ignore the legacy wallet and create a fresh one. */
  onStartFresh: () => void;
  onBack: () => void;
}

/**
 * Guided upgrade for users with an older Verium-Qt / Vericoin `wallet.dat`.
 *
 * Flow: explain -> adopt the existing data folder (or import the file) ->
 * unlock with the original passphrase -> add a BIP39 recovery phrase (the
 * capability old passphrase-only wallets lacked) -> continue. Funds, addresses,
 * and history are preserved throughout.
 */
export function LegacyUpgradeWizard({
  coin,
  legacyPath,
  connected,
  onComplete,
  onStartFresh,
  onBack,
}: LegacyUpgradeWizardProps) {
  const profile = COIN_PROFILES[coin];
  const queryClient = useQueryClient();
  const [step, setStep] = useState<LegacyStep>("explain");
  const [error, setError] = useState<string | null>(null);

  const adoptDatadir = useMutation({
    mutationFn: async () => {
      const candidate = await legacyDatadirCandidate(coin);
      if (!candidate) {
        throw new Error(
          "Could not locate the legacy data folder. Use Import wallet.dat instead.",
        );
      }
      await rpcSetConfig(coin, { datadir: candidate });
      await tauriRestartDaemon(coin);
      return candidate;
    },
    onSuccess: () => {
      setError(null);
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "wallet-file-status"),
      });
      setStep("unlock");
    },
    onError: (e) => setError(String(e)),
  });

  const startHdUpgrade = useMutation({
    mutationFn: async () => {
      // Restart with -upgradewallet so a pre-HD wallet.dat can accept sethdseed.
      await legacyRequestHdUpgrade(coin);
    },
    onSuccess: () => {
      setError(null);
      setStep("recovery");
    },
    onError: (e) => setError(String(e)),
  });

  const applyRecovery = useMutation({
    mutationFn: (phrase: string) => recoveryApplyHdSeed(coin, phrase),
    onSuccess: async () => {
      setError(null);
      await queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "wallet-is-hd"),
      });
      onComplete();
    },
    onError: (e) => setError(String(e)),
  });

  return (
    <div className="flex flex-col gap-4 text-sm">
      {error && (
        <p className="rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
          {error}
        </p>
      )}

      {step === "explain" && (
        <div className="flex flex-col gap-4 text-fg-muted">
          <div className="flex items-center gap-2 text-fg">
            <WalletIcon className="h-4 w-4 text-accent" />
            Existing {profile.displayName} wallet found
          </div>
          <p>
            We detected an older {profile.displayName} wallet on this computer.
            You can keep using it — your coins, addresses, and history are
            preserved. After unlocking, we will help you add a recovery phrase,
            which older passphrase-only wallets did not have.
          </p>
          {legacyPath && (
            <p className="break-all rounded-md border border-accent/30 bg-accent/5 px-3 py-2 font-mono text-[11px] text-fg">
              {legacyPath}
            </p>
          )}
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => setStep("choose_source")}>
              Use my existing wallet
            </Button>
            <Button variant="secondary" onClick={onStartFresh}>
              Start fresh instead
            </Button>
            <Button variant="ghost" onClick={onBack}>
              Back
            </Button>
          </div>
          <p className="text-xs text-fg-subtle">
            Starting fresh creates a brand-new wallet. Your old funds stay in the
            old wallet file and will not appear here.
          </p>
        </div>
      )}

      {step === "choose_source" && (
        <div className="flex flex-col gap-3">
          <p className="text-fg-muted">
            How should we open your existing wallet?
          </p>
          <button
            type="button"
            disabled={adoptDatadir.isPending}
            onClick={() => adoptDatadir.mutate()}
            className="flex flex-col gap-1 rounded-md border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent disabled:opacity-60"
          >
            <span className="flex items-center gap-2 font-medium text-fg">
              {adoptDatadir.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin text-accent" />
              ) : (
                <FolderInput className="h-4 w-4 text-accent" />
              )}
              Use my existing data folder (recommended)
            </span>
            <span className="text-xs text-fg-muted">
              Point the node at your current {profile.displayName} folder. No
              copying — fastest, keeps your full history.
            </span>
          </button>
          <button
            type="button"
            onClick={() => setStep("unlock")}
            className="flex flex-col gap-1 rounded-md border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent"
          >
            <span className="flex items-center gap-2 font-medium text-fg">
              <HardDriveUpload className="h-4 w-4 text-accent" />
              Import a wallet.dat file
            </span>
            <span className="text-xs text-fg-muted">
              Copy a backup into this app's data folder (a safety backup of any
              current wallet is made first).
            </span>
          </button>
          <Button variant="ghost" className="self-start" onClick={() => setStep("explain")}>
            Back
          </Button>
        </div>
      )}

      {step === "unlock" && (
        <div className="flex flex-col gap-3">
          {!connected && (
            <p className="rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-fg-muted">
              Waiting for the node to load your wallet…
            </p>
          )}
          <WalletImportForm
            onRestored={() => {
              void queryClient.invalidateQueries({
                queryKey: coinQueryKey(coin, "wallet-file-status"),
              });
            }}
          />
          <div className="border-t border-border pt-3">
            <WalletUnlockForm
              title="Unlock your existing wallet"
              description={`Enter the passphrase from your previous ${profile.displayName} wallet. Your balance and history are preserved.`}
              onUnlocked={() => setStep("secure")}
            />
          </div>
        </div>
      )}

      {step === "secure" && (
        <div className="flex flex-col gap-4">
          <div className="flex items-center gap-2 text-fg">
            <ShieldCheck className="h-4 w-4 text-accent" />
            Add a recovery phrase
          </div>
          <p className="text-fg-muted">
            Your old wallet used a passphrase only. A recovery phrase is a
            24-word backup that lets you restore on another device if you lose
            this computer or its <span className="font-mono text-xs">wallet.dat</span>.
            We will upgrade your wallet so it can store one.
          </p>
          <div className="flex flex-wrap gap-2">
            <Button
              disabled={startHdUpgrade.isPending}
              onClick={() => startHdUpgrade.mutate()}
            >
              {startHdUpgrade.isPending ? (
                <span className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" /> Upgrading wallet…
                </span>
              ) : (
                "Add recovery phrase"
              )}
            </Button>
            <Button variant="ghost" onClick={onComplete}>
              I'll do this later
            </Button>
          </div>
          <p className="text-xs text-fg-subtle">
            Deferring is allowed, but without a recovery phrase your only backup
            is the wallet file plus its passphrase.
          </p>
        </div>
      )}

      {step === "recovery" && (
        <div className="flex flex-col gap-4">
          <div className="flex items-center gap-2 text-fg">
            <ShieldCheck className="h-4 w-4 text-accent" />
            Save your recovery phrase
          </div>
          <p className="text-fg-muted">
            Write these 24 words down and store them offline. Vericonomy cannot
            recover them for you.
          </p>
          <RecoveryPhraseWizard
            onComplete={async (phrase) => {
              await applyRecovery.mutateAsync(phrase);
            }}
          />
          {applyRecovery.isPending && (
            <p className="flex items-center gap-2 text-xs text-fg-muted">
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Applying recovery seed to your wallet…
            </p>
          )}
        </div>
      )}
    </div>
  );
}
