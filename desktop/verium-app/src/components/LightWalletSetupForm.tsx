import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { RecoveryPhraseWizard } from '@/components/RecoveryPhraseWizard';
import { TwoFactorPrompt } from '@/components/TwoFactorPrompt';
import { useTwoFactorGate } from '@/hooks/useTwoFactorGate';
import { useActiveCoin } from '@/lib/coin/context';
import { invalidateLightWalletQueries } from '@/lib/invalidate-wallet-queries';
import {
  lightWalletCreate,
  lightWalletImport,
  lightWalletUnlock,
  walletModeSetForCoin,
} from '@/lib/light-wallet/client';
import { onboardingMarkComplete } from '@/lib/wallet-profile';
import { scorePassphrase } from '@/lib/passphrase-strength';
import { cn } from '@/lib/utils';

interface LightWalletSetupFormProps {
  mode: 'create' | 'import' | 'unlock';
  importPhrase?: string;
  onDone: () => void;
  onBack?: () => void;
}

function formatMutationError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  return 'Something went wrong. Check your passphrase and try again.';
}

export function LightWalletSetupForm({
  mode,
  importPhrase,
  onDone,
  onBack,
}: LightWalletSetupFormProps) {
  const coin = useActiveCoin();
  const queryClient = useQueryClient();
  const twoFa = useTwoFactorGate(coin);
  const [passphrase, setPassphrase] = useState('');
  const [confirm, setConfirm] = useState('');
  const [phrase, setPhrase] = useState(importPhrase ?? '');
  const [phase, setPhase] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [createStep, setCreateStep] = useState<'passphrase' | 'phrase'>('passphrase');

  const score = scorePassphrase(passphrase);
  const matches = passphrase.length > 0 && passphrase === confirm;
  const passphraseOk = passphrase.trim().length > 0;
  const recoveryMaterialOk = mode !== 'import' || phrase.trim().length > 0;

  const action = useMutation({
    mutationFn: async (opts?: { mnemonic?: string; totpCode?: string }) => {
      setError(null);
      if (mode === 'unlock') {
        setPhase('Unlocking wallet…');
        await lightWalletUnlock(coin, passphrase);
        return;
      }
      const mnemonicToUse = (opts?.mnemonic ?? phrase).trim();
      setPhase(mode === 'import' ? 'Importing wallet…' : 'Encrypting wallet…');
      if (mode === 'import') {
        await lightWalletImport(coin, mnemonicToUse, passphrase, undefined, opts?.totpCode);
      } else {
        await lightWalletCreate(coin, mnemonicToUse, passphrase);
      }
    },
    onSuccess: async () => {
      setPhase('');
      setError(null);
      await invalidateLightWalletQueries(queryClient, coin);
      if (mode !== 'unlock') {
        await walletModeSetForCoin(coin, 'light').catch(() => undefined);
        await onboardingMarkComplete(coin).catch(() => undefined);
      }
      onDone();
    },
    onError: (err) => {
      setPhase('');
      setError(formatMutationError(err));
    },
  });

  const disabled =
    mode === 'unlock'
      ? !passphraseOk || action.isPending
      : !matches || !passphraseOk || !recoveryMaterialOk || action.isPending;

  if (mode === 'create' && createStep === 'phrase') {
    return (
      <div className="flex flex-col gap-3">
        <RecoveryPhraseWizard
          onComplete={async (verifiedPhrase) => {
            await action.mutateAsync({ mnemonic: verifiedPhrase });
          }}
        />
        <Button
          type="button"
          size="sm"
          variant="ghost"
          className="self-start"
          onClick={() => setCreateStep('passphrase')}
        >
          Back
        </Button>
      </div>
    );
  }

  const submitForm = () => {
    if (disabled) return;
    if (mode === 'create' && createStep === 'passphrase') {
      setCreateStep('phrase');
      return;
    }
    if (mode === 'import') {
      void twoFa.gate('restore_wallet', (code) => action.mutate({ totpCode: code }), {
        title: 'Confirm light wallet import with 2FA',
      });
      return;
    }
    action.mutate({});
  };

  return (
    <>
      <TwoFactorPrompt
        open={twoFa.open}
        title={twoFa.title}
        onVerified={twoFa.verified}
        onCancel={twoFa.cancel}
      />
      <form
        className="flex flex-col gap-3"
        onSubmit={(e) => {
          e.preventDefault();
          submitForm();
        }}
      >
        {mode === 'import' && (
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
        {mode !== 'unlock' && (
          <input
            type="password"
            className={cn(
              'rounded border bg-bg px-3 py-2 text-sm',
              confirm.length > 0 && !matches ? 'border-danger' : 'border-border'
            )}
            placeholder="Confirm passphrase"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        )}
        {mode === 'import' && phrase.trim().length === 0 && (
          <p className="text-xs text-fg-muted">
            Paste your 24-word recovery phrase or HD master key from Security → Export. Your
            existing full-node wallet on this device cannot be opened directly in light mode —
            import the same recovery material you used there.
          </p>
        )}
        {mode === 'create' && passphrase.length > 0 && score.score < 2 && (
          <p className="text-[11px] text-fg-subtle">Tip: {score.hint}</p>
        )}
        {mode !== 'unlock' && confirm.length > 0 && !matches && (
          <p className="text-xs text-danger">Passphrases do not match.</p>
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
            ? 'Working…'
            : mode === 'unlock'
              ? 'Unlock'
              : mode === 'create'
                ? 'Continue to recovery phrase'
                : 'Import light wallet'}
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
    </>
  );
}
