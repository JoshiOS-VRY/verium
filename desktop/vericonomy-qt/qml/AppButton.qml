import QtQuick
import QtQuick.Controls.Basic
import com.vericonomy.verium

// Mirrors desktop/verium-app/src/components/ui/Button.tsx
// variants: primary | secondary | ghost | danger ; sizes: sm | md | lg
Button {
    id: control

    property string variant: "primary"
    property string size: "md"

    readonly property int _h: size === "sm" ? 32 : size === "lg" ? 44 : 36
    readonly property int _px: size === "sm" ? 12 : size === "lg" ? 24 : 16
    readonly property int _fs: size === "sm" ? 12 : size === "lg" ? 16 : 14

    implicitHeight: _h
    leftPadding: _px
    rightPadding: _px
    font.family: Theme.fontFamily
    font.pixelSize: _fs
    font.weight: Font.Medium

    readonly property color _bg: {
        if (variant === "primary") return Theme.accent
        if (variant === "danger")  return Theme.danger
        if (variant === "secondary") return Theme.bgPanel
        return "transparent" // ghost
    }
    readonly property color _fg: {
        if (variant === "primary") return Theme.accentFg
        if (variant === "danger")  return "#ffffff"
        return Theme.fg
    }
    readonly property bool _bordered: variant === "secondary"

    contentItem: Text {
        text: control.text
        color: control._fg
        font: control.font
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: Theme.radiusMd
        border.width: control._bordered ? 1 : 0
        border.color: Theme.border
        color: {
            var base = control._bg
            if (variant === "ghost")
                return control.hovered ? Theme.bgPanel : "transparent"
            if (control.down)    return Qt.darker(base, 1.12)
            if (control.hovered) return Qt.lighter(base, 1.06)
            return base
        }
        opacity: control.enabled ? 1.0 : 0.5
        Behavior on color { ColorAnimation { duration: 120 } }
    }
}
