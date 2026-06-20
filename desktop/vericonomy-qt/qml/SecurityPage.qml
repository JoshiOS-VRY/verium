import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            padding: 24
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12
                SectionHeader {
                    title: qsTr("Security")
                    subtitle: qsTr("Wallet protection & advanced features")
                    Layout.fillWidth: true
                }
                Repeater {
                    model: [
                        qsTr("Wallet encryption — lock/unlock via node RPC"),
                        qsTr("Message signing — available on Sign / Verify page"),
                        qsTr("Spending controls — planned in shared SDK"),
                        qsTr("Two-factor & recovery — planned in shared SDK"),
                        qsTr("Hardware wallet (PSBT) — planned")
                    ]
                    delegate: RowLayout {
                        required property string modelData
                        Layout.fillWidth: true
                        Text { text: "\u2022"; color: Theme.accent; font.pixelSize: 14 }
                        Text {
                            text: modelData
                            color: Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
