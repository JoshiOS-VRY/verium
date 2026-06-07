import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { AlertTriangle, Copy, Eye, EyeOff, Lock } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { TwoFactorPrompt } from "@/components/TwoFactorPrompt";
import { useActiveCoin } from "@/lib/coin/context";
import { useTwoFactorGate } from "@/hooks/useTwoFactorGate";
import { useWalletMode } from "@/hooks/useWalletMode";
import {
  recoveryExportSeed,
  type RecoveryExportResult,
} from "@/lib/security/client";

export function ExportRecoveryPhrasePanel() {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();
  const twoFa = useTwoFactorGate(coin);
  const [passphrase, setPassphrase] = useState("");
  const [revealed, setRevealed] = useState(false);
  const [exported, setExported] = useState<RecoveryExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const exportSeed = useMutation({
    mutationFn: (totpCode?: string) =>
      recoveryExportSeed(coin, passphrase, totpCode),
    onSuccess: (result) => {
      setExported(result);
      setError(null);
      setRevealed(false);
    },
    onError: (e) => {
      setExported(null);
      setError(String(e));
    },
  });

  const secretText =
    exported?.kind === "mnemonic"
      ? exported.mnemonic
      : exported?.kind === "hd_master_xprv"
        ? exported.xprv
        : "";

  const words =
    exported?.kind === "mnemonic"
      ? exported.mnemonic.split(/\s+/).filter(Boolean)
      : [];

  const startExport = () => {
    if (!passphrase.trim()) {
      setError("Enter your wallet passphrase first.");
      return;
    }
    void twoFa.gate("show_recovery_phrase", (code) => exportSeed.mutate(code), {
      title: "Confirm export with 2FA",
    });
  };

  const prompt = (
    <TwoFactorPrompt
      open={twoFa.open}
      title={twoFa.title}
      onVerified={twoFa.verified}
      onCancel={twoFa.cancel}
    />
  );

  return (
    <>
      {prompt}
      <div className="flex flex-col gap-3 rounded-md border border-border bg-bg-subtle p-4">
        <div className="flex items-start gap-2 text-sm text-fg-muted">
          <Lock className="mt-0.5 h-4 w-4 shrink-0 text-accent" />
          <p>
            {isLight
              ? "Decrypt and view your light wallet seed. Anyone with this material can spend your coins."
              : "View your BIP39 phrase if it was saved when you upgraded to HD, or export the HD master key for light-wallet import."}
          </p>
        </div>

        {!exported && (
          <>
            <input
              type="password"
              autoComplete="current-password"
              value={passphrase}
              onChange={(e) => {
                setPassphrase(e.target.value);
                setError(null);
              }}
              placeholder="Wallet passphrase (required)"
              className="h-9 rounded-md border border-border bg-bg px-3 text-sm outline-none focus:border-accent"
            />
            <Button
              size="sm"
              variant="secondary"
              onClick={startExport}
              disabled={exportSeed.isPending || !passphrase.trim()}
            >
              {exportSeed.isPending ? "Decrypting…" : "Export recovery material"}
            </Button>
          </>
        )}

        {error && (
          <p className="flex items-start gap-1.5 text-xs text-danger">
            <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
            {error}
          </p>
        )}

        {exported?.kind === "mnemonic" && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-fg">
                Your {exported.word_count}-word recovery phrase
              </span>
              <button
                type="button"
                onClick={() => setRevealed((v) => !v)}
                className="flex items-center gap-1 text-xs text-fg-muted hover:text-fg"
              >
                {revealed ? (
                  <EyeOff className="h-3.5 w-3.5" />
                ) : (
                  <Eye className="h-3.5 w-3.5" />
                )}
                {revealed ? "Hide" : "Reveal"}
              </button>
            </div>
            <div
              className={`grid grid-cols-3 gap-2 rounded-lg border border-border p-4 ${
                revealed ? "bg-bg" : "bg-bg blur-sm select-none"
              }`}
            >
              {words.map((word, i) => (
                <div key={i} className="text-xs">
                  <span className="text-fg-subtle">{i + 1}.</span> {word}
                </div>
              ))}
            </div>
            {revealed && (
              <Button
                size="sm"
                variant="secondary"
                onClick={() => {
                  void navigator.clipboard.writeText(secretText);
                  window.setTimeout(
                    () => void navigator.clipboard.writeText(""),
                    30_000,
                  );
                }}
              >
                <Copy className="h-3.5 w-3.5" />
                Copy phrase (clears in 30s)
              </Button>
            )}
          </div>
        )}

        {exported?.kind === "hd_master_xprv" && (
          <div className="flex flex-col gap-3">
            <p className="text-xs text-warning">{exported.message}</p>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-fg">HD master key</span>
              <button
                type="button"
                onClick={() => setRevealed((v) => !v)}
                className="flex items-center gap-1 text-xs text-fg-muted hover:text-fg"
              >
                {revealed ? (
                  <EyeOff className="h-3.5 w-3.5" />
                ) : (
                  <Eye className="h-3.5 w-3.5" />
                )}
                {revealed ? "Hide" : "Reveal"}
              </button>
            </div>
            <div
              className={`break-all rounded-lg border border-border p-3 font-mono text-xs ${
                revealed ? "bg-bg" : "bg-bg blur-sm select-none"
              }`}
            >
              {exported.xprv}
            </div>
            <p className="text-xs text-fg-muted">
              In light-wallet setup or Settings → Wallet backup, choose Import and
              paste this key instead of a 24-word phrase.
            </p>
            {revealed && (
              <Button
                size="sm"
                variant="secondary"
                onClick={() => {
                  void navigator.clipboard.writeText(secretText);
                  window.setTimeout(
                    () => void navigator.clipboard.writeText(""),
                    30_000,
                  );
                }}
              >
                <Copy className="h-3.5 w-3.5" />
                Copy HD master key (clears in 30s)
              </Button>
            )}
          </div>
        )}

        {exported && (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              setExported(null);
              setPassphrase("");
              setRevealed(false);
              setError(null);
            }}
          >
            Done — clear from screen
          </Button>
        )}
      </div>
    </>
  );
}
