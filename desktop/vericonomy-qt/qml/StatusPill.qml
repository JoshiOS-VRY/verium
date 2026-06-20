import QtQuick
import QtQuick.Controls.Basic
import com.vericonomy.verium

// Status pill with optional loading spinner (DashboardHero StatusPill).
Row {
    property string text: ""
    property string tone: "neutral" // neutral | success | accent
    property bool loading: false
    spacing: 6

    readonly property color _c: tone === "success" ? Theme.success
                              : tone === "accent"  ? Theme.accent
                              : Theme.fgMuted

    BusyIndicator {
        visible: loading
        running: loading
        implicitWidth: 14
        implicitHeight: 14
        anchors.verticalCenter: parent.verticalCenter
    }

    Rectangle {
        radius: height / 2
        color: tone === "neutral" ? Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.8)
                                  : Qt.rgba(parent._c.r, parent._c.g, parent._c.b, 0.10)
        border.width: 1
        border.color: tone === "neutral" ? Theme.border : Qt.rgba(parent._c.r, parent._c.g, parent._c.b, 0.25)
        implicitHeight: t.implicitHeight + 8
        implicitWidth: t.implicitWidth + 20
        anchors.verticalCenter: parent.verticalCenter

        Text {
            id: t
            anchors.centerIn: parent
            text: parent.parent.text
            color: parent.parent._c
            font.family: Theme.fontFamily
            font.pixelSize: 11
            font.weight: Font.DemiBold
        }
    }
}
