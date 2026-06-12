import { createContext, useContext, type RefObject } from 'react';

export const MobileScrollContext = createContext<RefObject<HTMLElement | null> | null>(null);

export function useMobileScrollContainer(): RefObject<HTMLElement | null> | null {
  return useContext(MobileScrollContext);
}
