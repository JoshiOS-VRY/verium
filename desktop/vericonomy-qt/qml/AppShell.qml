import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: shell

    property string route: "dashboard"
    property string coin: "verium"
    property bool showSetup: false

    readonly property var routeTitles: ({
        "dashboard": "Dashboard",
        "transactions": "Transactions",
        "send": "Send / Receive",
        "mining": "Mining",
        "staking": "Staking",
        "network": "Network",
        "explorer": "Explorer",
        "security": "Security",
        "settings": "Settings",
        "addressbook": "Address book",
        "signverify": "Sign & verify",
        "console": "RPC console",
        "logs": "Logs",
        "resources": "Resources",
        "binarychain": "Binary Chain"
    })
    readonly property var routeOrder: [
        "dashboard","mining","staking","network","binarychain",
        "transactions","addressbook","security","signverify",
        "console","logs","resources","settings",
        "send","explorer"
    ]

    function parseJson(str, fallback) {
        if (!str || str.length === 0)
            return fallback !== undefined ? fallback : []
        try { return JSON.parse(str) } catch (e) { return fallback !== undefined ? fallback : [] }
    }

    SetupController { id: setup; coin: shell.coin }
    WalletModeController { id: walletMode; coin: shell.coin }
    DashboardController  { id: dashboard; coin: shell.coin }
    LightWalletController { id: lightWallet; coin: shell.coin }
    NodeController       { id: node; coin: shell.coin }
    WalletController     { id: wallet; coin: shell.coin }
    TransactionsController { id: transactions; coin: shell.coin }
    MiningController     { id: mining; coin: shell.coin }
    StakingController    { id: staking; coin: shell.coin }
    NetworkController    { id: network; coin: shell.coin }
    ExplorerController   { id: explorer; coin: shell.coin }
    SettingsController   { id: settings }
    LogsController       { id: logs; coin: shell.coin }
    RpcController        { id: rpc; coin: shell.coin }

    Component.onCompleted: {
        setup.refresh()
        settings.load()
    }

    Connections {
        target: setup
        function onSetupRefreshed(ok) {
            if (ok) shell.showSetup = !setup.setupCompleted
            if (!shell.showSetup) shell.refreshAll()
        }
        function onSetupFinished(ok) {
            if (ok) shell.showSetup = false
            shell.refreshAll()
        }
    }

    function refreshAll() {
        node.refresh()
        wallet.refresh()
        transactions.refresh()
        mining.refresh()
        staking.refresh()
        network.refresh()
        dashboard.refresh()
        walletMode.refresh()
        explorer.refresh()
        logs.refresh()
    }

    onCoinChanged: {
        setup.coin = shell.coin
        lightWallet.coin = shell.coin
        explorer.coin = shell.coin
        walletMode.coin = shell.coin
        if (shell.route === "staking" && shell.coin === "verium")
            shell.route = "mining"
        else if (shell.route === "mining" && shell.coin === "vericoin")
            shell.route = "staking"
        lightWallet.refreshExists()
        refreshAll()
    }

    Connections {
        target: walletMode
        function onWalletModeChanged(ok) {
            if (!ok) return
            if (!walletMode.isLight) return
            var fullNodeOnly = ["mining","staking","network","signverify","console","logs","binarychain"]
            if (fullNodeOnly.indexOf(shell.route) >= 0)
                shell.route = "dashboard"
            shell.refreshAll()
        }
    }

    Timer {
        interval: 5000; running: !shell.showSetup; repeat: true
        onTriggered: shell.refreshAll()
    }

    SetupWizardPage {
        anchors.fill: parent
        visible: shell.showSetup
        setup: setup
        wallet: wallet
        coin: shell.coin
        onFinished: shell.showSetup = false
    }

    RowLayout {
        anchors.fill: parent
        visible: !shell.showSetup
        spacing: 0

        Sidebar {
            Layout.fillHeight: true
            current: shell.route
            coin: shell.coin
            wallet: wallet
            walletMode: walletMode
            node: node
            parseJson: shell.parseJson
            onNavigate: (r) => shell.route = r
            onSwitchCoin: (c) => shell.coin = c
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            TopBar {
                Layout.fillWidth: true
                title: shell.routeTitles[shell.route]
                node: node
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: Math.max(0, shell.routeOrder.indexOf(shell.route))

                DashboardPage {
                    dashboard: dashboard
                    explorer: explorer
                    coin: shell.coin
                    parseJson: shell.parseJson
                }
                MiningPage { coin: shell.coin; mining: mining }
                StakingPage { coin: shell.coin; staking: staking }
                NetworkPage { node: node; network: network; parseJson: shell.parseJson }
                PlaceholderPage {
                    title: qsTr("Binary Chain")
                    subtitle: qsTr("DACE binarytest network tools — coming soon in the Qt shell.")
                    glyph: "\u26D3"
                    features: [
                        qsTr("Binary chain funding"),
                        qsTr("Test network block explorer"),
                        qsTr("DACE wallet utilities")
                    ]
                }
                TransactionsPage {
                    coin: shell.coin
                    transactions: transactions
                    wallet: wallet
                    parseJson: shell.parseJson
                }
                AddressBookPage {}
                SecurityPage {}
                SignVerifyPage { coin: shell.coin; wallet: wallet }
                RpcConsolePage { rpc: rpc; parseJson: shell.parseJson }
                LogsPage { logs: logs; parseJson: shell.parseJson }
                ResourcesPage {}
                SettingsPage {
                    settings: settings
                    walletMode: walletMode
                    lightWallet: lightWallet
                    coin: shell.coin
                    parseJson: shell.parseJson
                }
                SendReceivePage { coin: shell.coin; wallet: wallet }
                ExplorerPage { coin: shell.coin; explorer: explorer; parseJson: shell.parseJson }
            }
        }
    }
}
