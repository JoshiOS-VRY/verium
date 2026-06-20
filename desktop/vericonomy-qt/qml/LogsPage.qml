import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var logs
    property var parseJson
    readonly property var entries: logs ? parseJson(logs.linesJson, []) : []

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 12

        SectionHeader { title: qsTr("Logs"); Layout.fillWidth: true }

        Card {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 8

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.entries
                    delegate: RowLayout {
                        required property var modelData
                        width: ListView.view.width
                        spacing: 8
                        Text {
                            text: (modelData.level || "info").toUpperCase()
                            color: {
                                if (modelData.level === "error") return Theme.danger
                                if (modelData.level === "warn") return Theme.warning
                                return Theme.fgMuted
                            }
                            font.family: Theme.monoFamily
                            font.pixelSize: 11
                            font.weight: Font.DemiBold
                            Layout.preferredWidth: 48
                        }
                        Text {
                            text: modelData.message || ""
                            color: Theme.fg
                            font.family: Theme.monoFamily
                            font.pixelSize: 12
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }
    }
}
