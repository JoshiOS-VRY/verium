import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Payment QR (Tauri QrCodeDisplay parity).
ColumnLayout {
    id: root
    property string coin: "verium"
    property string address: ""
    property real amount: 0
    property bool includeAmount: false
    property string label: ""
    property string message: ""
    property int size: 160

    spacing: 12
    Layout.alignment: Qt.AlignHCenter

    readonly property string uri: address.length > 0
        ? QrHelper.paymentUri(coin, address, amount, includeAmount, label, message)
        : ""
    readonly property string dataUrl: uri.length > 0 ? QrHelper.pngDataUrl(uri) : ""

    property bool copied: false

    Rectangle {
        Layout.alignment: Qt.AlignHCenter
        width: root.size + 24
        height: root.size + 24
        radius: Theme.radiusLg
        color: "#ffffff"
        border.color: Theme.border
        border.width: 1
        visible: root.dataUrl.length > 0

        Image {
            anchors.centerIn: parent
            width: root.size
            height: root.size
            source: root.dataUrl
            fillMode: Image.PreserveAspectFit
            smooth: true
        }
    }

    Text {
        Layout.maximumWidth: 320
        Layout.fillWidth: true
        visible: root.uri.length > 0
        text: root.uri
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.Wrap
        color: Theme.fgMuted
        font.family: Theme.monoFamily
        font.pixelSize: 10
    }

    RowLayout {
        Layout.alignment: Qt.AlignHCenter
        spacing: 8
        visible: root.uri.length > 0
        AppButton {
            text: root.copied ? qsTr("Copied!") : qsTr("Copy URI")
            variant: root.copied ? "secondary" : "secondary"
            onClicked: {
                if (HostLinks.copyText(root.uri))
                    root.copied = true
            }
        }
        AppButton {
            text: qsTr("Copy address")
            variant: "secondary"
            onClicked: HostLinks.copyText(root.address)
        }
    }

    onUriChanged: root.copied = false
}
