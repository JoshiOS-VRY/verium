import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, ScanFace } from 'lucide-react';
import { MobileSettingsGroup } from '@/components/mobile/MobileSettingsGroup';
import { Button } from '@/components/ui/Button';
import { useActiveCoin } from '@/lib/coin/context';
import {
  biometricUnlockDisable,
  biometricUnlockEnable,
  biometryLabel,
  promptBiometricUnlock,
  type BiometricStatus,
} from '@/lib/biometric/client';
import {
  biometricUnlockQueryKey,
  isBiometricUnlockReady,
  useBiometricUnlockStatus,
  usePromptBiometricUnlock,
} from '@/hooks/useBiometricUnlock';
import { useUserPreferences } from '@/lib/user-preferences';

function formatBiometricError(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  const text = String(error).trim();
  return text && text !== '[object Object]' ? text : 'Could not enable biometric unlock.';
}

export function BiometricUnlockCard() {
  const coin = useActiveCoin();
  const queryClient = useQueryClient();
  const updatePrefs = useUserPreferences((s) => s.update);

  const [passphrase, setPassphrase] = useState('');
  const [showSetup, setShowSetup] = useState(false);
  const [setupError, setSetupError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const status = useBiometricUnlockStatus(coin);

  const enable = useMutation({
    mutationFn: async () => {
      setSetupError(null);
      setSuccessMessage(null);
      await biometricUnlockEnable(coin, passphrase);
      const label = biometryLabel(status.data?.biometry_type ?? null);
      await promptBiometricUnlock(`Confirm ${label} for Vericonomy Wallet`);
    },
    onSuccess: async () => {
      const label = biometryLabel(status.data?.biometry_type ?? null);
      setPassphrase('');
      setShowSetup(false);
      setSuccessMessage(`${label} unlock enabled`);

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
    },
    onError: (error) => {
      setSetupError(formatBiometricError(error));
    },
  });

  const disable = useMutation({
    mutationFn: () => biometricUnlockDisable(coin),
    onSuccess: async () => {
      setSuccessMessage(null);
      queryClient.setQueryData(
        biometricUnlockQueryKey(coin),
        (prev: BiometricStatus | undefined): BiometricStatus => ({
          available: prev?.available ?? true,
          biometry_type: prev?.biometry_type ?? null,
          enabled: false,
          configured: false,
        })
      );
      await updatePrefs({
        biometric_unlock_enabled: false,
        biometric_unlock_prompt_dismissed: true,
      });
      await queryClient.invalidateQueries({ queryKey: biometricUnlockQueryKey(coin) });
    },
  });

  if (status.isLoading) {
    return null;
  }

  if (!status.data?.available) {
    return (
      <MobileSettingsGroup
        title="Unlock with Face ID"
        description="Use Face ID instead of typing your wallet passphrase."
        defaultOpen={false}
      >
        <p className="text-xs leading-relaxed text-fg-muted">
          Biometrics are not available in this build. Confirm Face ID is set up in iOS Settings,
          then reinstall the latest app build.
        </p>
      </MobileSettingsGroup>
    );
  }

  const label = biometryLabel(status.data.biometry_type);
  const enabled = isBiometricUnlockReady(status.data);

  return (
    <MobileSettingsGroup
      title={`Unlock with ${label}`}
      description={`Use ${label} instead of typing your wallet passphrase each time.`}
      defaultOpen={enabled || showSetup}
    >
      <label className="mobile-checkbox-row">
        <input
          type="checkbox"
          checked={enabled}
          disabled={disable.isPending || enable.isPending}
          onChange={(e) => {
            setSuccessMessage(null);
            if (e.target.checked) {
              setShowSetup(true);
              setSetupError(null);
            } else {
              disable.mutate();
            }
          }}
        />
        <span>Enable {label} unlock</span>
      </label>

      {successMessage && (
        <p className="mt-2 flex items-center gap-2 text-xs text-success">
          <CheckCircle2 className="h-3.5 w-3.5 shrink-0" />
          {successMessage}
        </p>
      )}

      {showSetup && !enabled && (
        <div className="mt-3 flex flex-col gap-2 rounded-xl border border-border bg-bg-subtle/50 p-3">
          <p className="text-xs leading-relaxed text-fg-muted">
            Enter your wallet passphrase once. After you tap Enable, {label} will confirm your
            identity, then the passphrase is saved on this device for quick unlock.
          </p>
          <input
            type="password"
            autoComplete="current-password"
            value={passphrase}
            onChange={(e) => {
              setPassphrase(e.target.value);
              setSetupError(null);
            }}
            placeholder="Wallet passphrase (leave empty if none)"
            className="mobile-input w-full"
          />
          {setupError && <p className="text-xs text-danger">{setupError}</p>}
          <div className="flex gap-2">
            <Button
              variant="secondary"
              className="h-11 flex-1 rounded-xl"
              onClick={() => {
                setShowSetup(false);
                setPassphrase('');
                setSetupError(null);
              }}
            >
              Cancel
            </Button>
            <Button
              className="h-11 flex-1 rounded-xl"
              disabled={enable.isPending}
              onClick={() => enable.mutate()}
            >
              {enable.isPending ? 'Enabling…' : 'Enable'}
            </Button>
          </div>
        </div>
      )}

      {enabled && (
        <p className="mt-2 text-xs text-fg-subtle">
          Unlock from the Dashboard or before sending. Disabling removes the saved passphrase from
          this device.
        </p>
      )}
    </MobileSettingsGroup>
  );
}

/** Compact Face ID button for unlock forms. */
export function BiometricUnlockButton({
  onUnlocked,
  className,
}: {
  onUnlocked?: () => void;
  className?: string;
}) {
  const coin = useActiveCoin();
  const { status, unlock, prompt, ready } = usePromptBiometricUnlock(coin, onUnlocked);

  if (status.isLoading || !ready) {
    return null;
  }

  const label = biometryLabel(status.data?.biometry_type ?? null);

  return (
    <div className={className}>
      <Button
        type="button"
        variant="secondary"
        className="h-11 w-full rounded-xl"
        disabled={unlock.isPending}
        onClick={prompt}
      >
        <ScanFace className="h-4 w-4" />
        {unlock.isPending ? 'Verifying…' : `Unlock with ${label}`}
      </Button>
      {unlock.error && (
        <p className="mt-2 text-xs text-danger">{formatBiometricError(unlock.error)}</p>
      )}
    </div>
  );
}
