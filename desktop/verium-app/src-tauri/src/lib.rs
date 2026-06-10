#![allow(dead_code)]

mod block_height_cache;
mod chain;
mod node;
mod addressbook;
mod audit_log;
mod auto_lock;
mod backup_scheduler;
mod bootstrap;
mod chain_tip_watcher;
mod coin_profile;
mod commands;
mod config;
mod cpuminer_topo;
mod dace_commands;
mod daemon;
mod error;
mod explorer_api;
mod local_block_feed;
mod pool_api;
mod pool_miner;
mod features;
mod http_shared;
mod memory_telemetry;
mod hardware_wallet;
mod installer_verify;
mod logs;
mod mining_opt;
mod mining_supervisor;
mod multisig;
mod network_mode_commands;
mod onboarding;
mod onboarding_commands;
mod prefs;
mod receive_requests;
mod hd_wallet_export;
mod mnemonic_backup;
mod recovery;
mod rpc;
mod rpc_guard;
mod secret_store;
mod security_commands;
mod security_policy;
mod slip39_recovery;
mod spending_controls;
mod state;
mod two_factor;
mod updates;
mod wallet;
mod wallet_commands;
mod wallet_secrets;

use state::AppState;
use tauri::{Manager, WindowEvent};
use tracing_subscriber::EnvFilter;

fn load_dotenv_files() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in [".env", "../.env", "../.env.local"] {
        let path = manifest_dir.join(rel);
        if path.exists() {
            let _ = dotenvy::from_path(&path);
        }
    }
}

fn install_rustls_crypto_provider() {
    // reqwest enables aws-lc-rs; electrum TLS uses ring — pick one before any TLS handshake.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_dotenv_files();
    install_rustls_crypto_provider();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,verium_app_lib=debug")),
        )
        .try_init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    commands::graceful_shutdown_and_exit(app).await;
                });
            }
        })
        .setup(|app| {
            let state = AppState::initialize(app.handle().clone())?;
            let startup_state = state.clone();
            let startup_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                node::orchestrator::startup(startup_app, &startup_state).await;
            });
            tauri::async_runtime::spawn(async {
                let _ = tauri::async_runtime::spawn_blocking(|| {
                    crate::wallet::keystore::sync_manifest_from_keystore();
                    crate::onboarding_commands::log_wallet_storage_diagnostics();
                })
                .await;
            });
            // Reap any pool-miner sidecar orphaned by a previous (crashed) session
            // so it does not hold the API port or double-mine.
            tauri::async_runtime::spawn(async move {
                mining_supervisor::kill_stray_miners().await;
            });
            // Prewarm cpuminer `--tune` cache so the Mining page / pool start do
            // not block IPC. Deferred ~25s so it does not add a CPU/thread spike
            // during launch (it spawns cpuminer and waits ~2.5s); the tune cache
            // TTL is 300s, so this still warms before the user reaches mining.
            tauri::async_runtime::spawn(async {
                tokio::time::sleep(std::time::Duration::from_secs(25)).await;
                let _ = tauri::async_runtime::spawn_blocking(|| {
                    if let Some(binary) = mining_supervisor::resolve_cpuminer_binary() {
                        let _ =
                            cpuminer_topo::cpuminer_recommended_threads(Some(binary.as_path()));
                    }
                })
                .await;
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_node_status,
            commands::get_blockchain_info,
            commands::get_network_info,
            commands::get_peer_info,
            commands::get_added_node_info,
            commands::add_node,
            commands::get_mining_info,
            commands::get_wallet_info,
            commands::get_new_address,
            commands::list_transactions,
            commands::list_address_groupings,
            commands::miner_start,
            commands::miner_stop,
            commands::get_miner_state,
            commands::staking_start,
            commands::staking_stop,
            commands::get_staking_state,
            commands::reserve_balance_set,
            commands::get_coin_profiles,
            commands::detect_daemon,
            commands::wallet_unlock,
            commands::wallet_lock,
            commands::try_auto_unlock_wallet,
            commands::wallet_create_encrypted,
            commands::wallet_change_passphrase,
            commands::wallet_backup,
            commands::open_wallet_backup_folder,
            commands::wallet_restore,
            commands::wallet_dump_privkey,
            commands::wallet_import_privkey,
            commands::wallet_sign_message,
            commands::wallet_verify_message,
            commands::wallet_set_tx_fee,
            commands::wallet_list_unspent,
            commands::wallet_send_with_inputs,
            commands::rpc_raw_call,
            commands::send_to_address,
            commands::get_daemon_config,
            commands::set_daemon_config,
            commands::test_rpc_connection,
            commands::get_rpc_auth_diagnostics,
            commands::setup_rpc_credentials,
            commands::start_daemon,
            commands::stop_daemon,
            commands::restart_daemon,
            commands::tail_logs,
            commands::debug_log_status,
            commands::check_for_updates,
            commands::open_external_url,
            commands::play_block_chime,
            commands::read_verium_conf,
            commands::write_verium_conf,
            commands::open_verium_conf,
            commands::detect_veriumd,
            commands::detect_veriumd_runtime,
            commands::wallet_file_status,
            commands::ensure_first_run,
            commands::restart_after_encrypt,
            commands::get_user_preferences,
            commands::set_user_preferences,
            commands::import_bootstrap,
            commands::cancel_bootstrap,
            commands::quit_wallet,
            commands::fetch_explorer_stats,
            commands::fetch_explorer_blocks,
            commands::fetch_local_blocks_for_feed,
            commands::fetch_explorer_blocks_for_feed_cmd,
            commands::fetch_explorer_transactions,
            commands::fetch_explorer_extraction,
            commands::fetch_explorer_chain_tips,
            commands::fetch_explorer_peers_cmd,
            commands::get_explorer_logo_url,
            commands::is_explorer_api_enabled,
            commands::is_pool_api_enabled_cmd,
            commands::fetch_pool_stats_cmd,
            commands::fetch_miner_overview_cmd,
            commands::fetch_miner_hashrate_history_cmd,
            commands::fetch_pool_payout_summary_cmd,
            commands::fetch_miner_payouts_cmd,
            pool_miner::pool_miner_detect,
            pool_miner::pool_miner_memory_limits,
            pool_miner::pool_miner_status,
            pool_miner::pool_miner_log_lines,
            pool_miner::pool_miner_start,
            pool_miner::pool_miner_stop,
            commands::ensure_daemon_connected,
            commands::repair_chain,
            commands::node_retry,
            commands::node_clear_invalid_block,
            commands::node_reset_credentials,
            commands::address_book_list,
            commands::address_book_upsert,
            commands::address_book_delete,
            commands::diagnostic_bundle,
            mining_opt::cpu_topology,
            mining_opt::cpu_utilization_snapshot,
            mining_opt::bench_scrypt,
            mining_opt::battery_on_ac_power,
            dace_commands::binarychain_status,
            dace_commands::binarychain_metrics,
            dace_commands::binarychain_anchor,
            dace_commands::binarychain_redeem_claim,
            dace_commands::binarychain_register_ticket,
            dace_commands::binarychain_unbond_ticket,
            dace_commands::binarychain_fund_wallet,
            network_mode_commands::network_mode_get,
            network_mode_commands::network_mode_preview,
            network_mode_commands::network_mode_set,
            security_commands::recovery_generate_mnemonic,
            security_commands::recovery_validate_mnemonic,
            security_commands::recovery_verification_indices,
            security_commands::recovery_verify_words,
            security_commands::recovery_apply_hd_seed,
            security_commands::recovery_wallet_is_hd,
            security_commands::recovery_mnemonic_backup_exists,
            security_commands::recovery_export_seed,
            security_commands::two_factor_status,
            security_commands::two_factor_start_enrollment,
            security_commands::two_factor_confirm_enrollment,
            security_commands::two_factor_pending_otpauth_uri,
            security_commands::two_factor_verify,
            security_commands::two_factor_disable,
            security_commands::two_factor_is_gated,
            security_commands::two_factor_save_config,
            security_commands::auto_lock_get_config,
            security_commands::auto_lock_set_config,
            security_commands::auto_lock_record_activity,
            security_commands::auto_lock_should_lock,
            security_commands::audit_log_list,
            security_commands::audit_log_export,
            security_commands::audit_log_record,
            security_commands::receive_requests_list,
            security_commands::receive_requests_append,
            security_commands::receive_requests_delete,
            security_commands::hardware_wallet_list,
            security_commands::hardware_wallet_add,
            security_commands::hardware_wallet_remove,
            security_commands::hardware_wallet_detect,
            security_commands::hardware_wallet_import_xpub,
            security_commands::hardware_wallet_send_psbt,
            security_commands::hardware_wallet_finalize_psbt,
            security_commands::multisig_list,
            security_commands::multisig_save,
            security_commands::multisig_remove,
            security_commands::multisig_create_address,
            security_commands::spending_controls_get,
            security_commands::spending_controls_save,
            security_commands::spending_controls_check_send,
            security_commands::spending_controls_record_send,
            security_commands::spending_controls_check_allowlist,
            security_commands::backup_scheduler_get_config,
            security_commands::backup_scheduler_save_config,
            security_commands::backup_scheduler_set_interval,
            security_commands::backup_health,
            security_commands::backup_run_now,
            security_commands::backup_run_scheduled,
            security_commands::backup_export_cloud,
            security_commands::backup_verify,
            security_commands::slip39_split,
            security_commands::slip39_combine,
            security_commands::verify_installation,
            security_commands::parse_payment_uri,
            security_commands::build_payment_uri,
            memory_telemetry::get_memory_diagnostics,
            memory_telemetry::set_node_state_listener_count,
            wallet_commands::wallet_mode_get,
            wallet_commands::wallet_mode_set,
            wallet_commands::electrum_servers_get,
            wallet_commands::electrum_servers_set,
            wallet_commands::electrum_test_connection,
            wallet_commands::electrum_validate_defaults,
            wallet_commands::light_wallet_create,
            wallet_commands::light_wallet_import,
            wallet_commands::light_wallet_unlock,
            wallet_commands::light_wallet_rescan,
            wallet_commands::light_wallet_lock,
            wallet_commands::light_wallet_exists,
            wallet_commands::light_server_status,
            wallet_commands::electrum_cross_verify_tip,
            wallet_commands::wallet_mode_get_for_coin,
            wallet_commands::wallet_mode_set_for_coin,
            onboarding_commands::wallet_profile,
            onboarding_commands::wallet_storage_diagnostics,
            onboarding_commands::secret_store_status,
            onboarding_commands::secret_store_quarantine_orphaned,
            onboarding_commands::onboarding_checkpoint_get,
            onboarding_commands::onboarding_checkpoint_set,
            onboarding_commands::onboarding_mark_complete,
            onboarding_commands::legacy_datadir_candidate,
            onboarding_commands::legacy_request_hd_upgrade,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app
        .run(|app_handle, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                commands::run_shutdown_on_exit(app_handle);
            }
        });
}
