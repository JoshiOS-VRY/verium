import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Sidebar navigation row (Tauri NavLink parity).
Rectangle {
    id: item
    property string label: ""
    property string icon: "gauge"
    property bool selected: false
    property bool showLock: false
    signal activated()

    Layout.fillWidth: true
    implicitHeight: 36
    radius: Theme.radiusMd
    color: item.selected || hover.hovered ? Theme.bgPanel : "transparent"
    Behavior on color { ColorAnimation { duration: 100 } }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12
        spacing: 12

        NavIcon {
            name: item.icon
            tint: item.selected || hover.hovered ? Theme.fg : Theme.fgMuted
        }

        Text {
            text: item.label
            color: item.selected || hover.hovered ? Theme.fg : Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 14
            font.weight: Font.Medium
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        Item {
            Layout.preferredWidth: 14
            Layout.preferredHeight: 14
            visible: item.showLock
            NavIcon {
                anchors.centerIn: parent
                width: 14
                height: 14
                name: "lock"
                tint: Qt.rgba(251/255, 191/255, 36/255, 1)
            }
        }
    }

    HoverHandler { id: hover }
    TapHandler { onTapped: item.activated() }
}
