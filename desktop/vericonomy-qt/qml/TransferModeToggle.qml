import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Send / Receive segmented control (Tauri TransferModeToggle parity).
RowLayout {
    id: toggle
    property string mode: "send"

    spacing: 0
    implicitHeight: 36
    implicitWidth: 200

    Rectangle {
        Layout.fillWidth: true
        Layout.fillHeight: true
        radius: Theme.radiusMd
        color: Theme.bgSubtle
        border.color: Theme.border
        border.width: 1

        RowLayout {
            anchors.fill: parent
            anchors.margins: 4
            spacing: 2

            Repeater {
                model: [
                    { id: "send", label: qsTr("Send"), glyph: "\u2197" },
                    { id: "receive", label: qsTr("Receive"), glyph: "\u2199" }
                ]
                delegate: Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: Theme.radiusSm
                    color: toggle.mode === modelData.id ? Theme.accent : "transparent"

                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 6
                        Text {
                            text: modelData.glyph
                            color: toggle.mode === modelData.id ? Theme.accentFg : Theme.fgMuted
                            font.pixelSize: 12
                        }
                        Text {
                            text: modelData.label
                            color: toggle.mode === modelData.id ? Theme.accentFg : Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Medium
                        }
                    }

                    TapHandler {
                        onTapped: toggle.mode = modelData.id
                    }
                }
            }
        }
    }
}
