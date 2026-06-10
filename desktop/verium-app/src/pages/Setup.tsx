import { useCallback, useEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  CheckCircle2,
  Cog,
  HardDriveDownload,
  HardDriveUpload,
  Loader2,
  ShieldCheck,
  Smartphone,
  Wallet as WalletIcon,
} from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import {
  useActiveCoin,
  useEnabledCoins,
  useSetActiveCoin,
} from "@/lib/coin/context";
import { coinQueryKey, COIN_PROFILES, type CoinId } from "@/lib/coin/profile";
import {
  anyEnabledCoinSetupIncomplete,
  coinSetupCompletePatch,
  isCoinWalletReady,
  needsLightWalletRecovery,
} from "@/lib/setup";
import {
  rpcGetConfig,
  rpcGetWalletInfo,
  rpcSetConfig,
  tauriDetectDaemon,
  tauriDetectDaemonRuntime,
  tauriEnsureFirstRun,
  tauriWalletFileStatus,
} from "@/lib/rpc/client";
import { useUserPreferences } from "@/lib/user-preferences";
import {
  resolveWalletSetupMode,
  walletSetupModeLabel,
} from "@/lib/wallet-setup";
import { DaemonConnectionPanel } from "@/components/DaemonConnectionPanel";
import { BootstrapDialog } from "@/components/BootstrapDialog";
import { WalletCreateForm } from "@/components/WalletCreateForm";
import { WalletImportForm } from "@/components/WalletImportForm";
import { WalletUnlockForm } from "@/components/WalletUnlockForm";
import { RestoreFromPhraseForm } from "@/components/RestoreFromPhraseForm";
import { RecoveryPhraseWizard } from "@/components/RecoveryPhraseWizard";
import { TwoFactorEnrollmentPanel } from "@/components/TwoFactorEnrollmentPanel";
import {
  recoveryApplyHdSeed,
  recoveryWalletIsHd,
  twoFactorStatus,
} from "@/lib/security/client";
import { useDaemonStatus } from "@/hooks/useDaemonStatus";
import { isNodeReady, nodeStatusLabel } from "@/lib/node/status";
import { useIsTestNetwork } from "@/lib/network-mode";
import { LIGHT_WALLET_ENABLED } from "@/lib/features";
import { walletModeSet, walletModeSetForCoin } from "@/lib/light-wallet/client";
import { LightWalletSetupForm } from "@/components/LightWalletSetupForm";
import { SetupWalletHub } from "@/components/SetupWalletHub";
import { useInvalidateWalletMode, useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import {
  onboardingCheckpointSet,
  onboardingMarkComplete,
  tauriWalletProfile,
} from "@/lib/wallet-profile";
import {
  profileWalletPresence,
  useSetupHubProfile,
} from "@/hooks/useSetupHubProfiles";
import { FeatureTile } from "@/components/onboarding/FeatureTile";
import { SetupStepIndicator } from "@/components/onboarding/SetupStepIndicator";
import { LegacyUpgradeWizard } from "@/components/onboarding/LegacyUpgradeWizard";

type Step =
  | "hub"
  | "welcome"
  | "daemon"
  | "wallet"
  | "recovery"
  | "twofa"
  | "bootstrap"
  | "done"
  | "advanced";

const FULL_STEPS: { id: Exclude<Step, "advanced">; label: string }[] = [
  { id: "welcome", label: "Welcome" },
  { id: "daemon", label: "Start node" },
  { id: "wallet", label: "Wallet" },
  { id: "recovery", label: "Recovery" },
  { id: "twofa", label: "2FA" },
  { id: "bootstrap", label: "Sync" },
  { id: "done", label: "Finish" },
];

const LIGHT_STEPS: { id: Exclude<Step, "advanced">; label: string }[] = [
  { id: "welcome", label: "Welcome" },
  { id: "wallet", label: "Wallet" },
  { id: "twofa", label: "2FA" },
  { id: "done", label: "Finish" },
];

type WalletAction =
  | "choose"
  | "create"
  | "import"
  | "restore_phrase"
  | "unlock";

export function Setup() {
  const coin = useActiveCoin();
  const profile = COIN_PROFILES[coin];
  const navigate = useNavigate();
  const location = useLocation();
  const setActiveCoin = useSetActiveCoin();
  const enabledCoins = useEnabledCoins();
  const isTestNetwork = useIsTestNetwork();
  const {
    isLight,
    mode: persistedWalletMode,
    isLoading: walletModeLoading,
  } = useWalletMode();
  const invalidateWalletMode = useInvalidateWalletMode();
  const [setupWalletMode, setSetupWalletMode] = useState<"light" | "full_node">(
    "full_node",
  );
  const [showAdvancedLightMode, setShowAdvancedLightMode] = useState(false);
  const lightSetupActive = setupWalletMode === "light";
  const [step, setStep] = useState<Step>("hub");
  const [bootstrapOpen, setBootstrapOpen] = useState(false);
  const [datadirDraft, setDatadirDraft] = useState<string>("");
  const [walletAction, setWalletAction] = useState<WalletAction>("choose");
  const [pendingPassphrase, setPendingPassphrase] = useState<string | null>(
    null,
  );
  const [recoveryError, setRecoveryError] = useState<string | null>(null);
  const [legacyFreshStart, setLegacyFreshStart] = useState(false);
  const [hubOpeningCoin, setHubOpeningCoin] = useState<CoinId | null>(null);
  const defaultedFullNodeForExistingWallet = useRef(false);
  const handledSetupNav = useRef<string | null>(null);
  const queryClient = useQueryClient();

  const updatePrefs = useUserPreferences((s) => s.update);
  const prefs = useUserPreferences((s) => s.prefs);
  const prefsLoaded = useUserPreferences((s) => s.loaded);
  const hubModeInitialized = useRef(false);

  useEffect(() => {
    if (!prefsLoaded || hubModeInitialized.current) return;
    hubModeInitialized.current = true;
    if (prefs.wallet_mode === "light") {
      setSetupWalletMode("light");
      setShowAdvancedLightMode(true);
    }
  }, [prefsLoaded, prefs.wallet_mode]);

  const resetCoinOnboarding = useCallback(() => {
    setWalletAction("choose");
    setBootstrapOpen(false);
    setPendingPassphrase(null);
    setRecoveryError(null);
    setLegacyFreshStart(false);
  }, []);

  const goToHub = useCallback(() => {
    resetCoinOnboarding();
    setStep("hub");
    void queryClient.invalidateQueries({
      predicate: (q) =>
        Array.isArray(q.queryKey) && q.queryKey[1] === "wallet-profile",
    });
  }, [resetCoinOnboarding, queryClient]);

  const openReadyCoinDashboard = useCallback(
    async (targetCoin: CoinId, openMode: "light" | "full_node") => {
      await updatePrefs({
        ...coinSetupCompletePatch(targetCoin, prefs),
        active_coin: targetCoin,
      });
      await onboardingMarkComplete(targetCoin).catch(() => undefined);
      if (LIGHT_WALLET_ENABLED) {
        try {
          await walletModeSetForCoin(targetCoin, openMode);
          invalidateWalletMode();
        } catch {
          /* prefs may reconcile on next load */
        }
      }
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(targetCoin, "wallet-profile"),
      });
      navigate("/dashboard", { replace: true });
    },
    [
      prefs,
      updatePrefs,
      invalidateWalletMode,
      navigate,
      queryClient,
    ],
  );

  const startCoinSetup = useCallback(
    async (targetCoin: CoinId) => {
      const walletProfile = await tauriWalletProfile(targetCoin).catch(
        () => null,
      );
      const presence = profileWalletPresence(walletProfile ?? undefined);

      if (walletProfile && needsLightWalletRecovery(walletProfile, setupWalletMode)) {
        setActiveCoin(targetCoin);
        resetCoinOnboarding();
        if (LIGHT_WALLET_ENABLED) {
          try {
            await walletModeSetForCoin(targetCoin, "light");
            invalidateWalletMode();
          } catch {
            /* continue */
          }
        }
        setWalletAction("restore_phrase");
        setStep("wallet");
        return;
      }

      if (
        walletProfile?.ready ||
        isCoinWalletReady(targetCoin, prefs, {
          walletMode: setupWalletMode,
          ...presence,
          lightKeystoreHealth: walletProfile?.light_keystore_health,
        })
      ) {
        const openMode =
          walletProfile?.ready && walletProfile.mode === "light"
            ? "light"
            : walletProfile?.ready && walletProfile.mode === "full_node"
              ? "full_node"
              : setupWalletMode;
        await openReadyCoinDashboard(targetCoin, openMode);
        return;
      }
      // Full-node hub + decryptable light-only wallet on disk.
      if (
        setupWalletMode === "full_node" &&
        !presence.hasFullNodeWallet &&
        presence.hasLightWallet &&
        walletProfile?.light_keystore_health === "ok"
      ) {
        await openReadyCoinDashboard(targetCoin, "light");
        return;
      }
      setActiveCoin(targetCoin);
      resetCoinOnboarding();
      if (LIGHT_WALLET_ENABLED) {
        const setupMode = setupWalletMode;
        try {
          await walletModeSetForCoin(targetCoin, setupMode);
          invalidateWalletMode();
        } catch {
          /* continue with local mode choice */
        }
      }
      setStep("welcome");
    },
    [
      prefs,
      setActiveCoin,
      resetCoinOnboarding,
      setupWalletMode,
      invalidateWalletMode,
      openReadyCoinDashboard,
    ],
  );

  const handleHubSelectCoin = useCallback(
    async (targetCoin: CoinId) => {
      setHubOpeningCoin(targetCoin);
      try {
        await startCoinSetup(targetCoin);
      } finally {
        setHubOpeningCoin(null);
      }
    },
    [startCoinSetup],
  );
  const config = useQuery({
    queryKey: coinQueryKey(coin, "daemon-config"),
    queryFn: () => rpcGetConfig(coin),
  });
  const binary = useQuery({
    queryKey: coinQueryKey(coin, "detect-daemon"),
    queryFn: () => tauriDetectDaemon(coin),
  });
  const runtime = useQuery({
    queryKey: coinQueryKey(coin, "detect-daemon-runtime"),
    queryFn: () => tauriDetectDaemonRuntime(coin),
    refetchInterval: step === "daemon" ? 4_000 : false,
    enabled: step === "daemon",
  });
  const walletFile = useQuery({
    queryKey: coinQueryKey(coin, "wallet-file-status"),
    queryFn: () => tauriWalletFileStatus(coin),
    refetchInterval: step === "daemon" ? 4_000 : false,
    enabled:
      !lightSetupActive &&
      (step === "wallet" || step === "daemon" || step === "welcome"),
  });
  const setupCoinProfile = useSetupHubProfile(
    coin,
    step !== "hub" && step !== "advanced",
  );
  const setupCoinPresence = profileWalletPresence(setupCoinProfile.data);
  const setupCoinHasLightWallet = setupCoinPresence.hasLightWallet;
  const setupCoinKeystoreUnreadable = needsLightWalletRecovery(
    setupCoinProfile.data,
    setupWalletMode,
  );
  /** Light setup UI when hub chose light or prefs already use light without full-node wallet. */
  const lightWalletFlow =
    setupWalletMode === "light" ||
    (isLight && !setupCoinPresence.hasFullNodeWallet);
  const activeSteps = lightWalletFlow ? LIGHT_STEPS : FULL_STEPS;

  useEffect(() => {
    if (step !== "wallet" || !setupCoinKeystoreUnreadable) return;
    setWalletAction("restore_phrase");
  }, [step, setupCoinKeystoreUnreadable]);

  const hasFullNodeWallet = lightSetupActive
    ? setupCoinPresence.hasFullNodeWallet
    : walletFile.data?.exists === true ||
      walletFile.data?.legacy_wallet_detected === true;
  const showFullNodeMigrationHint =
    lightSetupActive && hasFullNodeWallet && !setupCoinHasLightWallet;

  // Full-node legacy upgrade: a legacy wallet.dat exists outside the configured
  // datadir. Route through the dedicated wizard unless the user opts to start
  // fresh. The wizard owns its own sub-steps until it calls onComplete.
  const legacyWizardActive =
    step === "wallet" &&
    !isLight &&
    setupWalletMode !== "light" &&
    walletFile.data?.legacy_wallet_detected === true &&
    walletFile.data?.exists !== true &&
    !legacyFreshStart;

  const switchToFullNodeSetup = async () => {
    setSetupWalletMode("full_node");
    setWalletAction("choose");
    try {
      await walletModeSet("full_node");
      invalidateWalletMode();
    } catch {
      /* still continue into full-node setup */
    }
    setStep("daemon");
  };

  const { data: nodeStatus, isConnecting } = useDaemonStatus(coin);
  const connected = isNodeReady(nodeStatus);

  const walletInfo = useQuery({
    queryKey: coinQueryKey(coin, "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: step === "wallet" && (lightSetupActive || connected),
    retry: 1,
  });

  useEffect(() => {
    if (defaultedFullNodeForExistingWallet.current || !walletFile.data) return;
    if (setupWalletMode === "light" || isLight) return;
    if (hasFullNodeWallet && !setupCoinHasLightWallet && LIGHT_WALLET_ENABLED) {
      defaultedFullNodeForExistingWallet.current = true;
      setSetupWalletMode("full_node");
    }
  }, [
    walletFile.data,
    hasFullNodeWallet,
    setupCoinHasLightWallet,
    setupWalletMode,
    isLight,
  ]);

  useEffect(() => {
    if (!lightSetupActive || step !== "wallet" || isLight) return;
    void walletModeSetForCoin(coin, "light")
      .then(() => invalidateWalletMode())
      .catch(() => undefined);
  }, [lightSetupActive, step, isLight, coin, invalidateWalletMode]);

  const walletSetupMode = resolveWalletSetupMode(
    coin,
    connected || lightSetupActive,
    walletInfo.isLoading,
    walletInfo.data,
    walletFile.data?.exists,
  );
  const walletUnlockAwaitingNode =
    walletAction === "unlock" &&
    walletSetupMode === "loading" &&
    walletFile.data?.exists === true;

  useEffect(() => {
    if (walletModeLoading || step === "hub") return;
    if (persistedWalletMode === "light" && setupWalletMode !== "full_node") {
      setSetupWalletMode("light");
      setShowAdvancedLightMode(true);
    }
  }, [persistedWalletMode, walletModeLoading, step, setupWalletMode]);

  const handleSetupWalletModeChange = useCallback(
    async (mode: "light" | "full_node") => {
      setSetupWalletMode(mode);
      if (mode === "light") {
        setShowAdvancedLightMode(true);
      }
    },
    [],
  );

  useEffect(() => {
    const state = location.state as {
      setupHub?: boolean;
      lightWalletSetup?: CoinId;
    } | null;
    if (state?.setupHub) {
      goToHub();
      navigate(location.pathname, { replace: true, state: null });
      return;
    }
    if (!state?.lightWalletSetup) return;
    const navKey = `light:${state.lightWalletSetup}`;
    if (handledSetupNav.current === navKey) return;
    handledSetupNav.current = navKey;
    setSetupWalletMode("light");
    setShowAdvancedLightMode(true);
    void startCoinSetup(state.lightWalletSetup).finally(() => {
      navigate(location.pathname, { replace: true, state: null });
    });
  }, [location.pathname, location.state, goToHub, navigate, startCoinSetup]);

  useEffect(() => {
    if (config.data && !datadirDraft) {
      setDatadirDraft(config.data.datadir);
    }
  }, [config.data, datadirDraft]);

  const walletIsHd = useQuery({
    queryKey: coinQueryKey(coin, "wallet-is-hd"),
    queryFn: () => recoveryWalletIsHd(coin),
    enabled: connected && (step === "wallet" || step === "recovery"),
  });

  const twoFa = useQuery({
    queryKey: ["two-factor"],
    queryFn: twoFactorStatus,
    staleTime: 30_000,
    enabled: step !== "hub" && step !== "welcome" && step !== "advanced",
  });

  const advanceAfterChainWalletReady = () => {
    const twoFaEnabled =
      queryClient.getQueryData<Awaited<ReturnType<typeof twoFactorStatus>>>([
        "two-factor",
      ])?.enabled ?? twoFa.data?.enabled;
    if (lightWalletFlow) {
      setStep(twoFaEnabled ? "done" : "twofa");
      return;
    }
    setStep(twoFaEnabled ? "bootstrap" : "twofa");
  };

  const applyRecovery = useMutation({
    mutationFn: ({ phrase, unlock }: { phrase: string; unlock?: string }) =>
      recoveryApplyHdSeed(coin, phrase, undefined, unlock),
    onSuccess: async () => {
      setRecoveryError(null);
      setPendingPassphrase(null);
      await queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, "wallet-is-hd"),
      });
      advanceAfterChainWalletReady();
    },
    onError: (err) => setRecoveryError(String(err)),
  });

  useEffect(() => {
    if (step === "twofa" && twoFa.data?.enabled) {
      setStep(lightSetupActive ? "done" : "bootstrap");
    }
  }, [step, twoFa.data?.enabled, lightSetupActive]);

  useEffect(() => {
    if (step !== "wallet" || lightSetupActive || isLight || setupCoinKeystoreUnreadable) {
      return;
    }
    if (walletSetupMode === "ready" && walletIsHd.data === true) {
      if (twoFa.isLoading) return;
      advanceAfterChainWalletReady();
      return;
    }
    if (walletSetupMode === "needs_unlock") {
      setWalletAction("unlock");
    } else if (
      walletSetupMode === "needs_encrypt" &&
      walletAction === "unlock"
    ) {
      setWalletAction("choose");
    }
  }, [
    step,
    walletSetupMode,
    walletAction,
    walletIsHd.data,
    twoFa.data?.enabled,
    twoFa.isLoading,
    lightSetupActive,
    isLight,
    setupCoinKeystoreUnreadable,
  ]);

  const saveConfig = useMutation({
    mutationFn: (partial: Parameters<typeof rpcSetConfig>[1]) =>
      rpcSetConfig(coin, partial),
  });

  const ensureFirstRun = useMutation({
    mutationFn: () => tauriEnsureFirstRun(coin),
  });

  useEffect(() => {
    if (step !== "daemon") return;
    void ensureFirstRun.mutate();
  }, [step, coin]);

  useEffect(() => {
    if (step === "daemon" && connected) {
      setStep("wallet");
    }
  }, [step, connected]);

  // Persist a resumable onboarding checkpoint per coin so a refresh, crash, or
  // restart returns the user to the same step instead of the hub. Best-effort:
  // failures never block the wizard.
  useEffect(() => {
    if (step === "hub" || step === "advanced") return;
    void onboardingCheckpointSet(coin, {
      phase: step === "done" ? "complete" : "in_progress",
      intent: lightSetupActive ? "light_continue" : null,
      step,
      legacy_path: walletFile.data?.legacy_wallet_path ?? null,
    }).catch(() => undefined);
  }, [step, coin, lightSetupActive, walletFile.data?.legacy_wallet_path]);

  const onboardingBusy =
    applyRecovery.isPending || ensureFirstRun.isPending || saveConfig.isPending;

  const finish = async () => {
    const nextPrefs = {
      ...prefs,
      ...coinSetupCompletePatch(coin, prefs),
    };
    await updatePrefs(coinSetupCompletePatch(coin, prefs));
    await onboardingMarkComplete(coin).catch(() => undefined);
    if (LIGHT_WALLET_ENABLED && lightSetupActive) {
      await walletModeSetForCoin(coin, "light").catch(() => undefined);
      invalidateWalletMode();
    }
    void queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, "wallet-profile"),
    });
    if (anyEnabledCoinSetupIncomplete(enabledCoins, nextPrefs)) {
      goToHub();
      return;
    }
    navigate("/dashboard");
  };

  const openDashboardFromHub = async () => {
    let patch = { ...prefs };
    for (const targetCoin of enabledCoins) {
      const walletProfile = await tauriWalletProfile(targetCoin).catch(
        () => null,
      );
      if (walletProfile && needsLightWalletRecovery(walletProfile, setupWalletMode)) {
        continue;
      }
      const presence = profileWalletPresence(walletProfile ?? undefined);
      let openMode: "light" | "full_node" | null = null;
      if (
        walletProfile?.ready ||
        isCoinWalletReady(targetCoin, patch, {
          walletMode: setupWalletMode,
          ...presence,
          lightKeystoreHealth: walletProfile?.light_keystore_health,
        })
      ) {
        openMode = walletProfile?.ready
          ? (walletProfile.mode as "light" | "full_node")
          : setupWalletMode;
      } else if (
        setupWalletMode === "full_node" &&
        !presence.hasFullNodeWallet &&
        presence.hasLightWallet &&
        walletProfile?.light_keystore_health === "ok"
      ) {
        openMode = "light";
      }
      if (openMode) {
        patch = { ...patch, ...coinSetupCompletePatch(targetCoin, patch) };
        if (LIGHT_WALLET_ENABLED) {
          await walletModeSetForCoin(targetCoin, openMode).catch(
            () => undefined,
          );
        }
      }
    }
    await updatePrefs({
      setup_completed: patch.setup_completed,
      setup_completed_by_coin: patch.setup_completed_by_coin,
      active_coin: coin,
    });
    invalidateWalletMode();
    navigate("/dashboard", { replace: true });
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg p-8 text-fg">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <CardTitle className="!normal-case !tracking-normal !text-base">
            {step === "hub"
              ? "Vericonomy wallets"
              : `Set up ${profile.displayName}`}
          </CardTitle>
          <CardDescription>
            {step === "hub"
              ? "Pick Verium or Vericoin, choose light or full-node mode, then walk through setup for each chain."
              : lightWalletFlow
                ? `Set up your ${profile.symbol} light wallet, save your recovery phrase, and enable app-wide 2FA. No local node or blockchain sync required.`
                : `Start the bundled ${profile.binaryName} node, set up your ${profile.symbol} wallet and recovery phrase, enable app-wide 2FA, then optionally import a chain bootstrap.`}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-5">
          {step === "hub" && (
            <SetupWalletHub
              walletMode={setupWalletMode}
              onWalletModeChange={(mode) =>
                void handleSetupWalletModeChange(mode)
              }
              showAdvancedLightMode={showAdvancedLightMode}
              onShowAdvancedLightMode={() => setShowAdvancedLightMode(true)}
              onSelectCoin={handleHubSelectCoin}
              onOpenDashboard={() => void openDashboardFromHub()}
              openingCoin={hubOpeningCoin}
            />
          )}

          {step !== "hub" && step !== "advanced" && (
            <SetupStepIndicator steps={activeSteps} currentId={step} />
          )}

          {step === "welcome" && (
            <div className="flex flex-col gap-4 text-sm text-fg-muted">
              <p>
                {setupWalletMode === "light" && LIGHT_WALLET_ENABLED
                  ? lightWalletCopy.setupWelcomeLight
                  : lightWalletCopy.setupWelcomeFull}
              </p>
              {coin === "vericoin" && setupWalletMode !== "light" && (
                <p>
                  New to Vericoin? Choose <strong>Create new wallet</strong> on
                  the next steps. We only reuse an older wallet when one is
                  already on this computer.
                </p>
              )}
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                {setupWalletMode === "light" && LIGHT_WALLET_ENABLED ? (
                  <>
                    <FeatureTile
                      icon={<ShieldCheck className="h-4 w-4" />}
                      title="Keys on device"
                      body="Recovery phrase and passphrase never leave this computer."
                    />
                    <FeatureTile
                      icon={<Cog className="h-4 w-4" />}
                      title="No sync"
                      body="Balance and history from Vericonomy light wallet servers."
                    />
                    <FeatureTile
                      icon={<HardDriveUpload className="h-4 w-4" />}
                      title="Import phrase"
                      body="Restore from your 24-word recovery phrase on any device."
                    />
                    <FeatureTile
                      icon={<Smartphone className="h-4 w-4" />}
                      title="Optional 2FA"
                      body="Protect sends and sensitive actions with an authenticator app."
                    />
                  </>
                ) : (
                  <>
                    <FeatureTile
                      icon={<Cog className="h-4 w-4" />}
                      title="Auto-start node"
                      body={`The wallet starts and stops ${profile.binaryName} when you open and close it.`}
                    />
                    <FeatureTile
                      icon={<ShieldCheck className="h-4 w-4" />}
                      title="Encrypted wallet"
                      body="Strong passphrase, stored only inside wallet.dat."
                    />
                    <FeatureTile
                      icon={<HardDriveUpload className="h-4 w-4" />}
                      title="Import backup"
                      body={`Restore wallet.dat from ${profile.displayName}-Qt or a saved backup.`}
                    />
                    <FeatureTile
                      icon={<HardDriveDownload className="h-4 w-4" />}
                      title="Optional bootstrap"
                      body="Skip the slow P2P sync with the official snapshot."
                    />
                  </>
                )}
              </div>
              {showFullNodeMigrationHint && (
                <div className="rounded-md border border-warning/40 bg-warning/10 px-3 py-3 text-xs text-fg-muted">
                  <p className="font-medium text-fg">
                    {lightWalletCopy.setupFullNodeWalletFoundTitle}
                  </p>
                  <p className="mt-1">
                    {lightWalletCopy.setupFullNodeWalletFoundBody}
                  </p>
                  <Button
                    size="sm"
                    variant="secondary"
                    className="mt-3"
                    onClick={() => void switchToFullNodeSetup()}
                  >
                    {lightWalletCopy.setupUseFullNodeCta}
                  </Button>
                </div>
              )}
              <p className="text-xs text-fg-subtle">
                Wallet mode:{" "}
                <strong className="font-medium text-fg">
                  {setupWalletMode === "light" ? "Light wallet" : "Full node"}
                </strong>
                . Change it from the wallet menu.
              </p>
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  onClick={async () => {
                    if (LIGHT_WALLET_ENABLED) {
                      await walletModeSet(setupWalletMode);
                      invalidateWalletMode();
                    }
                    setStep(setupWalletMode === "light" ? "wallet" : "daemon");
                  }}
                >
                  Continue
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setStep("advanced")}
                >
                  Advanced setup
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Back to wallet menu
                </Button>
              </div>
            </div>
          )}

          {step === "daemon" && (
            <div className="flex flex-col gap-4 text-sm">
              <div className="rounded-md border border-border bg-bg-subtle p-3">
                <div className="flex items-center gap-2 font-medium text-fg">
                  {(isConnecting || !connected) && (
                    <Loader2 className="h-4 w-4 animate-spin text-accent" />
                  )}
                  {connected && (
                    <CheckCircle2 className="h-4 w-4 text-success" />
                  )}
                  {nodeStatusLabel(nodeStatus)}
                </div>
                <p className="mt-2 text-xs text-fg-muted">
                  The wallet starts your node automatically. This may take a
                  minute while the blockchain index loads.
                </p>
                <div className="mt-2 break-all text-[11px] text-fg-subtle">
                  {config.data?.datadir}
                </div>
              </div>

              {runtime.data?.datadir_locked && (
                <div className="rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
                  Another Verium instance is using this data directory. Quit
                  Verium-Qt or any other node using the same folder, then reopen
                  the wallet.
                </div>
              )}

              {!binary.data?.manageable && !binary.isLoading && (
                <div className="rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
                  Node software is not available in this install. Re-download
                  from the official release page.
                </div>
              )}

              <div className="flex flex-wrap gap-2">
                {connected && (
                  <Button onClick={() => setStep("wallet")}>Continue</Button>
                )}
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setStep("welcome")}
                >
                  Back
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Wallet menu
                </Button>
              </div>
            </div>
          )}

          {step === "wallet" && lightWalletFlow && (
            <div className="flex flex-col gap-4">
              {setupCoinKeystoreUnreadable ? (
                <>
                  <div className="rounded-md border border-warning/40 bg-warning/10 px-3 py-3 text-xs text-fg-muted">
                    <p className="font-medium text-fg">
                      Recovery phrase required
                    </p>
                    <p className="mt-1">
                      Wallet files are on this device but the saved keys need to
                      be rebuilt. Passphrase unlock will not work — import your
                      24-word recovery phrase (or HD master key) below. Seeds
                      stay encrypted with your new passphrase; no Windows
                      Credential Manager entry is required.
                    </p>
                  </div>
                  <LightWalletSetupForm
                    mode="import"
                    onDone={() => setStep("twofa")}
                    onBack={goToHub}
                  />
                </>
              ) : (
                <>
              <div className="rounded-md border border-border bg-bg-subtle p-3 text-xs text-fg-muted">
                <p>
                  Light wallet — your keys are encrypted on this device. Balance
                  and history come from Vericonomy Electrum servers.
                </p>
              </div>
              {showFullNodeMigrationHint && walletAction === "choose" && (
                <div className="rounded-md border border-warning/40 bg-warning/10 px-3 py-3 text-xs text-fg-muted">
                  <p className="font-medium text-fg">
                    {lightWalletCopy.setupFullNodeWalletFoundTitle}
                  </p>
                  <p className="mt-1">
                    {lightWalletCopy.setupFullNodeWalletFoundBody}
                  </p>
                  {walletFile.data?.path && (
                    <p className="mt-2 break-all font-mono text-[11px] text-fg-subtle">
                      {walletFile.data.path}
                    </p>
                  )}
                  <Button
                    size="sm"
                    className="mt-3"
                    onClick={() => void switchToFullNodeSetup()}
                  >
                    {lightWalletCopy.setupUseFullNodeCta}
                  </Button>
                </div>
              )}
              {walletAction === "choose" && (
                <div className="flex flex-col gap-3">
                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <Button onClick={() => setWalletAction("create")}>
                      Create new wallet
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={() => setWalletAction("restore_phrase")}
                    >
                      Import recovery phrase
                    </Button>
                    {!setupCoinHasLightWallet && (
                      <Button
                        variant="ghost"
                        className="sm:col-span-2"
                        onClick={() => setWalletAction("unlock")}
                      >
                        Unlock existing light wallet
                      </Button>
                    )}
                  </div>
                  {showFullNodeMigrationHint && (
                    <p className="text-xs text-fg-subtle">
                      {lightWalletCopy.setupCreateNewWarn}
                    </p>
                  )}
                </div>
              )}
              {walletAction === "create" && (
                <LightWalletSetupForm
                  mode="create"
                  onDone={() => setStep("twofa")}
                  onBack={() => setWalletAction("choose")}
                />
              )}
              {walletAction === "restore_phrase" && (
                <LightWalletSetupForm
                  mode="import"
                  onDone={() => setStep("twofa")}
                  onBack={() => setWalletAction("choose")}
                />
              )}
              {walletAction === "unlock" && (
                <div className="flex flex-col gap-3">
                  {!setupCoinHasLightWallet && (
                    <p className="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-xs text-fg-muted">
                      No light wallet was found for {profile.symbol} on this
                      device. Choose <strong>Create new wallet</strong> or{" "}
                      <strong>Import recovery phrase</strong> first.
                    </p>
                  )}
                  <LightWalletSetupForm
                    mode="unlock"
                    onDone={() => setStep("twofa")}
                    onBack={() => setWalletAction("choose")}
                  />
                </div>
              )}
              {walletAction === "choose" && (
                <Button
                  size="sm"
                  variant="ghost"
                  className="self-start"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Back to wallet menu
                </Button>
              )}
                </>
              )}
            </div>
          )}

          {legacyWizardActive && (
            <LegacyUpgradeWizard
              coin={coin}
              legacyPath={walletFile.data?.legacy_wallet_path}
              connected={connected}
              onComplete={() => advanceAfterChainWalletReady()}
              onStartFresh={() => {
                setLegacyFreshStart(true);
                setWalletAction("choose");
              }}
              onBack={goToHub}
            />
          )}

          {step === "wallet" &&
            !lightWalletFlow &&
            !legacyWizardActive && (
            <div className="flex flex-col gap-4">
              <div className="rounded-md border border-border bg-bg-subtle p-3 text-xs text-fg-muted">
                {walletFile.data?.path && (
                  <div className="break-all text-[11px]">
                    {walletFile.data.path}
                  </div>
                )}
                <p className="mt-1">
                  {walletSetupModeLabel(coin, walletSetupMode)}
                </p>
                {walletInfo.data && walletSetupMode === "needs_unlock" && (
                  <p className="mt-1 text-fg">
                    Balance: {walletInfo.data.balance.toFixed(8)}{" "}
                    {profile.symbol}
                    {walletInfo.data.txcount > 0
                      ? ` · ${walletInfo.data.txcount} transactions`
                      : ""}
                  </p>
                )}
                {walletFile.data?.legacy_wallet_detected &&
                  walletFile.data.legacy_wallet_path && (
                    <p className="mt-2 rounded-md border border-accent/30 bg-accent/5 px-2 py-1.5 text-xs text-fg-muted">
                      Existing wallet found at{" "}
                      <span className="break-all font-mono text-fg">
                        {walletFile.data.legacy_wallet_path}
                      </span>
                      . Import it, or in Advanced setup set your data directory
                      to that folder. Otherwise create a new wallet for a fresh
                      start.
                    </p>
                  )}
                {walletFile.data?.is_new_install && (
                  <p className="mt-2 text-xs text-fg-muted">
                    No existing {profile.displayName} wallet was detected on
                    this Mac — you are setting up a new {profile.symbol} wallet.
                  </p>
                )}
              </div>

              {walletSetupMode === "loading" && (
                <div className="flex items-center gap-2 text-sm text-fg-muted">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Checking wallet…
                </div>
              )}

              {walletSetupMode === "offline" && (
                <div className="rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-fg-muted">
                  Go back to <strong>Start node</strong> and connect to{" "}
                  {profile.binaryName} before setting up your wallet.
                </div>
              )}

              {(walletSetupMode === "needs_unlock" ||
                walletUnlockAwaitingNode) &&
                walletAction === "unlock" && (
                  <>
                    <WalletUnlockForm
                      title="Unlock your existing wallet"
                      description={`Enter the passphrase from your previous ${profile.displayName} wallet. Your coins, addresses, and transaction history stay exactly as they are.`}
                      submitDisabled={walletUnlockAwaitingNode}
                      submitDisabledMessage="Wallet is still loading in the node. Wait for checking to finish, then unlock."
                      onUnlocked={() => setStep("bootstrap")}
                    />
                    <div className="border-t border-border pt-3">
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => setWalletAction("import")}
                      >
                        Import a different wallet.dat instead
                      </Button>
                    </div>
                  </>
                )}

              {walletSetupMode === "needs_encrypt" &&
                walletAction === "choose" && (
                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <button
                      type="button"
                      onClick={() => setWalletAction("create")}
                      className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent"
                    >
                      <span className="flex items-center gap-2 text-sm font-medium text-fg">
                        <WalletIcon className="h-4 w-4 text-accent" />
                        Create new wallet
                      </span>
                      <span className="text-xs text-fg-muted">
                        First time on this machine — choose a passphrase and
                        encrypt a fresh wallet.dat.
                      </span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setWalletAction("import")}
                      className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent"
                    >
                      <span className="flex items-center gap-2 text-sm font-medium text-fg">
                        <HardDriveUpload className="h-4 w-4 text-accent" />
                        Import wallet.dat
                      </span>
                      <span className="text-xs text-fg-muted">
                        Restore a backup from {profile.displayName}-Qt, this
                        app, or another computer.
                      </span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setWalletAction("restore_phrase")}
                      className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent sm:col-span-2"
                    >
                      <span className="flex items-center gap-2 text-sm font-medium text-fg">
                        <ShieldCheck className="h-4 w-4 text-accent" />
                        Restore from recovery phrase
                      </span>
                      <span className="text-xs text-fg-muted">
                        Enter your 24-word BIP39 mnemonic to recover an HD
                        wallet.
                      </span>
                    </button>
                  </div>
                )}

              {walletAction === "create" &&
                walletSetupMode === "needs_encrypt" && (
                  <>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="self-start"
                      onClick={() => setWalletAction("choose")}
                    >
                      Back
                    </Button>
                    <WalletCreateForm
                      onCreated={(_result, passphrase) => {
                        setPendingPassphrase(passphrase);
                        setStep("recovery");
                      }}
                      onAlreadyEncrypted={() => {
                        void walletInfo.refetch();
                      }}
                    />
                  </>
                )}

              {walletAction === "restore_phrase" && (
                <RestoreFromPhraseForm
                  onRestored={() => {
                    setWalletAction("unlock");
                    void walletInfo.refetch();
                  }}
                />
              )}

              {walletAction === "import" && (
                <WalletImportForm
                  onRestored={() => {
                    setWalletAction("unlock");
                    void walletInfo.refetch();
                    void walletFile.refetch();
                  }}
                  onCancel={
                    walletSetupMode === "needs_unlock"
                      ? () => setWalletAction("unlock")
                      : walletSetupMode === "needs_encrypt"
                        ? () => setWalletAction("choose")
                        : undefined
                  }
                />
              )}
              {walletAction === "choose" && (
                <Button
                  size="sm"
                  variant="ghost"
                  className="self-start"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Back to wallet menu
                </Button>
              )}
            </div>
          )}

          {step === "recovery" && (
            <div className="flex flex-col gap-4">
              <div className="flex items-center gap-2 text-sm text-fg">
                <ShieldCheck className="h-4 w-4 text-accent" />
                Save your recovery phrase
              </div>
              <p className="text-sm text-fg-muted">
                This phrase is separate from your wallet passphrase: it is the
                HD master key for your addresses. Save it on paper so you can
                restore on a new device if you lose your passphrase, computer,
                or <span className="font-mono text-xs">wallet.dat</span>.
                Vericonomy cannot look it up for you.
              </p>
              <RecoveryPhraseWizard
                onComplete={async (phrase) => {
                  await applyRecovery.mutateAsync({
                    phrase,
                    unlock: pendingPassphrase ?? undefined,
                  });
                }}
              />
              {applyRecovery.isPending && (
                <p className="flex items-center gap-2 text-xs text-fg-muted">
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  Applying recovery seed to your wallet…
                </p>
              )}
              {recoveryError && (
                <p className="text-xs text-danger">{recoveryError}</p>
              )}
              {applyRecovery.isSuccess && (
                <p className="text-xs text-success">{applyRecovery.data}</p>
              )}
            </div>
          )}

          {step === "twofa" && (
            <div className="flex flex-col gap-4">
              <div className="flex items-center gap-2 text-sm text-fg">
                <Smartphone className="h-4 w-4 text-accent" />
                Two-factor authentication (app-wide)
              </div>
              <p className="text-sm text-fg-muted">
                Protect sends, passphrase changes, and sensitive actions with a
                code from an authenticator app. This applies to{" "}
                <strong className="font-medium text-fg">
                  both Verium and Vericoin
                </strong>
                — your {profile.symbol} passphrase and recovery phrase remain
                separate per chain.
              </p>
              <TwoFactorEnrollmentPanel
                autoStartEnrollment
                onEnabled={() =>
                  setStep(lightSetupActive ? "done" : "bootstrap")
                }
              />
              <div className="flex flex-wrap gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() =>
                    setStep(lightSetupActive ? "done" : "bootstrap")
                  }
                >
                  Skip for now
                </Button>
                {twoFa.data?.enabled && (
                  <Button
                    size="sm"
                    onClick={() =>
                      setStep(lightSetupActive ? "done" : "bootstrap")
                    }
                  >
                    Continue
                  </Button>
                )}
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Wallet menu
                </Button>
              </div>
            </div>
          )}

          {step === "bootstrap" && !lightSetupActive && isTestNetwork && (
            <div className="flex flex-col gap-3 text-sm text-fg-muted">
              <div className="flex items-center gap-2 text-fg">
                <HardDriveDownload className="h-4 w-4" />
                Binarytest network
              </div>
              <p>
                You are on the isolated Binary Chain (DACE) test network. There
                is no canonical snapshot CDN — the chain starts at genesis and
                grows as you mine VRM / stake VRC locally.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" onClick={() => setStep("done")}>
                  Continue
                </Button>
              </div>
            </div>
          )}

          {step === "bootstrap" && !lightSetupActive && !isTestNetwork && (
            <div className="flex flex-col gap-3 text-sm text-fg-muted">
              <div className="flex items-center gap-2 text-fg">
                <HardDriveDownload className="h-4 w-4" />
                Chain bootstrap (optional)
              </div>
              <p>
                Fresh installs sync much faster from the official chain snapshot
                than over P2P. The daemon downloads, extracts, and restarts
                automatically.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" onClick={() => setBootstrapOpen(true)}>
                  Import bootstrap now
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setStep("done")}
                >
                  Skip
                </Button>
              </div>
              <BootstrapDialog
                coin={coin}
                open={bootstrapOpen}
                onClose={() => {
                  setBootstrapOpen(false);
                  setStep("done");
                }}
              />
            </div>
          )}

          {step === "done" && (
            <div className="flex flex-col gap-3 text-sm">
              <div className="flex items-center gap-2 text-success">
                <WalletIcon className="h-4 w-4" /> All set.
              </div>
              <p className="text-fg-muted">
                You're ready to go. Settings, wallet backup, and advanced
                options are available from the sidebar.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" onClick={() => void finish()}>
                  {anyEnabledCoinSetupIncomplete(enabledCoins, {
                    ...prefs,
                    ...coinSetupCompletePatch(coin, prefs),
                  })
                    ? "Finish and return to wallet menu"
                    : "Open dashboard"}
                </Button>
                <Button size="sm" variant="ghost" onClick={goToHub}>
                  Wallet menu
                </Button>
              </div>
            </div>
          )}

          {step === "advanced" && (
            <div className="flex flex-col gap-4 text-sm">
              <p className="text-fg-muted">
                Point the app at an existing data directory or remote node. Most
                users should use the simple flow.
              </p>
              <div className="flex flex-col gap-1">
                <label className="text-fg-muted text-xs">Data directory</label>
                <div className="flex gap-2">
                  <input
                    value={datadirDraft}
                    onChange={(e) => setDatadirDraft(e.target.value)}
                    className="h-9 flex-1 rounded-md border border-border bg-bg-subtle px-3 text-xs outline-none focus:border-accent"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={async () => {
                      const picked = await openDialog({
                        directory: true,
                        multiple: false,
                      });
                      if (typeof picked === "string") {
                        setDatadirDraft(picked);
                      }
                    }}
                  >
                    Browse
                  </Button>
                </div>
              </div>

              <DaemonConnectionPanel
                coin={coin}
                config={config.data}
                mode="settings"
              />

              <div className="flex flex-wrap gap-2">
                <Button
                  size="sm"
                  onClick={async () => {
                    await saveConfig.mutateAsync({ datadir: datadirDraft });
                    setStep("daemon");
                  }}
                  disabled={!datadirDraft || saveConfig.isPending}
                >
                  Save and go to start node
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setStep("welcome")}
                >
                  Back to welcome
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={onboardingBusy}
                  onClick={goToHub}
                >
                  Wallet menu
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
