import type { CoinId } from '@/lib/coin/profile';
import type { WalletMode } from '@/lib/light-wallet/client';
import type { OnboardingCheckpoint, OnboardingIntent, WalletProfile } from '@/lib/wallet-profile';

/**
 * Intent router for the setup wizard.
 *
 * The wizard classifies *why* the user is here (fresh install, legacy upgrade,
 * unlock an existing wallet, continue a light wallet, or migrate between modes)
 * up front from the unified `WalletProfile`, then routes to the right flow. This
 * replaces the previous approach of branching implicitly inside each step via
 * many `useEffect` auto-advances, which was opaque to debug.
 */

export type WizardStep =
  | 'hub'
  | 'welcome'
  | 'daemon'
  | 'wallet'
  | 'recovery'
  | 'hd_upgrade'
  | 'twofa'
  | 'bootstrap'
  | 'done'
  | 'advanced';

export interface IntentRoute {
  intent: OnboardingIntent;
  /** Step the wizard should open at for this intent + mode. */
  initialStep: WizardStep;
  /** Ordered step ids shown in the progress indicator. */
  steps: WizardStep[];
  /** True when the legacy upgrade wizard owns this flow. */
  isLegacyUpgrade: boolean;
}

const FULL_STEPS: WizardStep[] = [
  'welcome',
  'daemon',
  'wallet',
  'recovery',
  'twofa',
  'bootstrap',
  'done',
];

const LIGHT_STEPS: WizardStep[] = ['welcome', 'wallet', 'twofa', 'done'];

/**
 * Legacy upgrade has its own ordered flow: explain -> connect node ->
 * unlock -> mandatory security (HD + recovery) -> finish.
 */
const LEGACY_STEPS: WizardStep[] = [
  'welcome',
  'daemon',
  'wallet',
  'hd_upgrade',
  'recovery',
  'twofa',
  'bootstrap',
  'done',
];

/**
 * Derive the route from a wallet profile. `mode` is the effective per-coin
 * wallet mode the user is setting up (full node vs light).
 */
export function routeForProfile(
  profile: Pick<WalletProfile, 'intent' | 'mode'>,
  mode: WalletMode = profile.mode
): IntentRoute {
  const intent = profile.intent;

  if (mode === 'full_node' && intent === 'legacy_upgrade') {
    return {
      intent,
      initialStep: 'welcome',
      steps: LEGACY_STEPS,
      isLegacyUpgrade: true,
    };
  }

  if (mode === 'light') {
    return {
      intent,
      initialStep: 'welcome',
      steps: LIGHT_STEPS,
      isLegacyUpgrade: false,
    };
  }

  return {
    intent,
    initialStep: 'welcome',
    steps: FULL_STEPS,
    isLegacyUpgrade: false,
  };
}

export function stepLabel(step: WizardStep): string {
  switch (step) {
    case 'welcome':
      return 'Welcome';
    case 'daemon':
      return 'Start node';
    case 'wallet':
      return 'Wallet';
    case 'hd_upgrade':
      return 'Upgrade';
    case 'recovery':
      return 'Recovery';
    case 'twofa':
      return '2FA';
    case 'bootstrap':
      return 'Sync';
    case 'done':
      return 'Finish';
    default:
      return step;
  }
}

/** Build a checkpoint to persist at a step transition. */
export function checkpointFor(
  intent: OnboardingIntent,
  step: WizardStep,
  legacyPath?: string | null
): OnboardingCheckpoint {
  return {
    phase: step === 'done' ? 'complete' : 'in_progress',
    intent,
    step,
    legacy_path: legacyPath ?? null,
  };
}

/**
 * Decide whether to resume a persisted checkpoint. We only resume in-progress
 * checkpoints whose step is still meaningful for the route; completed or empty
 * checkpoints send the user through the normal route.
 */
export function resumeStep(
  route: IntentRoute,
  checkpoint: OnboardingCheckpoint | undefined
): WizardStep {
  if (!checkpoint || checkpoint.phase !== 'in_progress' || !checkpoint.step) {
    return route.initialStep;
  }
  const step = checkpoint.step as WizardStep;
  return route.steps.includes(step) ? step : route.initialStep;
}

export type { OnboardingCheckpoint, OnboardingIntent } from '@/lib/wallet-profile';
export type { CoinId };
