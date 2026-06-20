import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string coin: "verium"
    property var wallet

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            Layout.maximumWidth: 640
            padding: 24
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14
                SectionHeader {
                    title: qsTr("Sign / Verify message")
                    subtitle: qsTr("Prove address ownership")
                    Layout.fillWidth: true
                }

                LabeledField { id: signAddr; label: qsTr("Address"); placeholder: "V…" }
                LabeledField { id: signMsg; label: qsTr("Message"); placeholder: qsTr("Message to sign") }

                AppButton {
                    text: qsTr("Sign")
                    enabled: page.wallet && signAddr.text.length && signMsg.text.length
                    onClicked: if (page.wallet) page.wallet.signMessage(signAddr.text, signMsg.text)
                }

                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: sigOut.implicitHeight + 20
                    radius: Theme.radiusMd
                    color: Theme.bgSubtle
                    border.color: Theme.border
                    Text {
                        id: sigOut
                        anchors.fill: parent
                        anchors.margins: 10
                        text: page.wallet ? page.wallet.lastMessage : ""
                        color: Theme.fgMuted
                        font.family: Theme.monoFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                    }
                }
            }
        }

        Card {
            Layout.fillWidth: true
            Layout.maximumWidth: 640
            padding: 24
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14
                SectionHeader { title: qsTr("Verify signature"); Layout.fillWidth: true }

                LabeledField { id: verifyAddr; label: qsTr("Address"); placeholder: "V…" }
                LabeledField { id: verifySig; label: qsTr("Signature"); placeholder: qsTr("Base64 signature") }
                LabeledField { id: verifyMsg; label: qsTr("Message"); placeholder: qsTr("Original message") }

                AppButton {
                    text: qsTr("Verify")
                    enabled: page.wallet && verifyAddr.text.length && verifySig.text.length && verifyMsg.text.length
                    onClicked: if (page.wallet)
                        page.wallet.verifyMessage(verifyAddr.text, verifySig.text, verifyMsg.text)
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
