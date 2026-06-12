import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { ScanFace } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useActiveCoin } from '@/lib/coin/context';
import { useWalletMode } from '@/hooks/useWalletMode';
import {
  biometricUnlockEnable,
  biometryLabel,
  promptBiometricUnlock,
  type BiometricStatus,
} from '@/lib/biometric/client';
import { biometricUnlockQueryKey, useBiometricUnlockStatus } from '@/hooks/useBiometricUnlock';
import { shouldOfferBiometricSetup, useBiometricSetupOffer } from '@/lib/biometric/setup-offer';
import { useUserPreferences } from '@/lib/user-preferences';

function formatBiometricError(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  const text = String(error).trim();
  return text && text !== '[object Object]' ? text : 'Could not enable biometric unlock.';
}

interface BiometricSetupPromptViewProps {
  label: string;
  busy: boolean;
  error: string | null;
  onEnable: () => void;
  onDismiss: () => void;
}

/** iOS-style permission sheet for enabling Face ID / Touch ID unlock. */
export function BiometricSetupPromptView({
  label,
  busy,
  error,
  onEnable,
  onDismiss,
}: BiometricSetupPromptViewProps) {
  return (
    <div
      className="fixed inset-0 z-[70] flex items-end justify-center bg-black/45 p-4 backdrop-blur-[2px] sm:items-center"
      role="dialog"
      aria-modal="true"
      aria-labelledby="biometric-setup-title"
    >
      <div className="w-full max-w-sm rounded-2xl border border-border bg-bg-panel p-5 shadow-2xl">
        <div className="flex flex-col items-center text-center">
          <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/10">
            <ScanFace className="h-8 w-8 text-accent" aria-hidden />
          </div>
          <h2 id="biometric-setup-title" className="text-lg font-semibold text-fg">
            Use {label} to unlock?
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-fg-muted">
            Unlock your wallet quickly without typing your passphrase each time. Your passphrase
            stays encrypted on this device and is only released after {label} confirms your
            identity.
          </p>
        </div>

        {error && <p className="mt-3 text-center text-xs text-danger">{error}</p>}

        <div className="mt-5 flex flex-col gap-2">
          <Button className="h-11 w-full rounded-xl" disabled={busy} onClick={onEnable}>
            {busy ? 'Enabling…' : `Enable ${label}`}
          </Button>
          <Button
            type="button"
            variant="ghost"
            className="h-11 w-full rounded-xl text-fg-muted"
            disabled={busy}
            onClick={onDismiss}
          >
            Not Now
          </Button>
        </div>
      </div>
    </div>
  );
}

/**
 * Global host: shows the Face ID setup sheet when `requestBiometricSetupOffer` is called
 * (after first unlock or wallet creation on mobile).
 */
export function BiometricSetupOfferHost() {
  const coin = useActiveCoin();
  const { mobileOnly } = useWalletMode();
  const pending = useBiometricSetupOffer((s) => s.pending);
  const clear = useBiometricSetupOffer((s) => s.clear);
  const status = useBiometricUnlockStatus(coin);
  const prefs = useUserPreferences((s) => s.prefs);
  const prefsLoaded = useUserPreferences((s) => s.loaded);
  const updatePrefs = useUserPreferences((s) => s.update);
  const queryClient = useQueryClient();
  const [setupError, setSetupError] = useState<string | null>(null);

  const enable = useMutation({
    mutationFn: async () => {
      if (!pending || pending.coin !== coin) return;
      setSetupError(null);
      await biometricUnlockEnable(coin, pending.passphrase);
      const label = biometryLabel(status.data?.biometry_type ?? null);
      await promptBiometricUnlock(`Confirm ${label} for Vericonomy Wallet`);
    },
    onSuccess: async () => {
      queryClient.setQueryData(
        biometricUnlockQueryKey(coin),
        (prev: BiometricStatus | undefined): BiometricStatus => ({
          available: prev?.available ?? true,
          biometry_type: prev?.biometry_type ?? status.data?.biometry_type ?? null,
          enabled: true,
          configured: true,
        })
      );
      await updatePrefs({
        biometric_unlock_enabled: true,
        biometric_unlock_prompt_dismissed: true,
      });
      await queryClient.invalidateQueries({ queryKey: biometricUnlockQueryKey(coin) });
      clear();
    },
    onError: (error) => {
      setSetupError(formatBiometricError(error));
    },
  });

  const dismiss = useMutation({
    mutationFn: async () => {
      await updatePrefs({ biometric_unlock_prompt_dismissed: true });
    },
    onSuccess: () => {
      clear();
    },
  });

  if (!mobileOnly || !pending || pending.coin !== coin || status.isLoading || !prefsLoaded) {
    return null;
  }

  if (!shouldOfferBiometricSetup(status.data, prefs, prefsLoaded)) {
    clear();
    return null;
  }

  const label = biometryLabel(status.data?.biometry_type ?? null);

  return (
    <BiometricSetupPromptView
      label={label}
      busy={enable.isPending || dismiss.isPending}
      error={setupError}
      onEnable={() => enable.mutate()}
      onDismiss={() => dismiss.mutate()}
    />
  );
}
