import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri PoolStatsStrip — live public pool metrics.
Card {
    id: strip
    property var poolStats
    property var parseJson: function(s, fb) { try { return JSON.parse(s) } catch(e) { return fb || {} } }
    property bool pollEnabled: true

    readonly property var stats: poolStats ? parseJson(poolStats.statsJson, {}) : {}
    readonly property bool loading: poolStats ? poolStats.loading : false
    readonly property bool hasError: poolStats && poolStats.lastMessage.length > 0 && !stats.poolHashrate
    readonly property real poolHm: stats.poolHashrate !== undefined && stats.poolHashrate !== null
        ? stats.poolHashrate * 60.0 : NaN

    function fmtHashrate(hm) {
        if (isNaN(hm)) return "—"
        if (hm >= 1000) return (hm / 1000).toLocaleString(Qt.locale(), 'f', 1) + " kH/m"
        return hm.toLocaleString(Qt.locale(), 'f', 1) + " H/m"
    }

    function timeAgo(iso) {
        if (!iso || iso.length === 0) return ""
        var d = new Date(iso)
        if (isNaN(d.getTime())) return ""
        var sec = Math.max(0, Math.floor((Date.now() - d.getTime()) / 1000))
        if (sec < 60) return sec + "s ago"
        var min = Math.floor(sec / 60)
        if (min < 60) return min + "m ago"
        var hr = Math.floor(min / 60)
        if (hr < 24) return hr + "h ago"
        return Math.floor(hr / 24) + "d ago"
    }

    Component.onCompleted: if (pollEnabled && poolStats) poolStats.refresh()
    onPollEnabledChanged: if (pollEnabled && poolStats) poolStats.refresh()

    Timer {
        interval: 30000
        running: strip.pollEnabled && strip.visible
        repeat: true
        onTriggered: if (poolStats) poolStats.refresh()
    }

    padding: 20

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 16

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: qsTr("Public Verium Mining Pool")
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
            AppButton {
                text: qsTr("Mining Pool Dashboard")
                variant: "secondary"
                size: "sm"
                onClicked: HostLinks.open("https://pool.vericonomy.com")
            }
        }

        Text {
            visible: hasError
            text: qsTr("Pool stats unavailable. Check network or build with POOL_SUPABASE_ANON_KEY.")
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 13
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        GridLayout {
            visible: !hasError
            Layout.fillWidth: true
            columns: 5
            columnSpacing: 12
            rowSpacing: 12

            Repeater {
                model: [
                    { label: qsTr("Pool hashrate"), value: strip.fmtHashrate(strip.poolHm) },
                    { label: qsTr("Active miners"), value: stats.activeMiners !== undefined ? String(stats.activeMiners) : "—" },
                    { label: qsTr("Active workers"), value: stats.activeWorkers !== undefined ? String(stats.activeWorkers) : "—" },
                    { label: qsTr("Blocks found"), value: stats.blocksFoundTotal !== undefined ? String(stats.blocksFoundTotal) : "—" },
                    { label: qsTr("Pool fee"), value: stats.poolFeePct !== undefined ? stats.poolFeePct + "%" : "—" }
                ]
                delegate: ColumnLayout {
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        text: modelData.label.toUpperCase()
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.letterSpacing: 0.5
                    }
                    Text {
                        text: loading && modelData.value === "—" ? "…" : modelData.value
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                }
            }
        }

        Text {
            visible: stats.lastBlockAt !== undefined && stats.lastBlockAt !== null
                && String(stats.lastBlockAt).length > 0
            text: qsTr("Last block found ") + timeAgo(stats.lastBlockAt)
            color: Theme.fgSubtle
            font.family: Theme.fontFamily
            font.pixelSize: 11
            Layout.fillWidth: true
        }
    }
}
