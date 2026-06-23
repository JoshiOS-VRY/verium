import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri BootstrapBanner — offer bootstrap import when chain is far behind.
Rectangle {
    id: banner
    property string coin: "verium"
    property var node
    property var explorer
    property var settings
    property var bootstrap
    property var walletMode
    property var parseJson: function(s, fb) { try { return JSON.parse(s) } catch(e) { return fb || {} } }

    readonly property var prefs: settings ? parseJson(settings.prefsJson, {}) : {}
    readonly property var explorerStats: explorer ? parseJson(explorer.statsJson, {}) : {}
    readonly property bool isLight: walletMode ? walletMode.isLight : false
    readonly property bool isTestNetwork: {
        if (!node || !node.chain) return false
        var c = String(node.chain).toLowerCase()
        return c.indexOf("binarytest") >= 0 || c === "test"
    }
    readonly property int networkTip: explorerStats.height !== undefined ? explorerStats.height : 0
    readonly property int syncTarget: {
        if (!node) return networkTip > 0 ? networkTip : 0
        return Math.max(node.headers, node.blocks, networkTip > 0 ? networkTip : 0)
    }
    readonly property int blocksBehind: node ? Math.max(0, syncTarget - node.blocks) : 0

    readonly property bool shouldShow: {
        if (isLight || isTestNetwork || !node || !node.initialBlockDownload)
            return false
        var dismissed = prefs.bootstrap_dismissed_at
        if (dismissed && (Date.now() / 1000 - dismissed) < 86400)
            return false
        var progress = node.verificationProgress
        var blocks = node.blocks
        var headers = node.headers
        var behind = blocksBehind
        if (progress >= 0.99 && blocks > 1000000) {
            var headerLag = Math.max(0, headers - blocks)
            if (headerLag < 500) return false
        }
        if (progress >= 0.95 && behind < 500) return false
        if (progress < 0.95) return true
        var now = Math.floor(Date.now() / 1000)
        if (node.medianTime > 0 && now - node.medianTime > 604800) return true
        if (Math.max(0, headers - blocks) > 1000) return true
        return behind > 1000
    }

    visible: shouldShow
    radius: Theme.radiusMd
    color: Qt.rgba(Theme.warning.r, Theme.warning.g, Theme.warning.b, 0.10)
    border.color: Qt.rgba(Theme.warning.r, Theme.warning.g, Theme.warning.b, 0.30)
    border.width: 1
    implicitHeight: inner.implicitHeight + 24
    Layout.fillWidth: true

    BootstrapDialog {
        id: bootstrapDialog
        coin: banner.coin
        bootstrap: banner.bootstrap
        parseJson: banner.parseJson
        node: banner.node
    }

    RowLayout {
        id: inner
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: 12
        spacing: 12

        Text {
            text: "\u2913"
            color: Theme.warning
            font.pixelSize: 16
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8

            Text {
                text: qsTr("Your chain is far behind the network. Importing the official bootstrap snapshot can jump you ahead faster than catching up over P2P.")
                color: Theme.warning
                font.family: Theme.fontFamily
                font.pixelSize: 13
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }

            Text {
                visible: node && node.blocks > 0
                text: {
                    var t = qsTr("Local block #") + node.blocks.toLocaleString(Qt.locale(), 'f', 0)
                    if (syncTarget > 0)
                        t += qsTr(" · network tip ~#") + syncTarget.toLocaleString(Qt.locale(), 'f', 0)
                    if (blocksBehind > 0)
                        t += qsTr(" · ~") + blocksBehind.toLocaleString(Qt.locale(), 'f', 0) + qsTr(" blocks remaining")
                    t += " · " + Math.round(node.verificationProgress * 100) + qsTr("% verified")
                    return t
                }
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 11
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }

            RowLayout {
                spacing: 8
                AppButton {
                    text: qsTr("Import bootstrap")
                    size: "sm"
                    onClicked: bootstrapDialog.open()
                }
                AppButton {
                    text: qsTr("Snooze for a day")
                    variant: "ghost"
                    size: "sm"
                    onClicked: {
                        if (!settings) return
                        var p = Object.assign({}, banner.prefs)
                        p.bootstrap_dismissed_at = Math.floor(Date.now() / 1000)
                        settings.savePrefs(JSON.stringify(p))
                    }
                }
            }
        }
    }
}
