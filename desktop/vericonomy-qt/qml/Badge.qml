import QtQuick
import com.vericonomy.verium

// Small status chip (tone: neutral | accent | success | warning | danger).
Rectangle {
    property string text: ""
    property string tone: "neutral"

    readonly property color _c: {
        if (tone === "success") return Theme.success
        if (tone === "accent")  return Theme.accent
        if (tone === "warning") return Theme.warning
        if (tone === "danger")  return Theme.danger
        return Theme.fgMuted
    }

    radius: height / 2
    color: tone === "neutral" ? Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.8)
                              : Qt.rgba(_c.r, _c.g, _c.b, 0.12)
    border.width: 1
    border.color: tone === "neutral" ? Theme.border : Qt.rgba(_c.r, _c.g, _c.b, 0.35)
    implicitHeight: label.implicitHeight + 10
    implicitWidth: label.implicitWidth + 22

    Text {
        id: label
        anchors.centerIn: parent
        text: parent.text
        color: tone === "neutral" ? Theme.fgMuted : parent._c
        font.family: Theme.fontFamily
        font.pixelSize: 11
        font.weight: Font.DemiBold
    }
}
