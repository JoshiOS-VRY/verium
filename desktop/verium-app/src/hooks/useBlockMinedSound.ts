import { useEffect } from 'react';
import { playBlockMinedSound } from '@/lib/block-mined-sound';
import { useUserPreferences } from '@/lib/user-preferences';
import { subscribeBlockMined } from '@/hooks/useBlockMinedWatcher';
import { useWalletMode } from '@/hooks/useWalletMode';

/** Plays the block-found chime when the user preference is enabled. */
export function useBlockMinedSound(): void {
  const { isLight } = useWalletMode();
  const enabled = useUserPreferences((s) => s.prefs.play_sound_on_block_mined === true);

  useEffect(() => {
    if (isLight || !enabled) return;
    return subscribeBlockMined(() => {
      void playBlockMinedSound();
    });
  }, [isLight, enabled]);
}
