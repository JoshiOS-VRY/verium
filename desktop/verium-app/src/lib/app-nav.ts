import type { LucideIcon } from "lucide-react";
import {
  ArrowLeftRight,
  BookOpen,
  BookUser,
  Coins,
  Cpu,
  Gauge,
  Link2,
  Lock,
  Network as NetworkIcon,
  ScrollText,
  Settings as SettingsIcon,
  ShieldCheck,
  Terminal,
} from "lucide-react";
import type { CoinId } from "@/lib/coin/profile";

export interface AppNavItem {
  to: string;
  label: string;
  shortLabel?: string;
  icon: LucideIcon;
  coins?: CoinId[];
  testNetworkOnly?: boolean;
  fullNodeOnly?: boolean;
  requiresPassphrase?: boolean;
  /** Shown in the mobile bottom tab bar. */
  mobileTab?: boolean;
}

export const APP_NAV_ITEMS: AppNavItem[] = [
  {
    to: "/dashboard",
    label: "Dashboard",
    shortLabel: "Home",
    icon: Gauge,
    mobileTab: true,
  },
  {
    to: "/mining",
    label: "Mining",
    icon: Cpu,
    coins: ["verium"],
    requiresPassphrase: true,
    fullNodeOnly: true,
  },
  {
    to: "/staking",
    label: "Staking",
    icon: Coins,
    coins: ["vericoin"],
    requiresPassphrase: true,
    fullNodeOnly: true,
  },
  {
    to: "/network",
    label: "Network",
    icon: NetworkIcon,
    fullNodeOnly: true,
  },
  {
    to: "/binary-chain",
    label: "Binary Chain",
    icon: Link2,
    testNetworkOnly: true,
  },
  {
    to: "/transactions",
    label: "Transactions",
    shortLabel: "Activity",
    icon: ArrowLeftRight,
    requiresPassphrase: true,
    mobileTab: true,
  },
  {
    to: "/addresses",
    label: "Address book",
    shortLabel: "Addresses",
    icon: BookUser,
    mobileTab: true,
  },
  { to: "/security", label: "Security", icon: Lock, requiresPassphrase: true },
  {
    to: "/sign",
    label: "Sign & verify",
    icon: ShieldCheck,
    requiresPassphrase: true,
    fullNodeOnly: true,
  },
  {
    to: "/console",
    label: "RPC console",
    icon: Terminal,
    requiresPassphrase: true,
    fullNodeOnly: true,
  },
  { to: "/logs", label: "Logs", icon: ScrollText, fullNodeOnly: true },
  { to: "/resources", label: "Resources", icon: BookOpen },
  {
    to: "/settings",
    label: "Settings",
    icon: SettingsIcon,
    mobileTab: true,
  },
];

export const APP_ROUTE_TITLES: Record<string, string> = Object.fromEntries(
  APP_NAV_ITEMS.map((item) => [item.to, item.label]),
);

export interface NavFilterOptions {
  activeCoin: CoinId;
  enabledCoins: CoinId[];
  isLight: boolean;
  isTestNetwork: boolean;
  binarytestEnabled: boolean;
}

export function filterAppNavItems(
  items: AppNavItem[],
  {
    activeCoin,
    enabledCoins,
    isLight,
    isTestNetwork,
    binarytestEnabled,
  }: NavFilterOptions,
): AppNavItem[] {
  return items.filter((item) => {
    if (item.fullNodeOnly && isLight) return false;
    if (item.testNetworkOnly && (!binarytestEnabled || !isTestNetwork)) {
      return false;
    }
    if (!item.coins) return true;
    return item.coins.some(
      (coin) => enabledCoins.includes(coin) && coin === activeCoin,
    );
  });
}
