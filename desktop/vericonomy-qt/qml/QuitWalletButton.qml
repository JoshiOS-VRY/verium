import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.vericonomy.verium

// Quit wallet with confirm dialog (Tauri QuitWalletButton parity).
Item {
    id: quitBtn
    property bool quitting: false

    implicitWidth: row.implicitWidth
    implicitHeight: row.implicitHeight

    RowLayout {
        id: row
        spacing: 8
        NavIcon {
            name: "power"
            tint: area.containsMouse ? Theme.fg : Theme.fgMuted
        }
        Text {
            text: quitBtn.quitting ? qsTr("Quitting…") : qsTr("Quit wallet")
            color: area.containsMouse ? Theme.fg : Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
            font.weight: Font.Medium
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        enabled: !quitBtn.quitting
        onClicked: confirmDialog.open()
    }

    Dialog {
        id: confirmDialog
        title: qsTr("Quit Vericonomy Wallet?")
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.NoButton
        width: 448

        background: Rectangle {
            radius: Theme.radiusLg
            color: Theme.bgPanel
            border.color: Theme.border
            border.width: 1
        }

        contentItem: ColumnLayout {
            id: dialogCol
            spacing: 16
            width: parent ? parent.width : 400

            Text {
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                text: qsTr("This stops CPU mining and staking, shuts down veriumd and vericoind, locks your wallets, and closes the application.")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 13
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 8
                AppButton {
                    text: qsTr("Cancel")
                    variant: "secondary"
                    size: "sm"
                    onClicked: confirmDialog.close()
                }
                AppButton {
                    text: qsTr("Quit wallet")
                    variant: "danger"
                    size: "sm"
                    onClicked: {
                        quitBtn.quitting = true
                        confirmDialog.close()
                        Qt.quit()
                    }
                }
            }
        }
    }
}
