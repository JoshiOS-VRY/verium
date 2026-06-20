import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string title: ""
    property string subtitle: ""
    property string glyph: "\u25C9"
    property var features: []

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            padding: 28

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    Rectangle {
                        Layout.preferredWidth: 40
                        Layout.preferredHeight: 40
                        radius: 12
                        color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.14)
                        Text {
                            anchors.centerIn: parent
                            text: page.glyph
                            color: Theme.accent
                            font.pixelSize: 20
                        }
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        Text {
                            text: page.title
                            color: Theme.fg
                            font.family: Theme.fontFamily
                            font.pixelSize: 20
                            font.weight: Font.Bold
                        }
                        Text {
                            text: page.subtitle
                            color: Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                    }
                }

                Repeater {
                    model: page.features
                    delegate: RowLayout {
                        required property var modelData
                        Layout.fillWidth: true
                        spacing: 8
                        Text { text: "\u2713"; color: Theme.success; font.pixelSize: 13 }
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
