import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri SendConfirmDialog parity — countdown + fee summary before broadcast.
Dialog {
    id: dialog
    modal: true
    focus: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.NoButton
    width: Math.min(520, 560)

    property string coin: "verium"
    property string address: ""
    property string label: ""
    property real amount: 0
    property real feeRate: 0.001
    property bool extraConfirmDelay: false
    property bool confirming: false
    property bool twoFactorRequired: false
    property string lookalikeWarning: ""

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property int confirmDelaySec: extraConfirmDelay ? 8 : 3
    readonly property real feeEstimate: Math.max(feeRate * 0.25, 0.0001)
    readonly property real totalDebited: amount + feeEstimate

    signal confirmed(string totpCode)
    signal cancelled()

    onOpened: {
        countdown.value = confirmDelaySec
        totpField.text = ""
    }

    background: Rectangle {
        radius: Theme.radiusLg
        color: Theme.bgPanel
        border.color: Theme.border
        border.width: 1
    }

    header: Rectangle {
        height: 52
        color: "transparent"
        Text {
            anchors.left: parent.left
            anchors.leftMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Confirm send coins")
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 16
            font.weight: Font.DemiBold
        }
    }

    contentItem: ColumnLayout {
        spacing: 12
        width: dialog.width - 40

        Text {
            text: qsTr("Are you sure you want to send?")
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 14
            font.weight: Font.DemiBold
        }
        Text {
            text: qsTr("Please review your transaction.")
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
        }
        Text {
            visible: extraConfirmDelay
            text: qsTr("First send to this address — extra review time before confirm is enabled.")
            color: Theme.warning
            font.family: Theme.fontFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
        Text {
            visible: lookalikeWarning.length > 0
            text: lookalikeWarning
            color: Theme.warning
            font.family: Theme.fontFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        Text {
            text: label.length > 0 ? label + " · " + address : address
            color: Theme.fg
            font.family: Theme.monoFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

        ColumnLayout {
            spacing: 4
            Layout.fillWidth: true
            Text {
                text: qsTr("Transaction fee (est.)")
                color: Theme.fg
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            Text {
                text: feeEstimate.toLocaleString(Qt.locale(), "f", 8) + " " + ticker
                color: Theme.danger
                font.pixelSize: 13
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

        Text {
            text: qsTr("Total debited: ") +
                  totalDebited.toLocaleString(Qt.locale(), "f", 8) + " " + ticker
            color: Theme.fg
            font.pixelSize: 13
            font.weight: Font.DemiBold
        }

        ColumnLayout {
            visible: twoFactorRequired
            Layout.fillWidth: true
            spacing: 6
            Text { text: qsTr("2FA code"); color: Theme.fgMuted; font.pixelSize: 12 }
            TextField {
                id: totpField
                Layout.fillWidth: true
                placeholderText: qsTr("6-digit code")
                maximumLength: 8
            }
        }
    }

    footer: RowLayout {
        spacing: 10
        Item { Layout.fillWidth: true }
        AppButton {
            text: qsTr("Cancel")
            variant: "secondary"
            size: "sm"
            enabled: !confirming
            onClicked: {
                dialog.close()
                dialog.cancelled()
            }
        }
        AppButton {
            text: confirming ? qsTr("Sending…")
                : (countdown.value > 0 ? qsTr("Yes (%1)").arg(countdown.value) : qsTr("Yes"))
            size: "sm"
            enabled: !confirming && countdown.value <= 0
                && (!twoFactorRequired || totpField.text.trim().length >= 6)
            onClicked: dialog.confirmed(totpField.text.trim())
        }
    }

    QtObject {
        id: countdown
        property int value: 3
    }

    Timer {
        interval: 1000
        running: dialog.visible && countdown.value > 0 && !confirming
        repeat: true
        onTriggered: countdown.value = Math.max(0, countdown.value - 1)
    }
}
