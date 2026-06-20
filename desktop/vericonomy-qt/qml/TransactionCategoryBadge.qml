import QtQuick
import com.vericonomy.verium

// Distinct badge per wallet `listtransactions` category (Tauri badge-cat-* parity).
Rectangle {
    id: badge
    property string category: "unknown"

    readonly property var style: {
        switch (category) {
        case "receive":      return { bg: Qt.rgba(56/255, 189/255, 248/255, 0.14), fg: Qt.rgba(125/255, 211/255, 252/255, 1), border: Qt.rgba(56/255, 189/255, 248/255, 0.35) }
        case "send":         return { bg: Qt.rgba(Theme.danger.r, Theme.danger.g, Theme.danger.b, 0.12), fg: Qt.rgba(251/255, 113/255, 133/255, 1), border: Qt.rgba(Theme.danger.r, Theme.danger.g, Theme.danger.b, 0.35) }
        case "generate":     return { bg: Qt.rgba(Theme.success.r, Theme.success.g, Theme.success.b, 0.12), fg: Qt.rgba(74/255, 222/255, 128/255, 1), border: Qt.rgba(Theme.success.r, Theme.success.g, Theme.success.b, 0.35) }
        case "immature":     return { bg: Qt.rgba(251/255, 191/255, 36/255, 0.14), fg: Qt.rgba(252/255, 211/255, 77/255, 1), border: Qt.rgba(251/255, 191/255, 36/255, 0.35) }
        case "stake":        return { bg: Qt.rgba(167/255, 139/255, 250/255, 0.14), fg: Qt.rgba(196/255, 181/255, 253/255, 1), border: Qt.rgba(167/255, 139/255, 250/255, 0.35) }
        case "stake-mint":   return { bg: Qt.rgba(45/255, 212/255, 191/255, 0.14), fg: Qt.rgba(94/255, 234/255, 212/255, 1), border: Qt.rgba(45/255, 212/255, 191/255, 0.35) }
        case "stake-orphan": return { bg: Qt.rgba(251/255, 146/255, 60/255, 0.14), fg: Qt.rgba(253/255, 186/255, 116/255, 1), border: Qt.rgba(251/255, 146/255, 60/255, 0.35) }
        case "move":         return { bg: Qt.rgba(148/255, 163/255, 184/255, 0.14), fg: Qt.rgba(203/255, 213/255, 225/255, 1), border: Qt.rgba(148/255, 163/255, 184/255, 0.35) }
        case "orphan":       return { bg: Qt.rgba(217/255, 70/255, 239/255, 0.14), fg: Qt.rgba(232/255, 121/255, 249/255, 1), border: Qt.rgba(217/255, 70/255, 239/255, 0.35) }
        default:             return { bg: Theme.bgSubtle, fg: Theme.fgMuted, border: Theme.border }
        }
    }

    readonly property string label: {
        switch (category) {
        case "receive": return qsTr("Received")
        case "send": return qsTr("Sent")
        case "generate": return qsTr("Mined")
        case "immature": return qsTr("Mined (immature)")
        case "stake": return qsTr("Staked")
        case "stake-mint": return qsTr("Stake reward")
        case "stake-orphan": return qsTr("Orphaned stake")
        case "move": return qsTr("Internal transfer")
        case "orphan": return qsTr("Orphaned")
        default:
            return category.split("-").filter(function(p) { return p.length > 0 })
                .map(function(p) { return p.charAt(0).toUpperCase() + p.slice(1) }).join(" ")
        }
    }

    radius: 4
    color: style.bg
    border.color: style.border
    border.width: 1
    implicitHeight: labelText.implicitHeight + 8
    implicitWidth: labelText.implicitWidth + 16

    Text {
        id: labelText
        anchors.centerIn: parent
        text: badge.label
        color: badge.style.fg
        font.family: Theme.fontFamily
        font.pixelSize: 11
        font.weight: Font.DemiBold
    }
}
