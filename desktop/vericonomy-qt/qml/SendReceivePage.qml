import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string coin: "verium"
    property var wallet
    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    property int tab: 0

    Component.onCompleted: if (page.wallet) page.wallet.refreshAddress()

    onCoinChanged: if (page.wallet) page.wallet.refreshAddress()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Rectangle {
            Layout.preferredWidth: 240
            implicitHeight: 36
            radius: Theme.radiusMd
            color: Theme.bgSubtle
            border.color: Theme.border
            border.width: 1
            RowLayout {
                anchors.fill: parent
                anchors.margins: 3
                spacing: 3
                Repeater {
                    model: [ qsTr("Send"), qsTr("Receive") ]
                    delegate: Rectangle {
                        required property int index
                        required property string modelData
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        radius: Theme.radiusSm
                        color: page.tab === index ? Theme.bgPanel : "transparent"
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            color: page.tab === index ? Theme.fg : Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            font.weight: Font.Medium
                        }
                        TapHandler { onTapped: page.tab = index }
                    }
                }
            }
        }

        Card {
            visible: page.tab === 0
            Layout.fillWidth: true
            Layout.maximumWidth: 560
            padding: 24
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14
                SectionHeader { title: qsTr("Send %1").arg(page.ticker); Layout.fillWidth: true }

                LabeledField { id: addr; label: qsTr("Recipient address"); placeholder: "V…" }
                LabeledField { id: amount; label: qsTr("Amount (%1)").arg(page.ticker); placeholder: "0.0000" }

                Text {
                    visible: page.wallet && page.wallet.lastMessage.length > 0
                    text: page.wallet ? page.wallet.lastMessage : ""
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Text {
                        text: qsTr("Available: ") +
                              (page.wallet ? page.wallet.balance.toLocaleString(Qt.locale(),'f',4) : "0") +
                              " " + page.ticker
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        Layout.fillWidth: true
                    }
                }

                AppButton {
                    Layout.fillWidth: true
                    text: qsTr("Send")
                    enabled: addr.text.length > 0 && amount.text.length > 0 && page.wallet
                    onClicked: {
                        var v = parseFloat(amount.text)
                        if (!isNaN(v) && page.wallet)
                            page.wallet.send(addr.text, v)
                    }
                }
            }
        }

        Card {
            visible: page.tab === 1
            Layout.fillWidth: true
            Layout.maximumWidth: 560
            padding: 24
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14
                SectionHeader { title: qsTr("Receive %1").arg(page.ticker); Layout.fillWidth: true }

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
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("Copy address")
                        variant: "secondary"
                        enabled: page.wallet && page.wallet.receiveAddress.length > 0
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
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
