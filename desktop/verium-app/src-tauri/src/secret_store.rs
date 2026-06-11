//! Encrypted blob storage backed by OS keychain + Argon2id + AES-256-GCM.

use std::path::PathBuf;
use std::sync::Mutex;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::config::app_config_base;
use crate::error::{AppError, AppResult};

const KEYCHAIN_ACCOUNT: &str = "secret-store-master-v1";
const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
#[cfg(mobile)]
const MASTER_KEY_FILE: &str = ".master-key";

fn keychain_service() -> &'static str {
    if cfg!(target_os = "ios") {
        "com.vericonomy.wallet.ios"
    } else if cfg!(target_os = "android") {
        "com.vericonomy.wallet.android"
    } else {
        "com.vericonomy.wallet.desktop"
    }
}

fn keychain_label() -> &'static str {
    if cfg!(windows) {
        "Windows Credential Manager"
    } else if cfg!(target_os = "macos") {
        "macOS Keychain"
    } else if cfg!(target_os = "ios") {
        "iOS Keychain"
    } else if cfg!(target_os = "android") {
        "Android Keystore"
    } else {
        "system keychain"
    }
}

fn orphaned_encrypted_data_message_text() -> String {
    #[cfg(mobile)]
    {
        return "Encrypted app data on this device could not be unlocked. \
                 Import your recovery phrase to restore a wallet, or set up a new wallet to start fresh."
            .to_string();
    }
    #[cfg(not(mobile))]
    format!(
        "{} entry missing for {}/{} but encrypted wallet data exists. \
         Restore the saved credential or recover from your recovery phrase — creating a new master key \
         would make existing light wallets unreadable.",
        keychain_label(),
        keychain_service(),
        KEYCHAIN_ACCOUNT
    )
}

/// True when the OS keychain has no master key but encrypted `.enc` blobs remain.
pub fn encrypted_data_orphaned() -> bool {
    #[cfg(not(test))]
    if MASTER_KEY
        .lock()
        .ok()
        .and_then(|guard| *guard)
        .is_some()
    {
        return false;
    }
    #[cfg(mobile)]
    if mobile_master_key_available() {
        return false;
    }
    match keyring_entry().get_password() {
        Err(keyring::Error::NoEntry) => secure_dir_has_encrypted_blobs(),
        _ => false,
    }
}

pub fn orphaned_encrypted_data_message() -> String {
    orphaned_encrypted_data_message_text()
}

fn store_dir() -> PathBuf {
    app_config_base().join("secure")
}

fn secure_dir_has_encrypted_blobs() -> bool {
    let dir = store_dir();
    if !dir.exists() {
        return false;
    }
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "enc" && e.path().file_stem().is_some())
        })
}

fn try_decrypt_backup_blob(label: &str) -> Option<Zeroizing<Vec<u8>>> {
    let bak = blob_backup_path(label);
    if !bak.exists() {
        return None;
    }
    let blob = std::fs::read(&bak).ok()?;
    decrypt(&blob).ok().map(Zeroizing::new)
}

fn blob_path(label: &str) -> PathBuf {
    store_dir().join(format!("{label}.enc"))
}

/// Whether an encrypted blob file exists for `label`.
pub fn blob_exists(label: &str) -> bool {
    blob_path(label).exists()
}

/// Whether the encrypted blob exists and decrypts with the current master key.
pub fn blob_readable(label: &str) -> bool {
    blob_decrypts(label)
}

/// Whether the `.enc.bak` copy exists and decrypts with the current master key.
pub fn blob_backup_readable(label: &str) -> bool {
    let path = blob_backup_path(label);
    if !path.exists() {
        return false;
    }
    match std::fs::read(path) {
        Ok(blob) => decrypt(&blob).is_ok(),
        Err(_) => false,
    }
}

fn keyring_entry() -> keyring::Entry {
    keyring::Entry::new(keychain_service(), KEYCHAIN_ACCOUNT).expect("keyring entry")
}

#[cfg(mobile)]
fn master_key_file_path() -> PathBuf {
    store_dir().join(MASTER_KEY_FILE)
}

#[cfg(mobile)]
fn load_master_key_from_file() -> Option<[u8; 32]> {
    let path = master_key_file_path();
    let hex_key = std::fs::read_to_string(path).ok()?;
    let bytes = hex::decode(hex_key.trim()).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

#[cfg(mobile)]
fn save_master_key_to_file(key: &[u8; 32]) -> AppResult<()> {
    std::fs::create_dir_all(store_dir())?;
    let path = master_key_file_path();
    let tmp = path.with_extension("new");
    std::fs::write(&tmp, hex::encode(key)).map_err(|e| {
        AppError::other(format!("could not write mobile master key file: {e}"))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).ok();
    }
    std::fs::rename(&tmp, &path).map_err(|e| {
        AppError::other(format!("could not finalize mobile master key file: {e}"))
    })?;
    Ok(())
}

#[cfg(mobile)]
fn mobile_master_key_available() -> bool {
    load_master_key_from_file().is_some()
}

/// Process-wide master key cache. Windows Credential Manager can be slow or
/// briefly inconsistent when multiple blobs are written in parallel; caching
/// also prevents a create-key race when two threads both see `NoEntry`.
#[cfg(not(test))]
static MASTER_KEY: Mutex<Option<[u8; 32]>> = Mutex::new(None);

#[cfg(not(test))]
static MASTER_KEY_INIT: Mutex<()> = Mutex::new(());

#[cfg(test)]
fn ensure_master_key() -> AppResult<[u8; 32]> {
    Ok([0xA5; 32])
}

#[cfg(not(test))]
fn load_master_key_from_keyring() -> AppResult<[u8; 32]> {
    match keyring_entry().get_password() {
        Ok(hex_key) => {
            let bytes = hex::decode(hex_key.trim()).map_err(|e| {
                AppError::other(format!("invalid master key in keychain: {e}"))
            })?;
            if bytes.len() != 32 {
                return Err(AppError::other("master key has wrong length"));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(key)
        }
        Err(keyring::Error::NoEntry) => {
            #[cfg(mobile)]
            if let Some(key) = load_master_key_from_file() {
                return Ok(key);
            }
            if secure_dir_has_encrypted_blobs() {
                return Err(AppError::other(orphaned_encrypted_data_message_text()));
            }
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            if let Err(e) = keyring_entry().set_password(&hex_key) {
                #[cfg(mobile)]
                {
                    tracing::warn!(
                        "keychain write failed ({e}); persisting master key to app secure storage"
                    );
                    save_master_key_to_file(&key)?;
                    return Ok(key);
                }
                #[cfg(not(mobile))]
                return Err(AppError::other(format!("could not store master key: {e}")));
            }
            #[cfg(mobile)]
            {
                let _ = save_master_key_to_file(&key);
            }
            Ok(key)
        }
        Err(e) => Err(AppError::other(format!("keychain read failed: {e}"))),
    }
}

#[cfg(not(test))]
fn ensure_master_key() -> AppResult<[u8; 32]> {
    if let Ok(guard) = MASTER_KEY.lock() {
        if let Some(key) = *guard {
            return Ok(key);
        }
    }

    let init_guard = MASTER_KEY_INIT
        .lock()
        .map_err(|e| AppError::other(format!("master key init lock failed: {e}")))?;
    if let Ok(guard) = MASTER_KEY.lock() {
        if let Some(key) = *guard {
            return Ok(key);
        }
    }

    let key = load_master_key_from_keyring()?;
    if let Ok(mut guard) = MASTER_KEY.lock() {
        *guard = Some(key);
    }
    drop(init_guard);
    Ok(key)
}

fn blob_decrypts(label: &str) -> bool {
    if encrypted_data_orphaned() {
        return false;
    }
    let path = blob_path(label);
    if !path.exists() {
        return false;
    }
    match std::fs::read(path) {
        Ok(blob) => decrypt(&blob).is_ok(),
        Err(_) => false,
    }
}

fn derive_key(master: &[u8; 32], salt: &[u8]) -> AppResult<[u8; 32]> {
    let params = Params::new(19 * 1024, 2, 1, Some(32))
        .map_err(|e| AppError::other(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(master, salt, &mut out)
        .map_err(|e| AppError::other(format!("argon2 derive: {e}")))?;
    Ok(out)
}

fn encrypt(plaintext: &[u8]) -> AppResult<Vec<u8>> {
    let master = ensure_master_key()?;
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(&master, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::other(format!("cipher init: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::other(format!("encrypt failed: {e}")))?;

    let mut out = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt(blob: &[u8]) -> AppResult<Vec<u8>> {
    if blob.len() < SALT_LEN + NONCE_LEN + 1 {
        return Err(AppError::other("encrypted blob too short"));
    }
    let master = ensure_master_key()?;
    let salt = &blob[..SALT_LEN];
    let nonce_bytes = &blob[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &blob[SALT_LEN + NONCE_LEN..];

    let key = derive_key(&master, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::other(format!("cipher init: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::other(format!("decrypt failed: {e}")))
}

fn blob_backup_path(label: &str) -> PathBuf {
    blob_path(label).with_extension("enc.bak")
}

/// Decrypt a blob file without recovery side effects (no quarantine).
pub fn read_decrypted_blob(label: &str) -> AppResult<Zeroizing<Vec<u8>>> {
    let path = blob_path(label);
    if !path.exists() {
        return Err(AppError::other(format!("encrypted blob missing: {label}")));
    }
    let blob = std::fs::read(&path).map_err(|e| {
        AppError::other(format!("could not read encrypted blob {label}: {e}"))
    })?;
    decrypt(&blob).map_err(|e| {
        AppError::other(format!(
            "could not decrypt {label}: {e}. Check {} (service: {}).",
            keychain_label(),
            keychain_service()
        ))
    })
    .map(Zeroizing::new)
}

/// Load JSON from an encrypted blob; returns the real decrypt/parse error instead
/// of silently falling back to defaults.
pub fn load_json_strict<T: serde::de::DeserializeOwned>(label: &str) -> AppResult<T> {
    let bytes = read_decrypted_blob(label)?;
    let s = String::from_utf8(bytes.to_vec())
        .map_err(|e| AppError::other(format!("invalid utf8 in {label}: {e}")))?;
    serde_json::from_str(&s).map_err(|e| AppError::other(format!("invalid json in {label}: {e}")))
}

/// Seal bytes to an encrypted file. Overwrites any existing blob.
pub fn seal(label: &str, plaintext: &[u8]) -> AppResult<()> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir)?;

    let encrypted = encrypt(plaintext)?;
    let decrypted = decrypt(&encrypted).map_err(|e| {
        AppError::other(format!("encrypt roundtrip failed for {label}: {e}"))
    })?;
    if decrypted.as_slice() != plaintext {
        return Err(AppError::other(format!(
            "encrypt roundtrip mismatch for {label}"
        )));
    }

    let path = blob_path(label);
    let tmp = path.with_extension("enc.new");
    std::fs::write(&tmp, &encrypted).map_err(|e| {
        AppError::other(format!("could not write encrypted blob {label}: {e}"))
    })?;

    let read_back = std::fs::read(&tmp).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("could not read back encrypted blob {label}: {e}"))
    })?;
    let disk_plain = decrypt(&read_back).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("post-write decrypt failed for {label}: {e}"))
    })?;
    if disk_plain.as_slice() != plaintext {
        let _ = std::fs::remove_file(&tmp);
        return Err(AppError::other(format!(
            "post-write encrypt roundtrip mismatch for {label}"
        )));
    }

    if path.exists() {
        let bak = blob_backup_path(label);
        let _ = std::fs::copy(&path, &bak);
    }
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("could not finalize encrypted blob {label}: {e}"))
    })?;
    Ok(())
}

/// Open and decrypt a sealed blob. Returns None if the blob does not exist.
pub fn open(label: &str) -> AppResult<Option<Zeroizing<Vec<u8>>>> {
    open_with_recovery(label, None)
}

/// Like [`open`], but on decrypt failure quarantines the corrupt blob and
/// optionally falls back to a plaintext file (then re-seals with the current
/// master key). This handles Windows Credential Manager resets where the
/// encrypted blob survives but the master key does not.
fn open_with_recovery(
    label: &str,
    plaintext_fallback: Option<&std::path::Path>,
) -> AppResult<Option<Zeroizing<Vec<u8>>>> {
    let path = blob_path(label);
    if !path.exists() {
        return read_plaintext_fallback(plaintext_fallback);
    }
    let blob = std::fs::read(&path)?;
    match decrypt(&blob) {
        Ok(plain) => Ok(Some(Zeroizing::new(plain))),
        Err(e) => {
            tracing::warn!(
                "secret_store: decrypt failed for {label} ({e}); attempting recovery"
            );
            if let Some(plain) = try_decrypt_backup_blob(label) {
                tracing::warn!("secret_store: recovered {label} from encrypted backup");
                let _ = quarantine_corrupt_blob(label);
                if let Err(seal_err) = seal(label, plain.as_ref()) {
                    tracing::warn!(
                        "secret_store: could not re-seal recovered {label}: {seal_err}"
                    );
                }
                return Ok(Some(plain));
            }
            if let Some(fallback) = plaintext_fallback {
                if migrate_plaintext_json(label, fallback)? {
                    return open_with_recovery(label, Some(fallback));
                }
                if let Some(plain) = read_plaintext_fallback(Some(fallback))? {
                    quarantine_corrupt_blob(label)?;
                    if encrypted_data_orphaned() {
                        tracing::warn!(
                            "secret_store: using plaintext fallback for {label} (credential manager orphaned)"
                        );
                    } else if let Err(seal_err) = seal(label, plain.as_ref()) {
                        tracing::warn!(
                            "secret_store: could not re-seal plaintext fallback for {label}: {seal_err}"
                        );
                    }
                    return Ok(Some(plain));
                }
            }
            // Do not quarantine here — callers may retry, and seal() verifies writes.
            Ok(None)
        }
    }
}

fn read_plaintext_fallback(
    path: Option<&std::path::Path>,
) -> AppResult<Option<Zeroizing<Vec<u8>>>> {
    let Some(path) = path else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read(path)?;
    Ok(Some(Zeroizing::new(raw)))
}

fn quarantine_corrupt_blob(label: &str) -> AppResult<()> {
    let path = blob_path(label);
    if !path.exists() {
        return Ok(());
    }
    let bak = blob_backup_path(label);
    if bak.exists() {
        let _ = std::fs::remove_file(&bak);
    }
    std::fs::rename(&path, &bak)?;
    tracing::warn!(
        "secret_store: quarantined corrupt blob for {label} at {}",
        bak.display()
    );
    Ok(())
}

/// Delete a sealed blob.
pub fn delete(label: &str) -> AppResult<()> {
    let path = blob_path(label);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Migrate plaintext JSON to encrypted storage, then remove the plaintext file.
pub fn migrate_plaintext_json(label: &str, plaintext_path: &std::path::Path) -> AppResult<bool> {
    if !plaintext_path.exists() {
        return Ok(false);
    }
    if blob_path(label).exists() {
        // Only discard plaintext once the encrypted blob is confirmed readable.
        if blob_decrypts(label) {
            let _ = std::fs::remove_file(plaintext_path);
            return Ok(true);
        }
        return Ok(false);
    }
    if encrypted_data_orphaned() {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(plaintext_path)?;
    seal(label, raw.as_bytes())?;
    std::fs::remove_file(plaintext_path)?;
    tracing::info!("secret_store: migrated {} to encrypted storage", label);
    Ok(true)
}

fn load_json_from_plaintext<T: serde::de::DeserializeOwned>(
    label: &str,
    plaintext_path: &std::path::Path,
    default: T,
) -> AppResult<T> {
    let Some(bytes) = read_plaintext_fallback(Some(plaintext_path))? else {
        return Ok(default);
    };
    let s = String::from_utf8(bytes.to_vec())
        .map_err(|e| AppError::other(format!("invalid utf8 in {label} plaintext: {e}")))?;
    match serde_json::from_str::<T>(&s) {
        Ok(v) => {
            if let Err(e) = seal(label, bytes.as_ref()) {
                tracing::warn!("secret_store: re-seal {label} from plaintext failed: {e}");
            }
            Ok(v)
        }
        Err(e) => {
            tracing::warn!("secret_store: invalid json in {label} plaintext, using defaults: {e}");
            Ok(default)
        }
    }
}

/// Load JSON from encrypted store, falling back to plaintext migration.
pub fn load_json<T: serde::de::DeserializeOwned>(
    label: &str,
    plaintext_path: &std::path::Path,
    default: T,
) -> AppResult<T> {
    if !blob_path(label).exists() {
        migrate_plaintext_json(label, plaintext_path)?;
    }
    match open_with_recovery(label, Some(plaintext_path))? {
        Some(bytes) => {
            let s = String::from_utf8(bytes.to_vec())
                .map_err(|e| AppError::other(format!("invalid utf8 in {label}: {e}")))?;
            match serde_json::from_str::<T>(&s) {
                Ok(v) => Ok(v),
                Err(e) => {
                    tracing::warn!("secret_store: invalid json in {label}, trying plaintext: {e}");
                    load_json_from_plaintext(label, plaintext_path, default)
                }
            }
        }
        None => load_json_from_plaintext(label, plaintext_path, default),
    }
}

/// Save JSON to encrypted store.
pub fn save_json<T: serde::Serialize + ?Sized>(label: &str, value: &T) -> AppResult<()> {
    if encrypted_data_orphaned() {
        return Err(AppError::other(orphaned_encrypted_data_message_text()));
    }
    #[cfg(mobile)]
    if let Ok(key) = ensure_master_key() {
        let _ = save_master_key_to_file(&key);
        let _ = keyring_entry().set_password(&hex::encode(key));
    }
    let json = serde_json::to_string_pretty(value)?;
    seal(label, json.as_bytes())
}

#[cfg(not(test))]
fn clear_master_key_cache() {
    if let Ok(mut guard) = MASTER_KEY.lock() {
        *guard = None;
    }
}

#[cfg(test)]
fn clear_master_key_cache() {}

/// Move orphaned encrypted blobs aside so a fresh credential-manager key can be created.
/// Existing light-wallet ciphertext becomes unreadable unless the user restores the old CM entry.
pub fn quarantine_orphaned_encrypted_data() -> AppResult<String> {
    if !encrypted_data_orphaned() {
        return Err(AppError::other(
            "Credential Manager entry is present; quarantine is not needed.",
        ));
    }
    let dir = store_dir();
    if !dir.exists() {
        return Err(AppError::other("no secure storage directory found"));
    }
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let dest = dir.join(format!("orphaned-{stamp}"));
    std::fs::create_dir_all(&dest)?;
    let mut moved = 0usize;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.ends_with(".enc")
            || name.ends_with(".enc.bak")
            || name.ends_with(".enc.new")
            || name.ends_with(".enc.corrupt")
        {
            let target = dest.join(name);
            std::fs::rename(&path, &target)?;
            moved += 1;
        }
    }
    if moved == 0 {
        let _ = std::fs::remove_dir(&dest);
        return Err(AppError::other("no encrypted blobs found to quarantine"));
    }
    clear_master_key_cache();
    let _ = ensure_master_key()?;
    tracing::warn!(
        "quarantined {moved} encrypted blob(s) to {}; a new Credential Manager master key was created",
        dest.display()
    );
    Ok(dest.display().to_string())
}

fn derive_key_from_passphrase(passphrase: &str, salt: &[u8]) -> AppResult<[u8; 32]> {
    let params = Params::new(19 * 1024, 2, 1, Some(32))
        .map_err(|e| AppError::other(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| AppError::other(format!("argon2 derive: {e}")))?;
    Ok(out)
}

/// Encrypt with a user passphrase (light-wallet mnemonic sealing).
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
) -> AppResult<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let key = derive_key_from_passphrase(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::other(format!("cipher init: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::other(format!("encrypt failed: {e}")))?;
    Ok((ciphertext, salt.to_vec(), nonce_bytes.to_vec()))
}

/// Decrypt bytes sealed with [`encrypt_with_passphrase`].
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    salt: &[u8],
    nonce_bytes: &[u8],
    passphrase: &str,
) -> AppResult<Vec<u8>> {
    let key = derive_key_from_passphrase(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::other(format!("cipher init: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::other(format!("decrypt failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let data = b"hello secure world";
        let enc = encrypt(data).unwrap();
        let dec = decrypt(&enc).unwrap();
        assert_eq!(dec, data);
    }
}
