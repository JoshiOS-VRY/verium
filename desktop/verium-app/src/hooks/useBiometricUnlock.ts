import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';
import type { CoinId } from '@/lib/coin/profile';
import { coinQueryKey } from '@/lib/coin/profile';
import {
  biometricUnlockStatus,
  biometricUnlockWallet,
  biometryLabel,
  promptBiometricUnlock,
  type BiometricStatus,
} from '@/lib/biometric/client';

export function biometricUnlockQueryKey(coin: CoinId) {
  return ['biometric-unlock-status', coin] as const;
}

export function useBiometricUnlockStatus(coin: CoinId) {
  return useQuery({
    queryKey: biometricUnlockQueryKey(coin),
    queryFn: () => biometricUnlockStatus(coin),
    staleTime: 30_000,
  });
}

export function isBiometricUnlockReady(status: BiometricStatus | undefined) {
  return Boolean(status?.available && status.enabled && status.configured);
}

export function useBiometricUnlockMutation(coin: CoinId, onUnlocked?: () => void) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (status: BiometricStatus) => {
      const label = biometryLabel(status.biometry_type);
      await promptBiometricUnlock(`Unlock Vericonomy Wallet with ${label}`);
      await biometricUnlockWallet(coin);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, 'getwalletinfo'),
      });
      await queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, 'listtransactions'),
      });
      onUnlocked?.();
    },
  });
}

export function usePromptBiometricUnlock(coin: CoinId, onUnlocked?: () => void) {
  const status = useBiometricUnlockStatus(coin);
  const unlock = useBiometricUnlockMutation(coin, onUnlocked);

  const prompt = useCallback(() => {
    if (!status.data || !isBiometricUnlockReady(status.data)) return;
    unlock.mutate(status.data);
  }, [status.data, unlock]);

  return { status, unlock, prompt, ready: isBiometricUnlockReady(status.data) };
}
