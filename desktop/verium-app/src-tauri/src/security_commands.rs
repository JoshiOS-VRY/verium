//! Tauri commands for security, recovery, 2FA, passkeys, hardware wallets, etc.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::audit_log::{self, AuditEntry};
use crate::auto_lock::{self, AutoLockConfig};
use crate::backup_scheduler::{self, BackupHealth, BackupSchedulerConfig, ScheduledBackupResult};
use crate::coin_profile::{parse_coin_id, CoinId};
use crate::commands;
use crate::error::{AppError, AppResult};
use crate::hardware_wallet::{self, HardwareWalletConfig, HardwareVendor, PsbtSendResult};
use crate::installer_verify::{self, VerificationStatus};
use crate::multisig::{self, MultisigWalletConfig};
use crate::passkey::{self, PasskeyConfig};
use crate::receive_requests::{self, ReceiveRequest};
use crate::recovery::{self, RecoveryPhraseBundle};
use crate::slip39_recovery::{self, ShamirSplitResult};
use crate::spending_controls::{self, SpendingControlsConfig, SpendCheckResult};
use crate::state::AppState;
use crate::two_factor::{self, TwoFactorConfig, TwoFactorEnrollment};

// ── Recovery phrase ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn recovery_generate_mnemonic() -> AppResult<RecoveryPhraseBundle> {
    recovery::generate_mnemonic()
}

#[tauri::command]
pub fn recovery_validate_mnemonic(phrase: String) -> AppResult<bool> {
    recovery::validate_mnemonic(&phrase)
}

#[tauri::command]
pub fn recovery_verification_indices(word_count: u32) -> AppResult<Vec<usize>> {
    Ok(recovery::verification_indices(word_count, 3))
}

#[tauri::command]
pub fn recovery_verify_words(
    phrase: String,
    indices: Vec<usize>,
    answers: Vec<String>,
) -> AppResult<bool> {
    Ok(recovery::verify_words_at_indices(&phrase, &indices, &answers))
}

const RECOVERY_UNLOCK_SECONDS: i64 = 600;

fn wallet_info_is_locked(info: &serde_json::Value) -> bool {
    let Some(until) = info.get("unlocked_until").and_then(|v| v.as_i64()) else {
        return false;
    };
    if until == 0 {
        return true;
    }
    let now = chrono::Utc::now().timestamp();
    until <= now
}

#[tauri::command]
pub async fn recovery_apply_hd_seed(
    state: State<'_, AppState>,
    coin: String,
    phrase: String,
    bip39_passphrase: Option<String>,
    unlock_passphrase: Option<String>,
    totp_code: Option<String>,
) -> AppResult<String> {
    crate::security_policy::require_gated_action("restore_wallet", totp_code.as_deref())?;
    let coin = parse_coin_id(&coin)?;
    let wif = recovery::master_xpriv_to_wif(coin, &phrase, bip39_passphrase.as_deref())?;
    let client = state.rpc_client(coin).await?;
    let info: serde_json::Value = client.call("getwalletinfo", json!([])).await?;

    if wallet_info_is_locked(&info) {
        let pass = unlock_passphrase
            .as_deref()
            .filter(|p| !p.is_empty())
            .ok_or_else(|| {
                AppError::other(
                    "Wallet is locked. Enter your wallet passphrase to apply the recovery phrase.",
                )
            })?;
        client
            .call_no_result(
                "walletpassphrase",
                json!([pass, RECOVERY_UNLOCK_SECONDS]),
            )
            .await?;
    }

    client
        .call_no_result("sethdseed", json!([true, wif]))
        .await?;

    let after: serde_json::Value = client.call("getwalletinfo", json!([])).await?;
    if after.get("hdseedid").is_none() {
        return Err(AppError::other(
            "Recovery seed was not applied (wallet is still non-HD). Back up wallet.dat, then try again or restore on a new wallet file.",
        ));
    }

    if let Some(pass) = unlock_passphrase.as_deref().filter(|p| !p.is_empty()) {
        if let Err(e) = crate::mnemonic_backup::save(coin, &phrase, pass) {
            tracing::warn!("could not store encrypted recovery phrase backup: {e}");
        }
    }

    audit_log::append("set_hd_seed", "Applied BIP39 HD seed via sethdseed", Some(coin.as_str()))?;
    Ok("HD seed applied. Back up wallet.dat immediately.".into())
}

#[tauri::command]
pub async fn recovery_wallet_is_hd(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<bool> {
    let coin = parse_coin_id(&coin)?;
    wallet_is_hd(&state, coin).await
}

async fn wallet_is_hd(state: &AppState, coin: CoinId) -> AppResult<bool> {
    let client = state.rpc_client(coin).await?;
    let info: serde_json::Value = client.call("getwalletinfo", json!([])).await?;
    Ok(info.get("hdseedid").is_some())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecoveryExportResult {
    Mnemonic {
        mnemonic: String,
        word_count: u32,
    },
    HdMasterXprv {
        xprv: String,
        message: String,
    },
}

#[tauri::command]
pub fn recovery_mnemonic_backup_exists(coin: String) -> AppResult<bool> {
    let coin = parse_coin_id(&coin)?;
    Ok(crate::mnemonic_backup::exists(coin))
}

#[tauri::command]
pub async fn recovery_export_seed(
    state: State<'_, AppState>,
    coin: String,
    wallet_passphrase: String,
    totp_code: Option<String>,
) -> AppResult<RecoveryExportResult> {
    crate::security_policy::require_gated_action("show_recovery_phrase", totp_code.as_deref())?;
    let coin = parse_coin_id(&coin)?;
    if wallet_passphrase.is_empty() {
        return Err(AppError::other(
            "Wallet passphrase is required to export your recovery material.",
        ));
    }

    let prefs = crate::prefs::load().await?;
    if prefs.wallet_mode.is_light() {
        if !crate::wallet::keystore::wallet_exists(coin)? {
            return Err(AppError::other("No light wallet found for this chain."));
        }
        if !crate::wallet::keystore::is_unlocked(coin)? {
            crate::wallet::keystore::unlock_wallet(
                coin,
                &wallet_passphrase,
                RECOVERY_UNLOCK_SECONDS as u32,
            )?;
        }
        let secret = crate::wallet::keystore::unlocked_mnemonic(coin, &wallet_passphrase)?;
        if crate::wallet::hd::is_hd_master_secret(coin, &secret) {
            audit_log::append(
                "export_hd_xprv",
                "Exported light wallet HD master xprv",
                Some(coin.as_str()),
            )?;
            return Ok(RecoveryExportResult::HdMasterXprv {
                xprv: secret,
                message: "Light wallet was imported from an HD master key (xprv), not a BIP39 phrase."
                    .into(),
            });
        }
        let word_count = secret.split_whitespace().count() as u32;
        audit_log::append(
            "export_recovery_phrase",
            "Exported light wallet BIP39 phrase",
            Some(coin.as_str()),
        )?;
        return Ok(RecoveryExportResult::Mnemonic {
            mnemonic: secret,
            word_count,
        });
    }

    if let Some(phrase) = crate::mnemonic_backup::load(coin, &wallet_passphrase)? {
        let word_count = phrase.split_whitespace().count() as u32;
        audit_log::append(
            "export_recovery_phrase",
            "Exported BIP39 phrase from encrypted backup",
            Some(coin.as_str()),
        )?;
        return Ok(RecoveryExportResult::Mnemonic {
            mnemonic: phrase,
            word_count,
        });
    }

    if wallet_is_hd(&state, coin).await? {
        let xprv =
            crate::hd_wallet_export::dump_hd_master_xprv(&state, coin, &wallet_passphrase).await?;
        audit_log::append(
            "export_hd_xprv",
            "Exported full-node HD master xprv (no BIP39 backup on file)",
            Some(coin.as_str()),
        )?;
        return Ok(RecoveryExportResult::HdMasterXprv {
            xprv,
            message: "No BIP39 phrase was stored when this wallet was upgraded. Use this xprv to import into light mode, or set up a new phrase under Security (back up wallet.dat first).".into(),
        });
    }

    Err(AppError::other(
        "This wallet is not HD. Upgrade to HD under Security (back up wallet.dat first), or continue using full-node mode with wallet.dat backups.",
    ))
}

// ── 2FA ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn two_factor_status() -> AppResult<TwoFactorConfig> {
    two_factor::status()
}

#[tauri::command]
pub fn two_factor_start_enrollment() -> AppResult<TwoFactorEnrollment> {
    two_factor::start_enrollment()
}

#[tauri::command]
pub fn two_factor_confirm_enrollment(
    code: String,
    enrollment_secret: Option<String>,
) -> AppResult<()> {
    two_factor::confirm_enrollment(&code, enrollment_secret.as_deref())
}

#[tauri::command]
pub fn two_factor_pending_otpauth_uri() -> AppResult<Option<String>> {
    two_factor::pending_otpauth_uri()
}

#[tauri::command]
pub fn two_factor_verify(code: String) -> AppResult<bool> {
    two_factor::verify(&code)
}

#[tauri::command]
pub fn two_factor_disable(code: String) -> AppResult<()> {
    two_factor::disable(&code)
}

#[tauri::command]
pub fn two_factor_is_gated(action: String, amount: Option<f64>, coin: String) -> AppResult<bool> {
    two_factor::is_action_gated(&action, amount, &coin)
}

#[tauri::command]
pub fn two_factor_save_config(partial: PartialTwoFactorConfig) -> AppResult<TwoFactorConfig> {
    let mut config = two_factor::load()?;
    if let Some(v) = partial.send_threshold_vrm {
        config.send_threshold_vrm = Some(v);
    }
    if let Some(v) = partial.send_threshold_vrc {
        config.send_threshold_vrc = Some(v);
    }
    if let Some(v) = partial.gated_actions {
        config.gated_actions = v;
    }
    two_factor::save(&config)?;
    Ok(config)
}

#[derive(Debug, Deserialize)]
pub struct PartialTwoFactorConfig {
    pub send_threshold_vrm: Option<f64>,
    pub send_threshold_vrc: Option<f64>,
    pub gated_actions: Option<Vec<String>>,
}

// ── Passkey / PIN ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn passkey_status() -> AppResult<PasskeyConfig> {
    passkey::status()
}

#[tauri::command]
pub fn passkey_gate_required() -> AppResult<bool> {
    passkey::gate_required()
}

#[tauri::command]
pub fn passkey_enroll_pin(pin: String) -> AppResult<()> {
    passkey::enroll_pin(&pin)
}

#[tauri::command]
pub fn passkey_verify_pin(pin: String) -> AppResult<bool> {
    passkey::verify_pin(&pin)
}

#[tauri::command]
pub fn passkey_disable(pin: String) -> AppResult<()> {
    passkey::disable(&pin)
}

// ── Auto-lock ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn auto_lock_get_config() -> AppResult<AutoLockConfig> {
    auto_lock::load_config()
}

#[tauri::command]
pub fn auto_lock_set_config(config: AutoLockConfig) -> AppResult<()> {
    auto_lock::save_config(&config)
}

#[tauri::command]
pub fn auto_lock_record_activity() -> AppResult<()> {
    auto_lock::record_activity();
    Ok(())
}

#[tauri::command]
pub fn auto_lock_should_lock() -> AppResult<bool> {
    let config = auto_lock::load_config()?;
    Ok(auto_lock::should_auto_lock(&config))
}

// ── Audit log ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn audit_log_list(limit: Option<usize>) -> AppResult<Vec<AuditEntry>> {
    audit_log::list(limit)
}

#[tauri::command]
pub fn audit_log_export() -> AppResult<String> {
    audit_log::export_json()
}

#[tauri::command]
pub fn audit_log_record(action: String, detail: String, coin: Option<String>) -> AppResult<AuditEntry> {
    audit_log::append(&action, &detail, coin.as_deref())
}

// ── Receive requests (encrypted) ─────────────────────────────────────────────

#[tauri::command]
pub fn receive_requests_list(coin: String) -> AppResult<Vec<ReceiveRequest>> {
    let coin = parse_coin_id(&coin)?;
    receive_requests::list(coin)
}

#[tauri::command]
pub fn receive_requests_append(coin: String, entry: ReceiveRequest) -> AppResult<ReceiveRequest> {
    let coin = parse_coin_id(&coin)?;
    receive_requests::append(coin, entry)
}

#[tauri::command]
pub fn receive_requests_delete(coin: String, id: String) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    receive_requests::delete_entry(coin, &id)
}

// ── Hardware wallets ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn hardware_wallet_list() -> AppResult<Vec<HardwareWalletConfig>> {
    hardware_wallet::list_wallets()
}

#[tauri::command]
pub fn hardware_wallet_add(config: HardwareWalletConfig) -> AppResult<HardwareWalletConfig> {
    hardware_wallet::add_wallet(config)
}

#[tauri::command]
pub fn hardware_wallet_remove(id: String) -> AppResult<()> {
    hardware_wallet::remove_wallet(&id)
}

#[tauri::command]
pub fn hardware_wallet_detect() -> AppResult<Vec<HardwareVendor>> {
    Ok(hardware_wallet::detect_devices())
}

#[tauri::command]
pub async fn hardware_wallet_import_xpub(
    state: State<'_, AppState>,
    coin: String,
    xpub: String,
    label: String,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    let client = state.rpc_client(coin).await?;
    hardware_wallet::import_xpub_watchonly(&client, &xpub, &label).await?;
    audit_log::append("hw_import_xpub", &format!("Imported xpub {label}"), Some(coin.as_str()))?;
    Ok(())
}

#[tauri::command]
pub async fn hardware_wallet_send_psbt(
    state: State<'_, AppState>,
    coin: String,
    outputs: serde_json::Map<String, serde_json::Value>,
    fee_rate: Option<f64>,
    totp_code: Option<String>,
    wallet_passphrase: Option<String>,
    extra_confirmed: Option<bool>,
) -> AppResult<PsbtSendResult> {
    let coin = parse_coin_id(&coin)?;
    let coin_str = coin.as_str();
    let mut output_pairs: Vec<(String, f64)> = Vec::new();
    for (addr, val) in &outputs {
        let amount = val
            .as_f64()
            .ok_or_else(|| AppError::other(format!("invalid amount for {addr}")))?;
        crate::wallet::address::validate_send_address(coin, addr)?;
        output_pairs.push((addr.clone(), amount));
    }
    crate::security_policy::authorize_send(
        &state,
        coin,
        totp_code.as_deref(),
        wallet_passphrase.as_deref(),
    )
    .await?;
    crate::security_policy::require_multi_send_allowed(
        coin_str,
        &output_pairs,
        extra_confirmed.unwrap_or(false),
    )?;
    let prefs = crate::prefs::load().await?;
    let pass = wallet_passphrase.unwrap_or_default();
    if prefs.wallet_mode.is_light() {
        return crate::wallet::psbt_light::build_hw_psbt(&state, coin, outputs, fee_rate, &pass).await;
    }
    let client = state.rpc_client(coin).await?;
    hardware_wallet::send_via_psbt(&client, outputs, fee_rate).await
}

#[tauri::command]
pub async fn hardware_wallet_finalize_psbt(
    state: State<'_, AppState>,
    coin: String,
    psbt_base64: String,
    totp_code: Option<String>,
    wallet_passphrase: Option<String>,
) -> AppResult<String> {
    let coin = parse_coin_id(&coin)?;
    crate::security_policy::authorize_send(
        &state,
        coin,
        totp_code.as_deref(),
        wallet_passphrase.as_deref(),
    )
    .await?;
    let prefs = crate::prefs::load().await?;
    let txid = if prefs.wallet_mode.is_light() {
        crate::wallet::psbt_light::finalize_and_broadcast_light(&state, coin, &psbt_base64).await?
    } else {
        let client = state.rpc_client(coin).await?;
        hardware_wallet::finalize_and_broadcast(&client, &psbt_base64).await?
    };
    audit_log::append("hw_send", &format!("Broadcast PSBT tx {txid}"), Some(coin.as_str()))?;
    Ok(txid)
}

// ── Multisig ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn multisig_list() -> AppResult<Vec<MultisigWalletConfig>> {
    multisig::list()
}

#[tauri::command]
pub fn multisig_save(wallet: MultisigWalletConfig) -> AppResult<MultisigWalletConfig> {
    multisig::save_wallet(wallet)
}

#[tauri::command]
pub fn multisig_remove(id: String) -> AppResult<()> {
    multisig::remove(&id)
}

#[tauri::command]
pub async fn multisig_create_address(
    state: State<'_, AppState>,
    coin: String,
    required: u32,
    pubkeys: Vec<String>,
    label: String,
) -> AppResult<String> {
    let coin = parse_coin_id(&coin)?;
    let client = state.rpc_client(coin).await?;
    let addr = multisig::create_multisig_address(&client, required, pubkeys, &label).await?;
    audit_log::append("multisig_create", &format!("Created {label} at {addr}"), Some(coin.as_str()))?;
    Ok(addr)
}

// ── Spending controls ────────────────────────────────────────────────────────

#[tauri::command]
pub fn spending_controls_get() -> AppResult<SpendingControlsConfig> {
    spending_controls::load()
}

#[tauri::command]
pub fn spending_controls_save(config: SpendingControlsConfig) -> AppResult<()> {
    spending_controls::save_ui_settings(&config)
}

async fn wallet_has_sent_to(state: &AppState, coin: CoinId, address: &str) -> bool {
    let address = address.trim();
    if address.is_empty() {
        return false;
    }
    let client = match state.rpc_client(coin).await {
        Ok(client) => client,
        Err(_) => return false,
    };
    let txs: Value = match client
        .call("listtransactions", json!(["*", 2000, 0]))
        .await
    {
        Ok(value) => value,
        Err(_) => return false,
    };
    let Some(items) = txs.as_array() else {
        return false;
    };
    items.iter().any(|tx| {
        tx.get("category").and_then(Value::as_str) == Some("send")
            && tx
                .get("address")
                .and_then(Value::as_str)
                .is_some_and(|addr| addr.eq_ignore_ascii_case(address))
    })
}

#[tauri::command]
pub async fn spending_controls_check_send(
    state: State<'_, AppState>,
    amount: f64,
    coin: String,
    address: String,
) -> AppResult<SpendCheckResult> {
    let coin_id = parse_coin_id(&coin)?;
    let address = address.trim().to_string();
    let wallet_already_sent = wallet_has_sent_to(state.inner(), coin_id, &address).await;
    spending_controls::check_spend_allowed(amount, &coin, &address, wallet_already_sent)
}

#[tauri::command]
pub fn spending_controls_record_send(
    amount: f64,
    coin: String,
    address: String,
) -> AppResult<()> {
    spending_controls::record_spend(amount, &coin, &address)
}

#[tauri::command]
pub fn spending_controls_check_allowlist(
    address: String,
    allowlist: Vec<String>,
) -> AppResult<bool> {
    Ok(spending_controls::check_allowlist(&address, &allowlist))
}

// ── Backup scheduler ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn backup_scheduler_get_config() -> AppResult<BackupSchedulerConfig> {
    backup_scheduler::load_config()
}

#[tauri::command]
pub fn backup_scheduler_save_config(config: BackupSchedulerConfig) -> AppResult<BackupSchedulerConfig> {
    let mut existing = backup_scheduler::load_config()?;
    existing.enabled = config.enabled;
    existing.daily_retention = config.daily_retention;
    existing.monthly_retention = config.monthly_retention;
    if config.cloud_folder.is_some() {
        existing.cloud_folder = config.cloud_folder;
    }
    if config.last_run_at.is_some() {
        existing.last_run_at = config.last_run_at;
    }
    // interval_hours is updated only via backup_scheduler_set_interval.
    backup_scheduler::save_config(&existing)?;
    Ok(existing)
}

#[tauri::command]
pub fn backup_scheduler_set_interval(interval_hours: u32) -> AppResult<BackupSchedulerConfig> {
    let mut config = backup_scheduler::load_config()?;
    config.interval_hours = interval_hours;
    backup_scheduler::save_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub fn backup_health() -> AppResult<BackupHealth> {
    backup_scheduler::health_status()
}

#[tauri::command]
pub async fn backup_run_now(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<String> {
    let coin = parse_coin_id(&coin)?;
    let cfg = state.config(coin).await?;
    let dest = backup_scheduler::auto_backup_path(&cfg, coin)?;
    commands::backup_wallet_to_path(state.inner(), coin, &cfg, &dest).await?;
    backup_scheduler::register_backup_hash(&dest)?;
    let config = backup_scheduler::load_config()?;
    backup_scheduler::prune_old_backups(coin, &cfg, &config)?;
    backup_scheduler::touch_last_run_at()?;
    let path = dest.display().to_string();
    if let Err(e) = audit_log::append("backup", &format!("Manual backup to {path}"), Some(coin.as_str()))
    {
        tracing::warn!("audit log append failed: {e}");
    }
    Ok(path)
}

#[tauri::command]
pub async fn backup_run_scheduled(
    state: State<'_, AppState>,
    coins: Vec<String>,
) -> AppResult<ScheduledBackupResult> {
    let config = backup_scheduler::load_config()?;
    if !backup_scheduler::is_backup_due(&config) {
        return Ok(ScheduledBackupResult {
            ran: false,
            paths: Vec::new(),
        });
    }

    let mut paths = Vec::new();
    for coin_str in coins {
        let coin = parse_coin_id(&coin_str)?;
        let cfg = match state.config(coin).await {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!("scheduled backup skipped for {}: {e}", coin.as_str());
                continue;
            }
        };
        if crate::config::resolve_wallet_dat_path(coin, &cfg).is_none() {
            continue;
        }
        let dest = match backup_scheduler::auto_backup_path(&cfg, coin) {
            Ok(dest) => dest,
            Err(e) => {
                tracing::warn!("scheduled backup path failed for {}: {e}", coin.as_str());
                continue;
            }
        };
        match commands::backup_wallet_to_path(state.inner(), coin, &cfg, &dest).await {
            Ok(()) => {
                if let Err(e) = backup_scheduler::register_backup_hash(&dest) {
                    tracing::warn!("scheduled backup hash register failed for {}: {e}", coin.as_str());
                }
                let config = backup_scheduler::load_config()?;
                let _ = backup_scheduler::prune_old_backups(coin, &cfg, &config);
                paths.push(dest.display().to_string());
            }
            Err(e) => {
                tracing::warn!("scheduled backup failed for {}: {e}", coin.as_str());
            }
        }
    }

    if paths.is_empty() {
        return Ok(ScheduledBackupResult {
            ran: false,
            paths: Vec::new(),
        });
    }

    backup_scheduler::touch_last_run_at()?;
    Ok(ScheduledBackupResult { ran: true, paths })
}

#[tauri::command]
pub async fn backup_export_cloud(
    state: State<'_, AppState>,
    coin: String,
    password: String,
) -> AppResult<String> {
    let coin = parse_coin_id(&coin)?;
    let cfg = state.config(coin).await?;
    let path = backup_scheduler::export_encrypted_cloud(&cfg, coin, &password)?;
    audit_log::append("cloud_backup", &format!("Encrypted cloud backup to {path}"), Some(coin.as_str()))?;
    Ok(path)
}

#[tauri::command]
pub fn backup_verify(path: String) -> AppResult<bool> {
    backup_scheduler::verify_backup(std::path::Path::new(&path))
}

// ── SLIP-39 ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn slip39_split(mnemonic: String, threshold: u8, total: u8) -> AppResult<ShamirSplitResult> {
    slip39_recovery::split_mnemonic(&mnemonic, threshold, total)
}

#[tauri::command]
pub fn slip39_combine(shares: Vec<String>) -> AppResult<String> {
    slip39_recovery::combine_shares(&shares)
}

// ── Installer verification ───────────────────────────────────────────────────

#[tauri::command]
pub fn verify_installation() -> AppResult<VerificationStatus> {
    installer_verify::verify_installation()
}

// ── BIP21 URI parsing ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ParsedPaymentUri {
    pub scheme: String,
    pub address: String,
    pub amount: Option<f64>,
    pub label: Option<String>,
    pub message: Option<String>,
}

#[tauri::command]
pub fn parse_payment_uri(uri: String) -> AppResult<ParsedPaymentUri> {
    let trimmed = uri.trim();
    let (scheme, rest) = trimmed
        .split_once(':')
        .ok_or_else(|| AppError::other("invalid payment URI"))?;
    if scheme != "verium" && scheme != "vericoin" {
        return Err(AppError::other("unsupported scheme"));
    }
    let (address, query) = match rest.split_once('?') {
        Some((a, q)) => (a, Some(q)),
        None => (rest, None),
    };
    let mut amount = None;
    let mut label = None;
    let mut message = None;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "amount" => amount = v.parse().ok(),
                    "label" => label = Some(urlencoding::decode(v).unwrap_or_default().into_owned()),
                    "message" => message = Some(urlencoding::decode(v).unwrap_or_default().into_owned()),
                    _ => {}
                }
            }
        }
    }
    let coin = match scheme {
        "vericoin" => CoinId::Vericoin,
        _ => CoinId::Verium,
    };
    crate::wallet::address::validate_send_address(coin, address)?;
    Ok(ParsedPaymentUri {
        scheme: scheme.to_string(),
        address: address.trim().to_string(),
        amount,
        label,
        message,
    })
}

#[tauri::command]
pub fn build_payment_uri(
    coin: String,
    address: String,
    amount: Option<f64>,
    label: Option<String>,
    message: Option<String>,
) -> AppResult<String> {
    let scheme = match coin.as_str() {
        "vericoin" => "vericoin",
        _ => "verium",
    };
    let mut uri = format!("{scheme}:{address}");
    let mut params = Vec::new();
    if let Some(a) = amount {
        params.push(format!("amount={a}"));
    }
    if let Some(l) = label.filter(|s| !s.is_empty()) {
        params.push(format!("label={}", urlencoding::encode(&l)));
    }
    if let Some(m) = message.filter(|s| !s.is_empty()) {
        params.push(format!("message={}", urlencoding::encode(&m)));
    }
    if !params.is_empty() {
        uri.push('?');
        uri.push_str(&params.join("&"));
    }
    Ok(uri)
}
