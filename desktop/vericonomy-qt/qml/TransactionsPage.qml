import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri Transactions.tsx desktop parity — balance, send/receive, history table.
Item {
    id: page
    property string coin: "verium"
    property var transactions
    property var wallet
    property var walletMode
    property var security
    property var parseJson
    property real feeRate: 0.001
    property var sendRecipients: [{ address: "", amount: "" }]
    property string sendWarning: ""

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property var poolPayoutTxids: transactions
        ? parseJson(transactions.poolPayoutTxidsJson, []) : []

    property string transferMode: "send"

    Component.onCompleted: {
        if (page.wallet) page.wallet.refreshAddress()
        if (page.transactions) page.transactions.refresh()
    }
    onVisibleChanged: {
        if (visible && page.transactions) page.transactions.refresh()
    }

    // Tauri parity: poll history only while this page is open (not global refreshAll).
    Timer {
        interval: page.walletMode && page.walletMode.isLight ? 5000 : 45000
        running: page.visible && page.wallet && !page.wallet.locked
        repeat: true
        onTriggered: if (page.transactions) page.transactions.refresh()
    }

    onCoinChanged: {
        page.transferMode = "send"
        if (page.wallet) page.wallet.refreshAddress()
    }

    WalletUnlockGate {
        anchors.fill: parent
        wallet: page.wallet
        coin: page.coin

        Flickable {
            anchors.fill: parent
            contentWidth: width
            contentHeight: col.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            ColumnLayout {
                id: col
                width: parent.width
                spacing: 24

                WalletBalanceSummary {
                    Layout.fillWidth: true
                    wallet: page.wallet
                    coin: page.coin
                }

                Card {
                    Layout.fillWidth: true
                    padding: 0

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        RowLayout {
                            Layout.fillWidth: true
                            Layout.margins: 20
                            spacing: 16
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 4
                                Text {
                                    text: page.transferMode === "send" ? qsTr("Send") : qsTr("Receive")
                                    color: Theme.fg
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }
                                Text {
                                    Layout.fillWidth: true
                                    wrapMode: Text.Wrap
                                    color: Theme.fgMuted
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 12
                                    text: page.transferMode === "send"
                                        ? qsTr("Pay to one or more %1 addresses. Labels are saved locally with the transaction comment.").arg(page.displayName)
                                        : qsTr("Create %1 receiving addresses with optional label, amount, and message.").arg(page.ticker)
                                }
                            }
                            TransferModeToggle {
                                mode: page.transferMode
                                onModeChanged: page.transferMode = mode
                            }
                        }

                        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.margins: 20
                            Layout.maximumWidth: 560
                            spacing: 14
                            visible: page.transferMode === "send"

                            LabeledField { id: sendAddr; label: qsTr("Recipient address"); placeholder: "V…" }
                            LabeledField { id: sendAmount; label: qsTr("Amount (%1)").arg(page.ticker); placeholder: "0.0000" }
                            LabeledField { id: sendFee; label: qsTr("Fee rate (coins/kB)"); placeholder: "0.001"; text: String(page.feeRate) }

                            Text {
                                visible: page.sendWarning.length > 0
                                text: page.sendWarning
                                color: Theme.danger
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }

                            Text {
                                visible: page.wallet && page.wallet.lastMessage.length > 0
                                text: page.wallet ? page.wallet.lastMessage : ""
                                color: Theme.fgMuted
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }

                            Text {
                                text: qsTr("Available: ") +
                                      (page.wallet ? page.wallet.balance.toLocaleString(Qt.locale(), "f", 4) : "0") +
                                      " " + page.ticker
                                color: Theme.fgSubtle
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                Layout.fillWidth: true
                            }

                            AppButton {
                                Layout.fillWidth: true
                                text: qsTr("Send")
                                enabled: sendAddr.text.length > 0 && sendAmount.text.length > 0
                                    && page.wallet && page.security
                                onClicked: {
                                    var fee = parseFloat(sendFee.text)
                                    if (isNaN(fee) || fee <= 0) fee = page.feeRate
                                    page.feeRate = fee
                                    var primary = parseFloat(sendAmount.text)
                                    if (isNaN(primary) || primary <= 0) {
                                        page.sendWarning = qsTr("Enter a valid amount")
                                        return
                                    }
                                    page.sendWarning = ""
                                    page.security.checkSend(sendAddr.text, primary)
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.margins: 20
                            Layout.maximumWidth: 560
                            spacing: 14
                            visible: page.transferMode === "receive"

                            ReceivePanel {
                                Layout.fillWidth: true
                                coin: page.coin
                                wallet: page.wallet
                                security: page.security
                                parseJson: page.parseJson
                            }
                        }
                    }
                }

                Card {
                    Layout.fillWidth: true
                    padding: 0
                    TransactionHistoryTable {
                        Layout.fillWidth: true
                        transactions: page.transactions
                        parseJson: page.parseJson
                        coin: page.coin
                        poolPayoutTxids: page.poolPayoutTxids
                    }
                }

                Item { Layout.preferredHeight: 8 }
            }
        }
    }

    SendConfirmDialog {
        id: sendConfirm
        coin: page.coin
        onConfirmed: function(totpCode) {
            sendConfirm.confirming = true
            var fee = parseFloat(sendFee.text)
            if (isNaN(fee) || fee <= 0) fee = page.feeRate
            page.wallet.send(sendAddr.text, parseFloat(sendAmount.text), fee, totpCode, true)
        }
        onCancelled: sendConfirm.confirming = false
    }

    Connections {
        target: page.security
        function onSendCheckCompleted(ok) {
            if (!ok) {
                page.sendWarning = page.security ? page.security.lastMessage : qsTr("Send check failed")
                return
            }
            var check = page.parseJson(page.security.sendCheckJson, {})
            if (!check.allowed) {
                page.sendWarning = check.reason || qsTr("Send blocked by spending controls.")
                return
            }
            page.sendWarning = ""
            sendConfirm.address = sendAddr.text
            sendConfirm.amount = parseFloat(sendAmount.text)
            sendConfirm.feeRate = page.feeRate
            sendConfirm.extraConfirmDelay = check.requires_extra_confirmation === true
            sendConfirm.lookalikeWarning = check.look_alike_warning || ""
            sendConfirm.twoFactorRequired = check.two_factor_required === true
            sendConfirm.open()
        }
    }

    Connections {
        target: page.wallet
        function onSendCompleted(ok) {
            sendConfirm.confirming = false
            if (ok) {
                sendConfirm.close()
                sendAddr.text = ""
                sendAmount.text = ""
                if (page.transactions) page.transactions.refresh()
            }
        }
        function onUnlockCompleted(ok) {
            if (ok && page.transactions) page.transactions.refresh()
        }
    }

    component LabeledField: ColumnLayout {
        property string label: ""
        property string placeholder: ""
        property alias text: field.text
        spacing: 6
        Layout.fillWidth: true
        Text {
            text: label
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 40
            radius: Theme.radiusMd
            color: Theme.bgSubtle
            border.width: 1
            border.color: field.activeFocus ? Theme.accent : Theme.border
            TextField {
                id: field
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 12
                verticalAlignment: TextInput.AlignVCenter
                placeholderText: placeholder
                color: Theme.fg
                placeholderTextColor: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 13
                background: null
            }
        }
    }
}
