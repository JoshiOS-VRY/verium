import { useState } from "react";

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { AlertTriangle } from "lucide-react";

import { Button } from "@/components/ui/Button";

import { LightWalletImportProgress } from "@/components/LightWalletImportProgress";

import { TwoFactorPrompt } from "@/components/TwoFactorPrompt";

import { useActiveCoin } from "@/lib/coin/context";

import { coinQueryKey } from "@/lib/coin/profile";

import { useTwoFactorGate } from "@/hooks/useTwoFactorGate";

import { lightWalletImport } from "@/lib/light-wallet/client";

import { lightWalletCopy } from "@/lib/light-wallet/copy";

import { scorePassphrase } from "@/lib/passphrase-strength";

import { WALLET_MODE_QUERY_KEY } from "@/hooks/useWalletMode";



interface LightWalletImportFormProps {

  onSuccess?: () => void;

}



export function LightWalletImportForm({ onSuccess }: LightWalletImportFormProps) {

  const coin = useActiveCoin();

  const queryClient = useQueryClient();

  const twoFa = useTwoFactorGate(coin);

  const [seed, setSeed] = useState("");

  const [passphrase, setPassphrase] = useState("");

  const [confirm, setConfirm] = useState("");

  const [error, setError] = useState<string | null>(null);

  const [syncingBalance, setSyncingBalance] = useState(false);



  const importWallet = useMutation({

    mutationFn: (totpCode?: string) =>
      lightWalletImport(coin, seed.trim(), passphrase, undefined, totpCode),

    onSuccess: async () => {

      setError(null);

      setSyncingBalance(true);

      try {

        await Promise.all([

          queryClient.invalidateQueries({

            queryKey: coinQueryKey(coin, "getwalletinfo"),

          }),

          queryClient.invalidateQueries({

            queryKey: coinQueryKey(coin, "light-server-status"),

          }),

          queryClient.invalidateQueries({

            queryKey: coinQueryKey(coin, "light-wallet-exists"),

          }),

          queryClient.invalidateQueries({ queryKey: WALLET_MODE_QUERY_KEY }),

        ]);

        await queryClient.refetchQueries({

          queryKey: coinQueryKey(coin, "getwalletinfo"),

        });

      } finally {

        setSyncingBalance(false);

        setSeed("");

        setPassphrase("");

        setConfirm("");

        onSuccess?.();

      }

    },

    onError: (err) => {

      setSyncingBalance(false);

      setError(String(err));

    },

  });



  const score = scorePassphrase(passphrase);

  const matches = passphrase.length > 0 && passphrase === confirm;

  const isBusy = importWallet.isPending || syncingBalance;

  const canSubmit =

    seed.trim().length > 0 &&

    matches &&

    score.score >= 2 &&

    !isBusy;



  const progressPhase = importWallet.isPending

    ? "importing"

    : syncingBalance

      ? "syncing"

      : importWallet.isSuccess

        ? "done"

        : "done";



  const submit = () => {

    importWallet.reset();

    setError(null);

    void twoFa.gate(

      "restore_wallet",

      (code) => importWallet.mutate(code),

      { title: "Confirm light wallet import with 2FA" },

    );

  };



  return (

    <>

      <TwoFactorPrompt

        open={twoFa.open}

        title={twoFa.title}

        onVerified={twoFa.verified}

        onCancel={twoFa.cancel}

      />

      <div className="flex flex-col gap-3">

        <div className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-xs text-warning">

          <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />

          <p>{lightWalletCopy.importLightBody}</p>

        </div>



        {isBusy && <LightWalletImportProgress phase={progressPhase} />}



        {!isBusy && (

          <>

            <textarea

              className="min-h-[96px] rounded-md border border-border bg-bg px-3 py-2 font-mono text-xs outline-none focus:border-accent"

              placeholder="24-word recovery phrase, or HD master key from Security → Export"

              value={seed}

              onChange={(e) => {

                setSeed(e.target.value);

                if (error) setError(null);

              }}

              spellCheck={false}

              autoComplete="off"

            />



            <input

              type="password"

              autoComplete="new-password"

              className="h-9 rounded-md border border-border bg-bg px-3 text-sm outline-none focus:border-accent"

              placeholder="New wallet passphrase"

              value={passphrase}

              onChange={(e) => setPassphrase(e.target.value)}

            />

            <input

              type="password"

              autoComplete="new-password"

              className="h-9 rounded-md border border-border bg-bg px-3 text-sm outline-none focus:border-accent"

              placeholder="Confirm passphrase"

              value={confirm}

              onChange={(e) => setConfirm(e.target.value)}

            />



            {passphrase.length > 0 && confirm.length > 0 && !matches && (

              <p className="text-xs text-danger">Passphrases do not match.</p>

            )}

          </>

        )}



        {!isBusy && (
          <Button size="sm" disabled={!canSubmit} onClick={submit}>
            Import light wallet
          </Button>
        )}



        {error && <p className="text-xs text-danger">{error}</p>}

        {importWallet.isSuccess && !isBusy && (

          <div className="rounded-md border border-success/40 bg-success/10 px-3 py-2 text-xs text-success">

            <p className="font-medium">Import complete</p>

            <p className="mt-1 text-fg-muted">{lightWalletCopy.importLightSuccess}</p>

            <p className="mt-1 text-fg-muted">{lightWalletCopy.importLightScanNote}</p>

            <a href="/" className="mt-2 inline-block text-accent underline">

              Open dashboard

            </a>

          </div>

        )}

      </div>

    </>

  );

}


