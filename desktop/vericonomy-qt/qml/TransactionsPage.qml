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
    property var parseJson

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property var poolPayoutTxids: transactions
        ? parseJson(transactions.poolPayoutTxidsJson, []) : []

    property string transferMode: "send"

    Component.onCompleted: if (page.wallet) page.wallet.refreshAddress()
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
                                enabled: sendAddr.text.length > 0 && sendAmount.text.length > 0 && page.wallet
                                onClicked: {
                                    var v = parseFloat(sendAmount.text)
                                    if (!isNaN(v) && page.wallet)
                                        page.wallet.send(sendAddr.text, v)
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.margins: 20
                            Layout.maximumWidth: 560
                            spacing: 14
                            visible: page.transferMode === "receive"

                            Rectangle {
                                Layout.alignment: Qt.AlignHCenter
                                width: 160; height: 160; radius: Theme.radiusLg
                                color: Theme.bgSubtle
                                border.color: Theme.border; border.width: 1
                                Text {
                                    anchors.centerIn: parent
                                    text: page.wallet && page.wallet.receiveAddress.length > 0
                                        ? qsTr("Address ready") : qsTr("Loading…")
                                    color: Theme.fgMuted
                                    font.pixelSize: 13
                                }
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: 44
                                radius: Theme.radiusMd
                                color: Theme.bgSubtle
                                border.color: Theme.border; border.width: 1
                                Text {
                                    anchors.fill: parent
                                    anchors.margins: 12
                                    verticalAlignment: Text.AlignVCenter
                                    text: page.wallet && page.wallet.receiveAddress.length
                                        ? page.wallet.receiveAddress : qsTr("No address yet")
                                    color: Theme.fg
                                    font.family: Theme.monoFamily
                                    font.pixelSize: 11
                                    wrapMode: Text.Wrap
                                }
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                AppButton {
                                    Layout.fillWidth: true
                                    text: qsTr("New address")
                                    variant: "secondary"
                                    onClicked: if (page.wallet) page.wallet.refreshAddress()
                                }
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

    Connections {
        target: page.wallet
        function onSendCompleted(ok) {
            if (ok && page.transactions) page.transactions.refresh()
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
