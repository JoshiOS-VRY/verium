import { useEffect } from 'react';
import { playStakeRewardSound } from '@/lib/block-mined-sound';
import { useUserPreferences } from '@/lib/user-preferences';
import { subscribeStakeReward } from '@/hooks/useStakeRewardWatcher';
import { useWalletMode } from '@/hooks/useWalletMode';

/** Plays the stake-reward chime when the user preference is enabled. */
export function useStakeRewardSound(): void {
  const { isLight } = useWalletMode();
  const enabled = useUserPreferences((s) => s.prefs.play_sound_on_stake_reward === true);

  useEffect(() => {
    if (isLight || !enabled) return;
    return subscribeStakeReward(() => {
      void playStakeRewardSound();
    });
  }, [isLight, enabled]);
}
