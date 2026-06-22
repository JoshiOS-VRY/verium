import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var setup
    property var wallet
    property string coin: "verium"
    property int step: 0
    property int walletAction: 0 // 0=hub, 1=create, 2=import dat, 3=unlock, 4=phrase
    signal finished()

    readonly property var steps: [
        { title: qsTr("Welcome"), body: qsTr("Set up your Vericonomy full-node wallet. Keys stay on this device.") },
        { title: qsTr("Wallet"), body: qsTr("Create, import wallet.dat, or unlock your full-node wallet.") },
        { title: qsTr("Finish"), body: qsTr("Open the dashboard.") }
    ]

    FileDialog {
        id: walletFileDialog
        title: qsTr("Select wallet.dat backup")
        nameFilters: ["Wallet backup (*.dat)", "All files (*)"]
        onAccepted: {
            if (!page.wallet) return
            var path = selectedFile.toString()
            if (path.startsWith("file://"))
                path = decodeURIComponent(path.slice(7))
            page.wallet.restoreWallet(path)
        }
    }

    Component.onCompleted: if (page.setup) page.setup.refresh()

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(600, page.width - 48)
        spacing: 20

        RowLayout {
            Layout.alignment: Qt.AlignHCenter
            spacing: 8
            Repeater {
                model: page.steps.length
                delegate: Rectangle {
                    required property int index
                    width: index === page.step ? 22 : 8
                    height: 8
                    radius: 4
                    color: index <= page.step ? Theme.accent : Theme.border
                }
            }
        }

        Card {
            Layout.fillWidth: true
            padding: 28
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14

                Text {
                    text: page.steps[page.step].title
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 22
                    font.weight: Font.Bold
                }
                Text {
                    text: page.steps[page.step].body
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 14
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                // Full-node wallet hub
                ColumnLayout {
                    visible: page.step === 1 && page.walletAction === 0
                    Layout.fillWidth: true
                    spacing: 10
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("Create new encrypted wallet")
                        onClicked: page.walletAction = 1
                    }
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("Import wallet.dat backup")
                        variant: "secondary"
                        onClicked: page.walletAction = 2
                    }
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("Restore 24-word phrase")
                        variant: "secondary"
                        onClicked: page.walletAction = 4
                    }
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("Unlock existing wallet")
                        variant: "secondary"
                        enabled: page.setup && (page.setup.hasFullNodeWallet || page.setup.walletReady)
                        onClicked: page.walletAction = 3
                    }
                    Text {
                        text: qsTr("Light wallet mode is available in Settings after setup. Full node is recommended.")
                        color: Theme.fgSubtle
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }

                // Create encrypted wallet
                ColumnLayout {
                    visible: page.step === 1 && page.walletAction === 1
                    Layout.fillWidth: true
                    spacing: 10
                    Text {
                        text: qsTr("Requires a running local node (veriumd / vericoind). The daemon may restart after encryption.")
                        color: Theme.fgSubtle
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    TextField {
                        id: createPass
                        Layout.fillWidth: true
                        placeholderText: qsTr("New wallet passphrase")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    TextField {
                        id: createPassConfirm
                        Layout.fillWidth: true
                        placeholderText: qsTr("Confirm passphrase")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton {
                            text: qsTr("Back")
                            variant: "ghost"
                            onClicked: page.walletAction = 0
                        }
                        Item { Layout.fillWidth: true }
                        AppButton {
                            text: qsTr("Create wallet")
                            enabled: page.wallet && createPass.text.length > 0
                                && createPass.text === createPassConfirm.text
                            onClicked: if (page.wallet)
                                page.wallet.createEncrypted(createPass.text)
                        }
                    }
                }

                // Import wallet.dat
                ColumnLayout {
                    visible: page.step === 1 && page.walletAction === 2
                    Layout.fillWidth: true
                    spacing: 10
                    Text {
                        text: qsTr("Choose a wallet.dat backup from Tauri or another Vericonomy install. The file is copied into your node datadir.")
                        color: Theme.fgSubtle
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton {
                            text: qsTr("Back")
                            variant: "ghost"
                            onClicked: page.walletAction = 0
                        }
                        Item { Layout.fillWidth: true }
                        AppButton {
                            text: qsTr("Choose wallet.dat…")
                            onClicked: walletFileDialog.open()
                        }
                    }
                }

                // Restore HD recovery phrase (sethdseed)
                ColumnLayout {
                    visible: page.step === 1 && page.walletAction === 4
                    Layout.fillWidth: true
                    spacing: 10
                    Text {
                        text: qsTr("Applies a BIP39 phrase to your full-node wallet via sethdseed. Requires a running daemon and wallet passphrase if locked.")
                        color: Theme.fgSubtle
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    TextArea {
                        id: recoveryPhrase
                        Layout.fillWidth: true
                        Layout.preferredHeight: 80
                        placeholderText: qsTr("24-word recovery phrase")
                        wrapMode: TextArea.Wrap
                        color: Theme.fg
                    }
                    TextField {
                        id: recoveryWalletPass
                        Layout.fillWidth: true
                        placeholderText: qsTr("Wallet passphrase (if locked)")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    TextField {
                        id: recoveryBip39Pass
                        Layout.fillWidth: true
                        placeholderText: qsTr("BIP39 passphrase (optional)")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton {
                            text: qsTr("Back")
                            variant: "ghost"
                            onClicked: page.walletAction = 0
                        }
                        Item { Layout.fillWidth: true }
                        AppButton {
                            text: qsTr("Apply phrase")
                            enabled: page.wallet && recoveryPhrase.text.trim().length > 0
                            onClicked: if (page.wallet)
                                page.wallet.applyRecoveryPhrase(
                                    recoveryPhrase.text,
                                    recoveryWalletPass.text,
                                    recoveryBip39Pass.text
                                )
                        }
                    }
                }

                // Unlock full-node wallet
                ColumnLayout {
                    visible: page.step === 1 && page.walletAction === 3
                    Layout.fillWidth: true
                    spacing: 10
                    TextField {
                        id: unlockPass
                        Layout.fillWidth: true
                        placeholderText: qsTr("Wallet passphrase")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton {
                            text: qsTr("Back")
                            variant: "ghost"
                            onClicked: page.walletAction = 0
                        }
                        Item { Layout.fillWidth: true }
                        AppButton {
                            text: qsTr("Unlock")
                            enabled: page.wallet && unlockPass.text.length > 0
                            onClicked: if (page.wallet)
                                page.wallet.unlock(unlockPass.text)
                        }
                    }
                }

                Text {
                    visible: page.wallet && page.wallet.lastMessage.length > 0
                    text: page.wallet ? page.wallet.lastMessage : ""
                    color: Theme.fgMuted
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                RowLayout {
                    visible: page.setup
                    Layout.fillWidth: true
                    spacing: 16
                    StatusPill {
                        tone: page.setup.nodeReachable ? "success" : "accent"
                        text: page.setup.nodeReachable ? qsTr("Node connected") : qsTr("Node offline")
                    }
                    StatusPill {
                        tone: page.setup.hasFullNodeWallet ? "success" : "accent"
                        text: page.setup.hasFullNodeWallet
                            ? qsTr("wallet.dat on disk") : qsTr("No wallet.dat")
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            AppButton {
                text: qsTr("Back")
                variant: "ghost"
                enabled: page.step > 0
                onClicked: page.step = Math.max(0, page.step - 1)
            }
            Item { Layout.fillWidth: true }
            AppButton {
                text: page.step === page.steps.length - 1 ? qsTr("Open wallet") : qsTr("Continue")
                onClicked: {
                    if (page.step < page.steps.length - 1) {
                        page.step++
                    } else if (page.setup) {
                        page.setup.completeSetup()
                    }
                }
            }
        }
    }

    Connections {
        target: page.setup
        function onSetupFinished(ok) { if (ok) page.finished() }
    }
    Connections {
        target: page.wallet
        function onCreateCompleted(ok) { if (ok) page.step = 2 }
        function onRestoreCompleted(ok) { if (ok) page.step = 2 }
        function onUnlockCompleted(ok) { if (ok) page.step = 2 }
        function onRecoveryCompleted(ok) { if (ok) page.step = 2 }
    }
}
