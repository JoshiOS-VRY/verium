import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Passphrase gate before send/receive/history (Tauri WalletUnlockGate parity).
Item {
    id: gate
    property var wallet
    property string coin: "verium"
    property string title: qsTr("Unlock to send and view transactions")
    property string description: ""
    default property alias content: contentSlot.data

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property string descText: description.length > 0
        ? description
        : qsTr("Enter your wallet passphrase to send or receive %1 and view your transaction history.").arg(ticker)

    readonly property bool showContent: wallet && !wallet.loading && !wallet.missing && !wallet.locked

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        visible: !gate.showContent

        Card {
            Layout.fillWidth: true
            padding: 24
            ColumnLayout {
                spacing: 12
                Layout.fillWidth: true

                Text {
                    visible: wallet && wallet.loading
                    text: qsTr("Loading wallet…")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 13
                    Layout.alignment: Qt.AlignHCenter
                }

                ColumnLayout {
                    visible: wallet && !wallet.loading && wallet.missing
                    spacing: 8
                    Layout.fillWidth: true
                    Text {
                        text: qsTr("Wallet unavailable")
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 16
                        font.weight: Font.DemiBold
                    }
                    Text {
                        text: qsTr("Connect to your node and ensure a wallet is loaded before using this page.")
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }

                ColumnLayout {
                    visible: wallet && !wallet.loading && !wallet.missing && wallet.locked
                    spacing: 12
                    Layout.fillWidth: true
                    Text {
                        text: gate.title
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 16
                        font.weight: Font.DemiBold
                    }
                    Text {
                        text: gate.descText
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.maximumWidth: 360
                        implicitHeight: 40
                        radius: Theme.radiusMd
                        color: Theme.bgSubtle
                        border.color: passField.activeFocus ? Theme.accent : Theme.border
                        border.width: 1
                        TextField {
                            id: passField
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            verticalAlignment: TextInput.AlignVCenter
                            placeholderText: qsTr("Passphrase")
                            echoMode: TextInput.Password
                            color: Theme.fg
                            placeholderTextColor: Theme.fgSubtle
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            background: null
                        }
                    }
                    AppButton {
                        text: qsTr("Unlock wallet")
                        enabled: passField.text.length > 0 && wallet
                        onClicked: if (wallet) wallet.unlock(passField.text)
                    }
                }
            }
        }
    }

    Item {
        id: contentSlot
        anchors.fill: parent
        visible: gate.showContent
    }
}
