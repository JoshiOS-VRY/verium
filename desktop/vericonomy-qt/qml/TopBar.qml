import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Top chrome: page title + live node status pill.
Rectangle {
    id: topbar
    property string title: ""
    property var node            // NodeController instance

    implicitHeight: 56
    color: Theme.bg

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width; height: 1
        color: Theme.border
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 24
        anchors.rightMargin: 24
        spacing: 12

        Text {
            text: topbar.title
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 18
            font.weight: Font.DemiBold
            Layout.fillWidth: true
        }

        StatusPill {
            visible: topbar.node !== undefined && topbar.node !== null
            loading: topbar.node ? topbar.node.loading : false
            tone: topbar.node && topbar.node.connected
                ? (topbar.node.stateLabel === "Synced" ? "success" : "accent")
                : "neutral"
            text: topbar.node
                ? (topbar.node.connected ? topbar.node.stateLabel : "Offline")
                : "—"
        }
    }
}
