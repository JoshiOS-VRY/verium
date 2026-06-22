import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Left navigation rail — Tauri Sidebar parity.
Rectangle {
    id: sidebar
    property string current: "dashboard"
    property string coin: "verium"
    property var wallet
    property var walletMode
    property var node
    property var parseJson

    signal navigate(string route)
    signal switchCoin(string coin)

    readonly property string appVersion: "1.0.0"
    readonly property bool binarytestEnabled: sidebar.appVersion.toLowerCase().indexOf("alpha") < 0
    readonly property bool isLight: walletMode ? walletMode.isLight : false
    readonly property bool isTestNetwork: {
        if (!node || !node.chain)
            return false
        var c = node.chain.toLowerCase()
        return c.indexOf("binarytest") >= 0 || c === "test"
    }
    readonly property bool walletLocked: wallet ? wallet.locked : false

    readonly property var allNavItems: [
        { route: "dashboard",    label: qsTr("Dashboard"),       icon: "gauge" },
        { route: "mining",       label: qsTr("Mining"),          icon: "cpu", coins: ["verium"], requiresPassphrase: true, fullNodeOnly: true },
        { route: "staking",      label: qsTr("Staking"),         icon: "coins", coins: ["vericoin"], requiresPassphrase: true, fullNodeOnly: true },
        { route: "network",      label: qsTr("Network"),         icon: "network", fullNodeOnly: true },
        { route: "binarychain",  label: qsTr("Binary Chain"),    icon: "link-2", testNetworkOnly: true },
        { route: "transactions", label: qsTr("Transactions"),    icon: "arrow-left-right", requiresPassphrase: true },
        { route: "addressbook",  label: qsTr("Address book"),    icon: "book-user" },
        { route: "security",     label: qsTr("Security"),        icon: "lock", requiresPassphrase: true },
        { route: "signverify",   label: qsTr("Sign & verify"),   icon: "shield-check", requiresPassphrase: true, fullNodeOnly: true },
        { route: "console",      label: qsTr("RPC console"),    icon: "terminal", requiresPassphrase: true, fullNodeOnly: true },
        { route: "logs",         label: qsTr("Logs"),            icon: "scroll-text", fullNodeOnly: true },
        { route: "resources",    label: qsTr("Resources"),       icon: "book-open" },
        { route: "settings",     label: qsTr("Settings"),        icon: "settings" }
    ]

    function itemVisible(item) {
        if (item.fullNodeOnly && sidebar.isLight)
            return false
        if (item.testNetworkOnly && (!sidebar.binarytestEnabled || !sidebar.isTestNetwork))
            return false
        if (item.coins)
            return item.coins.indexOf(sidebar.coin) >= 0
        return true
    }

    implicitWidth: 240
    color: Theme.bgSubtle
    border.width: 0

    Rectangle {
        anchors.right: parent.right
        width: 1
        height: parent.height
        color: Theme.border
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Item {
            id: coinHeader
            Layout.fillWidth: true
            Layout.preferredHeight: coinSwitcher.height + 8
            Layout.minimumHeight: 60
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.topMargin: 16
            Layout.bottomMargin: 8
            z: 1

            CoinSwitcher {
                id: coinSwitcher
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                height: 52
                coin: sidebar.coin
                onCoinSelected: (coinId) => sidebar.switchCoin(coinId)
            }
        }

        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 0
            contentHeight: navCol.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            ColumnLayout {
                id: navCol
                width: parent.width
                spacing: 2

                Repeater {
                    model: sidebar.allNavItems
                    delegate: NavItem {
                        required property var modelData
                        visible: sidebar.itemVisible(modelData)
                        label: modelData.label
                        icon: modelData.icon
                        selected: sidebar.current === modelData.route
                        showLock: modelData.requiresPassphrase === true && sidebar.walletLocked
                        onActivated: sidebar.navigate(modelData.route)
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.border
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 20
            Layout.rightMargin: 20
            Layout.topMargin: 12
            Layout.bottomMargin: 12
            spacing: 4

            QuitWalletButton {
                Layout.fillWidth: true
                Layout.leftMargin: -4
            }

            Text {
                text: qsTr("Vericonomy Wallet v") + sidebar.appVersion
                color: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 12
                Layout.fillWidth: true
            }

            RowLayout {
                visible: sidebar.binarytestEnabled && sidebar.isTestNetwork
                spacing: 8
                Rectangle {
                    radius: 4
                    color: Qt.rgba(245/255, 158/255, 11/255, 0.20)
                    border.color: Qt.rgba(245/255, 158/255, 11/255, 0.40)
                    border.width: 1
                    implicitHeight: badgeText.implicitHeight + 4
                    implicitWidth: badgeText.implicitWidth + 12
                    Text {
                        id: badgeText
                        anchors.centerIn: parent
                        text: qsTr("binarytest")
                        color: Qt.rgba(253/255, 230/255, 138/255, 1)
                        font.family: Theme.fontFamily
                        font.pixelSize: 9
                        font.weight: Font.DemiBold
                    }
                }
            }
        }
    }
}
