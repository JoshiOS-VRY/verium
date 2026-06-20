import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var settings
    property var walletMode
    property var lightWallet
    property string coin: "verium"
    property var parseJson: function(s, fb) { return fb || {} }
    property string pendingMode: ""

    readonly property var prefs: page.settings
        ? page.parseJson(page.settings.prefsJson, {})
        : {}
    readonly property bool isLight: page.walletMode ? page.walletMode.isLight : false

    Component.onCompleted: {
        if (page.settings) page.settings.load()
        if (page.walletMode) page.walletMode.refresh()
    }

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        ColumnLayout {
            width: page.width
            spacing: 16

            Item { Layout.preferredHeight: 8; Layout.fillWidth: true }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 16
                    SectionHeader {
                        title: qsTr("Wallet mode")
                        subtitle: qsTr("Full node validates the chain locally. Light mode syncs via Electrum servers.")
                        Layout.fillWidth: true
                        Badge {
                            text: page.isLight ? qsTr("Light") : qsTr("Full node")
                            tone: page.isLight ? "accent" : "neutral"
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: modeFullCol.implicitHeight + 24
                            radius: Theme.radiusMd
                            color: !page.isLight ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.10)
                                                 : Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.35)
                            border.color: !page.isLight ? Theme.accent : Theme.border
                            ColumnLayout {
                                id: modeFullCol
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 4
                                Text {
                                    text: qsTr("Full node (recommended)")
                                    color: Theme.fg
                                    font.pixelSize: 13
                                    font.weight: Font.DemiBold
                                }
                                Text {
                                    text: qsTr("Uses wallet.dat and your local daemon. Best for mining and staking.")
                                    color: Theme.fgMuted
                                    font.pixelSize: 11
                                    wrapMode: Text.Wrap
                                    Layout.fillWidth: true
                                }
                                AppButton {
                                    text: qsTr("Use full node")
                                    variant: !page.isLight ? "primary" : "secondary"
                                    enabled: page.walletMode && page.isLight
                                    onClicked: page.pendingMode = "full_node"
                                }
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: modeLightCol.implicitHeight + 24
                            radius: Theme.radiusMd
                            color: page.isLight ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.10)
                                                : Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.35)
                            border.color: page.isLight ? Theme.accent : Theme.border
                            ColumnLayout {
                                id: modeLightCol
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 4
                                Text {
                                    text: qsTr("Light wallet")
                                    color: Theme.fg
                                    font.pixelSize: 13
                                    font.weight: Font.DemiBold
                                }
                                Text {
                                    text: qsTr("Convenience tier — balance via Electrum; keys still sign on this device.")
                                    color: Theme.fgMuted
                                    font.pixelSize: 11
                                    wrapMode: Text.Wrap
                                    Layout.fillWidth: true
                                }
                                AppButton {
                                    text: qsTr("Use light wallet")
                                    variant: page.isLight ? "primary" : "secondary"
                                    enabled: page.walletMode && !page.isLight
                                    onClicked: page.pendingMode = "light"
                                }
                            }
                        }
                    }

                    Rectangle {
                        visible: page.pendingMode.length > 0
                        Layout.fillWidth: true
                        radius: Theme.radiusMd
                        color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.08)
                        border.color: Theme.border
                        implicitHeight: confirmCol.implicitHeight + 20
                        ColumnLayout {
                            id: confirmCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 8
                            Text {
                                text: page.pendingMode === "light"
                                    ? qsTr("Switch to light wallet?")
                                    : qsTr("Switch to full node?")
                                color: Theme.fg
                                font.weight: Font.DemiBold
                            }
                            Text {
                                text: page.pendingMode === "light"
                                    ? qsTr("Export your recovery phrase from Security before switching if you rely on wallet.dat.")
                                    : qsTr("You will need wallet.dat and a running node. Import or unlock in Setup if needed.")
                                color: Theme.fgMuted
                                font.pixelSize: 11
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }
                            RowLayout {
                                spacing: 8
                                AppButton {
                                    text: qsTr("Confirm")
                                    onClicked: {
                                        if (page.walletMode)
                                            page.walletMode.setWalletMode(page.pendingMode)
                                        page.pendingMode = ""
                                    }
                                }
                                AppButton {
                                    text: qsTr("Cancel")
                                    variant: "ghost"
                                    onClicked: page.pendingMode = ""
                                }
                            }
                        }
                    }

                    Text {
                        visible: page.walletMode && page.walletMode.lastMessage.length > 0
                        text: page.walletMode ? page.walletMode.lastMessage : ""
                        color: Theme.fgSubtle
                        font.pixelSize: 11
                        Layout.fillWidth: true
                    }
                }
            }

            Card {
                visible: page.isLight
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader {
                        title: qsTr("Light wallet")
                        subtitle: page.walletMode && page.walletMode.lightWalletExists
                            ? qsTr("Keystore found — unlock or re-import")
                            : qsTr("Import recovery phrase or xprv")
                        Layout.fillWidth: true
                    }
                    TextField {
                        id: seedField
                        visible: !(page.walletMode && page.walletMode.lightWalletExists)
                        Layout.fillWidth: true
                        placeholderText: qsTr("24-word phrase or xprv")
                        color: Theme.fg
                        font.family: Theme.monoFamily
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                        implicitHeight: 72
                    }
                    TextField {
                        id: lightPass
                        Layout.fillWidth: true
                        placeholderText: qsTr("Passphrase")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    AppButton {
                        text: page.walletMode && page.walletMode.lightWalletExists
                            ? qsTr("Unlock light wallet") : qsTr("Import light wallet")
                        enabled: page.lightWallet && lightPass.text.length > 0
                            && (page.walletMode && page.walletMode.lightWalletExists
                                || seedField.text.length > 0)
                        onClicked: {
                            if (!page.lightWallet) return
                            if (page.walletMode && page.walletMode.lightWalletExists)
                                page.lightWallet.unlock(lightPass.text)
                            else
                                page.lightWallet.importWallet(seedField.text, lightPass.text)
                        }
                    }
                    Text {
                        visible: page.lightWallet && page.lightWallet.lastMessage.length > 0
                        text: page.lightWallet ? page.lightWallet.lastMessage : ""
                        color: Theme.fgMuted
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 16
                    SectionHeader { title: qsTr("Appearance"); Layout.fillWidth: true }

                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: qsTr("Dark theme")
                            color: Theme.fg
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            Layout.fillWidth: true
                        }
                        Switch {
                            checked: Theme.dark
                            onToggled: {
                                Theme.setDarkMode(checked)
                                page.savePref("dark_theme", checked)
                            }
                        }
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader {
                        title: qsTr("Mining & staking")
                        Layout.fillWidth: true
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: qsTr("Auto-mine on open")
                            color: Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            Layout.fillWidth: true
                        }
                        Switch {
                            checked: page.prefs.auto_mine_on_open === true
                            onToggled: page.savePref("auto_mine_on_open", checked)
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: qsTr("Auto-stake on open")
                            color: Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            Layout.fillWidth: true
                        }
                        Switch {
                            checked: page.prefs.auto_stake_on_open === true
                            onToggled: page.savePref("auto_stake_on_open", checked)
                        }
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Text {
                        text: qsTr("Vericonomy Wallet — Qt / QML")
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }
                    Text {
                        text: page.settings && page.settings.lastMessage.length
                            ? page.settings.lastMessage
                            : qsTr("Native desktop shell over the shared Rust core.")
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                    }
                }
            }

            Item { Layout.preferredHeight: 16; Layout.fillWidth: true }
        }
    }

    Connections {
        target: page.walletMode
        function onWalletModeChanged(ok) {
            if (ok && page.lightWallet)
                page.lightWallet.refreshExists()
        }
    }

    function savePref(key, value) {
        if (!page.settings) return
        var p = Object.assign({}, page.prefs)
        p[key] = value
        page.settings.savePrefs(JSON.stringify(p))
    }
}
