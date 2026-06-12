/** User-facing copy for light vs full-node wallet modes. */

export const lightWalletCopy = {
  unlockUnavailableTitle: 'Light wallet not set up',
  unlockUnavailableDescription:
    'Create or unlock your light wallet to use this page. Your keys stay on this device; balance and history come from Vericonomy servers.',
  unlockUnavailableCta: 'Open setup',
  dashboardNoWalletTitle: 'No light wallet on this chain',
  dashboardNoWalletBody:
    'Light mode is on, but this device has no encrypted wallet saved for this chain yet. If you already imported a phrase, try unlocking from the dashboard card below. Otherwise import from setup, or switch back to full-node mode in Settings if your funds are in wallet.dat.',
  dashboardNoWalletCta: 'Set up or import light wallet',
  unlockVerifying: 'Checking passphrase…',
  unlockApplied: 'Unlock applied',
  unlockRefreshingBalance:
    'Finishing address scan on Vericonomy servers — first unlock can take several minutes. Your confirmed balance may appear before the scan completes; you can send once it shows.',
  modeMismatchTitle: 'Light wallet found — switch mode',
  modeMismatchDescription:
    'Your {coin} light wallet is saved on this device, but the app is in full-node mode. Switch to Light wallet in Settings to use it.',
  modeMismatchMiningTitle: 'Full-node wallet not loaded',
  modeMismatchMiningDescription:
    'Solo mining needs a full-node wallet (wallet.dat) unlocked on veriumd. This device only has a {coin} light wallet saved. Switch to Light wallet mode for pool mining with that wallet, or set up a full-node wallet in Setup to solo mine.',
  modeMismatchMiningPoolHint:
    'You can still pool mine below — enter your VRM payout address manually if it is not filled in automatically.',
  modeMismatchCta: 'Open Settings → Wallet mode',
  fullNodeOnlyToast: 'This page requires a local full node. Switch to Full node mode in Settings.',
  setupWelcomeLight:
    'Connect to Vericonomy light wallet servers. No blockchain download — your recovery phrase and passphrase stay on this device.',
  setupMobileOnly:
    'This device uses light wallet mode only. Your keys stay encrypted here; balance and history sync via Vericonomy Electrum servers.',
  setupHubMobileOnly:
    'Pick Verium or Vericoin, then set up a light wallet for each chain. No local node or blockchain sync on mobile.',
  mobileOnboardingWelcome:
    'Your keys stay encrypted on this device. Balance and history sync through Vericonomy Electrum servers — no blockchain download required.',
  mobileFundsDisclaimer:
    'This is wallet software for real mainnet assets. Only store amounts you are prepared to lose if something goes wrong. Always back up your recovery phrase before sending funds.',
  mobileOnboardingCreate:
    'Generate a new wallet with a recovery phrase you write down and store safely.',
  mobileOnboardingImport:
    'Already have a wallet? Restore it with your 24-word recovery phrase and passphrase.',
  sendAvailableIncludesChange:
    'Available balance includes unconfirmed change from your sends until the next block confirms.',
  balanceUpdating: 'Updating balance…',
  balanceUpToDate: 'Balance up to date',
  balanceCheckingAddresses: 'Checking all wallet addresses…',
  rescanTitle: 'Rescan addresses',
  rescanDescription:
    'Re-discovers receive and change addresses on Vericonomy servers and rebuilds your local balance cache. Use this if balance looks too low compared to a full-node wallet with the same recovery phrase.',
  rescanUnlockHint: 'Unlock the wallet on that chain before rescanning.',
  rescanSuccess: 'Address rescan started. Balance may update over the next few minutes.',
  mobileOnboardingRestoreHint:
    'We found wallet data on this device that could not be unlocked. Import your recovery phrase to restore access.',
  mobileOnboardingNotSetUp: 'Not set up yet',
  mobileOnboardingReady: 'Ready to use',
  setupWelcomeFull:
    'This wallet ships with a bundled node. Your keys can live in wallet.dat on this machine, with optional blockchain sync.',
  setupFullNodeRecommended:
    'Exchange-grade default: your wallet validates the chain locally. Recommended for custody, high-value holdings, and institutional workflows.',
  lightConvenienceWarning:
    'Light mode trusts Vericonomy Electrum servers for balance and UTXO data. Not recommended for high-value or exchange custody workflows.',
  lightModeBanner:
    'Light wallet mode — convenience tier. Balance and UTXOs are reported by index servers. Use full-node mode for exchange-grade assurance.',
  migrationFullToLightTitle: 'Switch to light wallet',
  migrationFullToLightBody:
    'Export your recovery phrase before switching. wallet.dat cannot be imported directly into light mode — use your seed to restore balances.',
  migrationLightToFullTitle: 'Switch to full node',
  migrationLightToFullBody:
    'Import your recovery phrase into the local node via Security after switching. The chain must sync before balances match.',
  backupLightHint:
    'Back up your recovery phrase and remember your passphrase. Light wallets do not use wallet.dat.',
  backupFullHint: 'Scheduled backups copy wallet.dat from your local node data folder.',
  setupFullNodeWalletFoundTitle: 'Existing full-node wallet on this computer',
  setupFullNodeWalletFoundBody:
    'Light wallets cannot open wallet.dat directly. To keep your balance, use full-node mode and unlock with your existing passphrase — or export your recovery phrase (or HD master key) from Security → Export recovery phrase and import it here.',
  setupUseFullNodeCta: 'Use my existing full-node wallet',
  setupCreateNewWarn:
    'Create new wallet starts a fresh light wallet with no coins. Only choose this if you want a new address.',
  importLightTitle: 'Import recovery phrase or HD master key',
  importLightBody:
    "Replace this chain's light wallet with a 24-word BIP39 phrase or the HD master key exported from Security → Export recovery phrase. Your previous light-wallet keys for this chain are discarded.",
  importLightSuccess:
    'Your light wallet is saved on this device. Balance and history will fill in as the server scan completes.',
  importLightScanNote:
    'Scanning your addresses on Vericonomy servers usually takes 15–45 seconds for HD master keys. You can open the dashboard now — balance updates automatically.',
  importLightSteps: [
    'Validating recovery material',
    'Saving encrypted wallet on this device',
    'Preparing your address list',
  ] as const,
  importLightSyncStep: 'Checking balance on Vericonomy servers',
} as const;
