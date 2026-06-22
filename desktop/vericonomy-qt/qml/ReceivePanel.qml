import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Receive panel with labeled requests (Tauri ReceivePanel parity).
ColumnLayout {
    id: panel
    property string coin: "verium"
    property var wallet
    property var security
    property var parseJson: function(s, fb) { return fb || [] }

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property var requests: security
        ? parseJson(security.receiveRequestsJson, [])
        : []

    property string recvLabel: ""
    property string recvAmount: ""
    property string recvMessage: ""
    property string selectedRequestId: ""
    property string displayAddress: wallet ? wallet.receiveAddress : ""

    readonly property var selectedRequest: {
        for (var i = 0; i < requests.length; i++) {
            if (requests[i].id === selectedRequestId)
                return requests[i]
        }
        return null
    }

    readonly property string qrAddress: selectedRequest
        ? selectedRequest.address
        : displayAddress
    readonly property real qrAmount: selectedRequest && selectedRequest.amount
        ? selectedRequest.amount : 0
    readonly property bool qrIncludeAmount: selectedRequest && selectedRequest.amount > 0
    readonly property string qrLabel: selectedRequest ? selectedRequest.label : recvLabel
    readonly property string qrMessage: selectedRequest ? selectedRequest.message : recvMessage

    spacing: 14
    Layout.fillWidth: true

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 8
        Text { text: qsTr("Label (optional)"); color: Theme.fgMuted; font.pixelSize: 12 }
        TextField {
            Layout.fillWidth: true
            text: panel.recvLabel
            onTextChanged: panel.recvLabel = text
            color: Theme.fg
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text { text: qsTr("Amount (%1, optional)").arg(panel.ticker); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    text: panel.recvAmount
                    placeholderText: "0.0000"
                    onTextChanged: panel.recvAmount = text
                    color: Theme.fg
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text { text: qsTr("Message (optional)"); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    text: panel.recvMessage
                    onTextChanged: panel.recvMessage = text
                    color: Theme.fg
                }
            }
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            AppButton {
                Layout.fillWidth: true
                text: qsTr("Generate payment request")
                enabled: wallet && security
                onClicked: {
                    if (!wallet || !security) return
                    if (wallet.receiveAddress.length === 0)
                        wallet.refreshAddress()
                    var amt = parseFloat(panel.recvAmount)
                    security.appendReceiveRequest(
                        panel.recvLabel,
                        panel.recvMessage,
                        isNaN(amt) ? 0 : amt,
                        !isNaN(amt) && amt > 0,
                        wallet.receiveAddress
                    )
                    wallet.refreshAddress()
                    panel.recvLabel = ""
                    panel.recvAmount = ""
                    panel.recvMessage = ""
                }
            }
            AppButton {
                text: qsTr("Address only")
                variant: "secondary"
                enabled: wallet
                onClicked: if (wallet) wallet.refreshAddress()
            }
        }
    }

    QrCodeDisplay {
        Layout.alignment: Qt.AlignHCenter
        coin: panel.coin
        address: panel.qrAddress
        amount: panel.qrAmount
        includeAmount: panel.qrIncludeAmount
        label: panel.qrLabel
        message: panel.qrMessage
        size: 160
    }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: 44
        radius: Theme.radiusMd
        color: Theme.bgSubtle
        border.color: Theme.border
        Text {
            anchors.fill: parent
            anchors.margins: 12
            verticalAlignment: Text.AlignVCenter
            text: panel.qrAddress.length ? panel.qrAddress : qsTr("No address yet")
            color: Theme.fg
            font.family: Theme.monoFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
        }
    }

    Text {
        visible: requests.length > 0
        text: qsTr("Saved requests")
        color: Theme.fg
        font.pixelSize: 13
        font.weight: Font.DemiBold
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 6
        visible: requests.length > 0
        Repeater {
            model: panel.requests
            delegate: Rectangle {
                required property var modelData
                Layout.fillWidth: true
                radius: Theme.radiusMd
                color: panel.selectedRequestId === modelData.id
                    ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.08)
                    : Theme.bgSubtle
                border.color: panel.selectedRequestId === modelData.id ? Theme.accent : Theme.border
                implicitHeight: inner.implicitHeight + 16
                RowLayout {
                    id: inner
                    anchors.fill: parent
                    anchors.margins: 10
                    spacing: 8
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        Text {
                            text: modelData.label.length ? modelData.label : qsTr("(no label)")
                            color: Theme.fg
                            font.pixelSize: 12
                            font.weight: Font.DemiBold
                        }
                        Text {
                            text: modelData.amount
                                ? modelData.amount.toLocaleString(Qt.locale(), "f", 4) + " " + panel.ticker
                                : qsTr("Any amount")
                            color: Theme.fgMuted
                            font.pixelSize: 11
                        }
                    }
                    AppButton {
                        text: qsTr("Show")
                        variant: "secondary"
                        size: "sm"
                        onClicked: panel.selectedRequestId = modelData.id
                    }
                    AppButton {
                        text: qsTr("Delete")
                        variant: "ghost"
                        size: "sm"
                        onClicked: if (security) security.deleteReceiveRequest(modelData.id)
                    }
                }
                TapHandler { onTapped: panel.selectedRequestId = modelData.id }
            }
        }
    }
}
