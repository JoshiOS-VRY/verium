import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var rpc
    property var parseJson
    readonly property var lines: rpc ? parseJson(rpc.historyJson, []) : []

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 12

        SectionHeader {
            title: qsTr("RPC console")
            subtitle: qsTr("Direct daemon commands · history in-memory only")
            Layout.fillWidth: true
        }

        Card {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 0

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.lines
                    delegate: ColumnLayout {
                        required property var modelData
                        width: ListView.view.width
                        spacing: 2
                        RowLayout {
                            Layout.leftMargin: 14
                            Layout.topMargin: 10
                            spacing: 6
                            Text { text: "\u203A"; color: Theme.accent; font.family: Theme.monoFamily; font.pixelSize: 12 }
                            Text { text: modelData.cmd; color: Theme.fg; font.family: Theme.monoFamily; font.pixelSize: 12 }
                        }
                        Text {
                            Layout.leftMargin: 26
                            Layout.rightMargin: 14
                            Layout.bottomMargin: 6
                            text: modelData.out
                            color: Theme.fgMuted
                            font.family: Theme.monoFamily
                            font.pixelSize: 12
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                    }
                }

                Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

                RowLayout {
                    Layout.fillWidth: true
                    Layout.margins: 10
                    spacing: 8
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 38
                        radius: Theme.radiusMd
                        color: Theme.bgSubtle
                        border.width: 1
                        border.color: input.activeFocus ? Theme.accent : Theme.border
                        TextField {
                            id: input
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            verticalAlignment: TextInput.AlignVCenter
                            placeholderText: qsTr("e.g. getblockchaininfo")
                            color: Theme.fg
                            placeholderTextColor: Theme.fgSubtle
                            font.family: Theme.monoFamily
                            font.pixelSize: 12
                            background: null
                            onAccepted: page.run()
                        }
                    }
                    AppButton {
                        text: qsTr("Run")
                        size: "sm"
                        enabled: input.text.length > 0 && !(page.rpc && page.rpc.loading)
                        onClicked: page.run()
                    }
                }
            }
        }
    }

    function run() {
        if (!page.rpc || input.text.length === 0) return
        page.rpc.call(input.text, "[]")
        input.text = ""
    }
}
