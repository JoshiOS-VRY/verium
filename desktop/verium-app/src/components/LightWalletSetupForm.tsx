import { useState } from "react";

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Loader2 } from "lucide-react";

import { Button } from "@/components/ui/Button";

import { RecoveryPhraseWizard } from "@/components/RecoveryPhraseWizard";

import { useActiveCoin } from "@/lib/coin/context";

import { coinQueryKey } from "@/lib/coin/profile";

import { lightWalletCreate, lightWalletUnlock } from "@/lib/light-wallet/client";

import { scorePassphrase } from "@/lib/passphrase-strength";



interface LightWalletSetupFormProps {

  mode: "create" | "import" | "unlock";

  importPhrase?: string;

  onDone: () => void;

  onBack?: () => void;

}



function formatMutationError(err: unknown): string {

  if (err instanceof Error) return err.message;

  if (typeof err === "string") return err;

  return "Something went wrong. Check your passphrase and try again.";

}



export function LightWalletSetupForm({

  mode,

  importPhrase,

  onDone,

  onBack,

}: LightWalletSetupFormProps) {

  const coin = useActiveCoin();

  const queryClient = useQueryClient();

  const [passphrase, setPassphrase] = useState("");

  const [confirm, setConfirm] = useState("");

  const [phrase, setPhrase] = useState(importPhrase ?? "");

  const [phase, setPhase] = useState("");

  const [error, setError] = useState<string | null>(null);

  const [createStep, setCreateStep] = useState<"passphrase" | "phrase">(

    "passphrase",

  );



  const score = scorePassphrase(passphrase);

  const matches = passphrase.length > 0 && passphrase === confirm;



  const action = useMutation({

    mutationFn: async (mnemonic?: string) => {

      setError(null);

      if (mode === "unlock") {

        setPhase("Unlocking wallet…");

        await lightWalletUnlock(coin, passphrase);

        return;

      }

      const mnemonicToUse = (mnemonic ?? phrase).trim();

      setPhase("Encrypting wallet…");

      await lightWalletCreate(coin, mnemonicToUse, passphrase);

    },

    onSuccess: async () => {

      setPhase("");

      setError(null);

      await queryClient.invalidateQueries({

        queryKey: coinQueryKey(coin, "getwalletinfo"),

      });

      await queryClient.invalidateQueries({

        queryKey: coinQueryKey(coin, "light-server-status"),

      });

      await queryClient.invalidateQueries({ queryKey: ["wallet-mode-status"] });

      onDone();

    },

    onError: (err) => {

      setPhase("");

      setError(formatMutationError(err));

    },

  });



  const disabled =

    mode === "unlock"

      ? !passphrase.trim() || action.isPending

      : !matches ||

        score.score < 2 ||

        (mode === "import" && !phrase.trim()) ||

        action.isPending;



  if (mode === "create" && createStep === "phrase") {

    return (

      <div className="flex flex-col gap-3">

        <RecoveryPhraseWizard

          onComplete={async (verifiedPhrase) => {

            await action.mutateAsync(verifiedPhrase);

          }}

        />

        <Button

          type="button"

          size="sm"

          variant="ghost"

          className="self-start"

          onClick={() => setCreateStep("passphrase")}

        >

          Back

        </Button>

      </div>

    );

  }



  return (

    <form

      className="flex flex-col gap-3"

      onSubmit={(e) => {

        e.preventDefault();

        if (mode === "create" && createStep === "passphrase" && !disabled) {

          setCreateStep("phrase");

          return;

        }

        if (!disabled) action.mutate(undefined);

      }}

    >

      {mode === "import" && (

        <textarea

          className="min-h-[96px] rounded border border-border bg-bg px-3 py-2 text-sm"

          placeholder="24-word recovery phrase, or HD master key from Security → Export"

          value={phrase}

          onChange={(e) => setPhrase(e.target.value)}

        />

      )}

      <input

        type="password"

        autoComplete="off"

        className="rounded border border-border bg-bg px-3 py-2 text-sm"

        placeholder="Wallet passphrase"

        value={passphrase}

        onChange={(e) => {

          setPassphrase(e.target.value);

          if (error) setError(null);

        }}

      />

      {mode !== "unlock" && (

        <input

          type="password"

          className="rounded border border-border bg-bg px-3 py-2 text-sm"

          placeholder="Confirm passphrase"

          value={confirm}

          onChange={(e) => setConfirm(e.target.value)}

        />

      )}

      {phase && (

        <p className="flex items-center gap-2 text-xs text-fg-muted">

          <Loader2 className="h-3 w-3 animate-spin" />

          {phase}

        </p>

      )}

      {error && <p className="text-xs text-danger">{error}</p>}

      <Button type="submit" disabled={disabled}>

        {action.isPending

          ? "Working…"

          : mode === "unlock"

            ? "Unlock"

            : mode === "create"

              ? "Continue to recovery phrase"

              : "Create light wallet"}

      </Button>

      {onBack && (

        <Button

          type="button"

          size="sm"

          variant="ghost"

          className="self-start"

          onClick={onBack}

          disabled={action.isPending}

        >

          Back

        </Button>

      )}

    </form>

  );

}


