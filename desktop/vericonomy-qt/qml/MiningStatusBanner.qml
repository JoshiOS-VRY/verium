import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningStatusBanner — sync / immature warnings above mining controls.
ColumnLayout {
    id: banner
    property bool syncStalled: false
    property bool chainSynced: true
    property bool ibd: false
    property int localBlocks: 0
    property int syncTarget: 0
    property int blocksBehind: 0
    property real immatureBalance: 0

    spacing: 8
    visible: childrenVisible
    readonly property bool childrenVisible: syncStalled || !chainSynced || immatureBalance > 0

    function fmt(n) {
        return Number(n).toLocaleString(Qt.locale(), 'f', 0)
    }

    function banner(text, tone) {
        return { text: text, tone: tone }
    }

    readonly property var items: {
        var out = []
        if (syncStalled) {
            out.push(banner(qsTr("Sync is stalled — check node status on the Network page."), "danger"))
        } else if (!chainSynced) {
            var msg = ibd
                ? qsTr("Mining is disabled while the node is syncing")
                : qsTr("Mining is disabled until the node reaches the network tip")
            msg += " (" + fmt(localBlocks)
            if (syncTarget > localBlocks)
                msg += " / ~" + fmt(syncTarget) + qsTr(" network tip")
            if (blocksBehind > 0)
                msg += " · ~" + fmt(blocksBehind) + qsTr(" blocks behind")
            msg += ")."
            out.push(banner(msg, "warning"))
        }
        if (immatureBalance > 0) {
            out.push(banner(
                qsTr("Pending from recent blocks: ") + immatureBalance.toLocaleString(Qt.locale(), 'f', 4)
                    + qsTr(" VRM (immature)"),
                "accent"))
        }
        return out
    }

    Repeater {
        model: banner.items
        delegate: Rectangle {
            required property var modelData
            Layout.fillWidth: true
            radius: Theme.radiusMd
            implicitHeight: txt.implicitHeight + 24
            color: modelData.tone === "danger"
                ? Qt.rgba(Theme.danger.r, Theme.danger.g, Theme.danger.b, 0.10)
                : modelData.tone === "warning"
                    ? Qt.rgba(Theme.warning.r, Theme.warning.g, Theme.warning.b, 0.10)
                    : Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.10)
            border.color: modelData.tone === "danger"
                ? Qt.rgba(Theme.danger.r, Theme.danger.g, Theme.danger.b, 0.30)
                : modelData.tone === "warning"
                    ? Qt.rgba(Theme.warning.r, Theme.warning.g, Theme.warning.b, 0.30)
                    : Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.30)
            border.width: 1
            Text {
                id: txt
                anchors.fill: parent
                anchors.margins: 12
                text: modelData.text
                wrapMode: Text.Wrap
                color: modelData.tone === "danger" ? Theme.danger
                    : modelData.tone === "warning" ? Theme.warning : Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 13
            }
        }
    }
}
