import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningStatTile — labeled stat card with optional highlight ring.
Card {
    id: tile
    property string label: ""
    property string value: "—"
    property string unit: ""
    property string hint: ""
    property string iconName: ""
    property bool highlight: false

    padding: 16
    border.color: highlight
        ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.35)
        : Theme.border
    border.width: highlight ? 1 : 1

    Rectangle {
        anchors.fill: parent
        radius: parent.radius
        visible: highlight
        color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.04)
        z: -1
    }

    ColumnLayout {
        spacing: 8
        Layout.fillWidth: true

        RowLayout {
            spacing: 8
            Layout.fillWidth: true
            NavIcon {
                visible: tile.iconName.length > 0
                name: tile.iconName
                tint: Theme.fgSubtle
            }
            Text {
                text: tile.label
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 13
                font.weight: Font.Medium
                Layout.fillWidth: true
            }
        }

        RowLayout {
            spacing: 4
            Text {
                text: tile.value
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 22
                font.weight: Font.DemiBold
            }
            Text {
                visible: tile.unit.length > 0
                text: tile.unit
                color: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 13
                Layout.alignment: Qt.AlignBottom
                Layout.bottomMargin: 2
            }
        }

        Text {
            visible: tile.hint.length > 0
            text: tile.hint
            color: Theme.fgSubtle
            font.family: Theme.fontFamily
            font.pixelSize: 11
            Layout.fillWidth: true
            wrapMode: Text.Wrap
        }
    }
}
