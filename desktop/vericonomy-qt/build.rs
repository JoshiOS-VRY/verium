use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Register the Rust QObject bridges and all QML files under one QML module.
    // QML files are kept flat so same-module type resolution works; the entry
    // point resolves to: qrc:/qt/qml/com/vericonomy/verium/qml/main.qml
    CxxQtBuilder::new()
        .cc_builder(|cc| {
            cc.file("src/fonts_init.cpp");
        })
        .qml_module(QmlModule {
            uri: "com.vericonomy.verium",
            rust_files: &[
                "src/theme.rs",
                "src/host_links.rs",
                "src/setup_controller.rs",
                "src/light_wallet_controller.rs",
                "src/dashboard_controller.rs",
                "src/node_controller.rs",
                "src/wallet_controller.rs",
                "src/transactions_controller.rs",
                "src/mining_controller.rs",
                "src/staking_controller.rs",
                "src/network_controller.rs",
                "src/explorer_controller.rs",
                "src/settings_controller.rs",
                "src/wallet_mode_controller.rs",
                "src/logs_controller.rs",
                "src/rpc_controller.rs",
            ],
            qml_files: &[
                "qml/main.qml",
                // Design system
                "qml/AppButton.qml",
                "qml/Card.qml",
                "qml/Badge.qml",
                "qml/StatTile.qml",
                "qml/StatusPill.qml",
                "qml/Skeleton.qml",
                "qml/SectionHeader.qml",
                "qml/BalanceChart.qml",
                "qml/TransactionCategoryBadge.qml",
                "qml/ConfirmationProgress.qml",
                "qml/ExplorerTxLink.qml",
                "qml/WalletBalanceSummary.qml",
                "qml/TransferModeToggle.qml",
                "qml/WalletUnlockGate.qml",
                "qml/TransactionHistoryTable.qml",
                // Shell
                "qml/AppShell.qml",
                "qml/CoinSwitcher.qml",
                "qml/Sidebar.qml",
                "qml/NavItem.qml",
                "qml/NavIcon.qml",
                "qml/QuitWalletButton.qml",
                "qml/TopBar.qml",
                // Pages
                "qml/DashboardHero.qml",
                "qml/ExplorerRecentBlocks.qml",
                "qml/DashboardPage.qml",
                "qml/TransactionsPage.qml",
                "qml/SendReceivePage.qml",
                "qml/SettingsPage.qml",
                "qml/SecurityPage.qml",
                "qml/MiningPage.qml",
                "qml/StakingPage.qml",
                "qml/NetworkPage.qml",
                "qml/ExplorerPage.qml",
                "qml/PlaceholderPage.qml",
                "qml/RpcConsolePage.qml",
                "qml/LogsPage.qml",
                "qml/AddressBookPage.qml",
                "qml/SignVerifyPage.qml",
                "qml/ResourcesPage.qml",
                "qml/SetupWizardPage.qml",
            ],
            qrc_files: &[
                "assets/verium-logo.svg",
                "assets/vericoin-logo.svg",
                "assets/fonts/Inter-Regular.ttf",
                "assets/fonts/Inter-SemiBold.ttf",
                "assets/icons/nav/gauge.svg",
                "assets/icons/nav/cpu.svg",
                "assets/icons/nav/coins.svg",
                "assets/icons/nav/network.svg",
                "assets/icons/nav/link-2.svg",
                "assets/icons/nav/arrow-left-right.svg",
                "assets/icons/nav/book-user.svg",
                "assets/icons/nav/lock.svg",
                "assets/icons/nav/shield-check.svg",
                "assets/icons/nav/terminal.svg",
                "assets/icons/nav/scroll-text.svg",
                "assets/icons/nav/book-open.svg",
                "assets/icons/nav/settings.svg",
                "assets/icons/nav/power.svg",
            ],
            ..Default::default()
        })
        .build();
}
