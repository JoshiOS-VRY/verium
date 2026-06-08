//! Onboarding intent + checkpoint model shared by the setup wizard and the
//! Rust backend. The checkpoint is persisted per coin in `prefs.json` so a
//! mid-flow refresh, crash, or app restart resumes at the same step instead of
//! dropping the user back at the hub (and losing in-memory state such as a
//! pending passphrase).

use serde::{Deserialize, Serialize};

/// Why the user is in the setup wizard for a given coin. Classified once, up
/// front, from the detected on-disk state so the wizard can route to the right
/// flow instead of branching implicitly inside each step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingIntent {
    /// No keys for either mode and no legacy wallet on disk.
    FreshInstall,
    /// A legacy Qt `wallet.dat` exists outside the configured datadir.
    LegacyUpgrade,
    /// A `wallet.dat` exists in the configured datadir (locked or unlocked).
    ExistingUnlock,
    /// A light keystore already exists for this coin.
    LightContinue,
    /// Keys exist only for the other mode; user is moving between modes.
    CrossModeMigration,
}

impl OnboardingIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            OnboardingIntent::FreshInstall => "fresh_install",
            OnboardingIntent::LegacyUpgrade => "legacy_upgrade",
            OnboardingIntent::ExistingUnlock => "existing_unlock",
            OnboardingIntent::LightContinue => "light_continue",
            OnboardingIntent::CrossModeMigration => "cross_mode_migration",
        }
    }
}

/// Lifecycle phase of onboarding for a coin. `Complete` means the wizard
/// finished and the chain is usable; `ready` on the wallet profile still
/// gates dashboard access on actual key presence for the active mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingPhase {
    /// Wizard has not been entered (or no checkpoint persisted).
    #[default]
    NotStarted,
    /// Wizard is mid-flow; `OnboardingCheckpoint::step` says where.
    InProgress,
    /// Wizard finished for this coin.
    Complete,
}

/// Durable, resumable onboarding position for one coin. Stored under
/// `prefs.onboarding_by_coin[coin]`. Intentionally free of secrets — the
/// passphrase lives in the OS keychain / encrypted session blob, not here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnboardingCheckpoint {
    #[serde(default)]
    pub phase: OnboardingPhase,
    /// Classified intent (`fresh_install`, `legacy_upgrade`, ...). `None`
    /// before the user picks a coin / the wizard classifies state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Step id the wizard should resume at (`welcome`, `daemon`, `wallet`,
    /// `recovery`, `hd_upgrade`, `twofa`, `bootstrap`, `done`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// Detected legacy `wallet.dat` path captured at classification time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_path: Option<String>,
}

impl OnboardingCheckpoint {
    pub fn complete() -> Self {
        Self {
            phase: OnboardingPhase::Complete,
            ..Default::default()
        }
    }

    pub fn is_complete(&self) -> bool {
        self.phase == OnboardingPhase::Complete
    }
}
