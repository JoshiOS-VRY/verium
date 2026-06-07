import { useCallback, useRef, useState } from "react";
import type { CoinId } from "@/lib/coin/profile";
import { twoFactorIsGated } from "@/lib/security/client";

export function useTwoFactorGate(coin: CoinId) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("Two-factor authentication");
  const pendingRef = useRef<((code: string) => void) | null>(null);

  const gate = useCallback(
    async (
      action: string,
      onVerified: (code: string) => void,
      options?: { amount?: number; title?: string },
    ) => {
      const gated = await twoFactorIsGated(action, coin, options?.amount);
      if (gated) {
        setTitle(options?.title ?? "Two-factor authentication");
        pendingRef.current = onVerified;
        setOpen(true);
        return;
      }
      onVerified("");
    },
    [coin],
  );

  const cancel = useCallback(() => {
    setOpen(false);
    pendingRef.current = null;
  }, []);

  const verified = useCallback((code: string) => {
    setOpen(false);
    pendingRef.current?.(code);
    pendingRef.current = null;
  }, []);

  return { open, title, gate, cancel, verified };
}
