import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri ExplorerRecentBlocks table parity (dashboard variant).
Item {
    id: panel
    property var blocks: []
    property string coin: "verium"
    readonly property bool isVerium: coin !== "vericoin"
    readonly property string ticker: isVerium ? "VRM" : "VRC"

    implicitWidth: parent ? parent.width : 640
    implicitHeight: tableCol.implicitHeight

    property int ageTick: 0
    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: panel.ageTick++
    }

    function formatBlockAge(unixSeconds) {
        if (!unixSeconds || unixSeconds <= 0) return "—"
        var total = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds)
        if (total < 60) return total + "s"
        var mins = Math.floor(total / 60)
        var secs = total % 60
        if (mins < 60) return secs > 0 ? mins + "m " + secs + "s" : mins + "m"
        var hours = Math.floor(mins / 60)
        var remMins = mins % 60
        if (hours < 24) return remMins > 0 ? hours + "h " + remMins + "m" : hours + "h"
        return Math.floor(hours / 24) + "d"
    }

    function fmtOutput(block) {
        var raw = block.output_total || block.mint
        if (!raw) return "—"
        var n = Number(raw)
        if (isNaN(n)) return raw
        return n.toLocaleString(Qt.locale(), 'f', 4) + " " + ticker
    }

    function fmtDiff(d) {
        if (!d) return "—"
        var n = Number(d)
        if (isNaN(n)) return d
        if (n < 0.00001) return n.toExponential(2)
        return n.toLocaleString(Qt.locale(), 'f', 7)
    }

    function shortAddr(addr) {
        if (!addr || addr.length < 12) return addr || "—"
        return addr.slice(0, 8) + "…" + addr.slice(-6)
    }

    ColumnLayout {
        id: tableCol
        width: parent.width
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            height: 36
            color: Theme.bgPanel
            Row {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                spacing: 0
                width: parent.width - 32
                BlockHeader { label: qsTr("Height"); fw: 0.13; align: Text.AlignLeft }
                BlockHeader { label: qsTr("Time"); fw: 0.13 }
                BlockHeader { label: qsTr("Txs"); fw: 0.08 }
                BlockHeader { label: qsTr("Out"); fw: 0.14 }
                BlockHeader { label: qsTr("Size"); fw: 0.10 }
                BlockHeader { label: qsTr("Difficulty"); fw: 0.16 }
                BlockHeader { label: qsTr("Extracted by"); fw: 0.26; align: Text.AlignLeft }
            }
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        Repeater {
            model: panel.blocks.slice(0, 10)
            delegate: Rectangle {
                required property var modelData
                required property int index
                width: panel.width
                height: 44
                color: index % 2 === 0 ? "transparent"
                    : Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.25)
                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 16
                    anchors.rightMargin: 16
                    spacing: 0
                    width: parent.width - 32
                    BlockCell {
                        value: String(modelData.height)
                        fw: 0.13
                        accent: true
                        bold: true
                        align: Text.AlignLeft
                    }
                    BlockCell {
                        value: panel.formatBlockAge(modelData.time)
                        fw: 0.13
                        muted: true
                    }
                    BlockCell {
                        value: modelData.n_tx !== undefined ? String(modelData.n_tx) : "—"
                        fw: 0.08
                        muted: true
                    }
                    BlockCell {
                        value: panel.fmtOutput(modelData)
                        fw: 0.14
                        muted: true
                    }
                    BlockCell {
                        value: modelData.size != null ? modelData.size + " B" : "—"
                        fw: 0.10
                        muted: true
                    }
                    BlockCell {
                        value: panel.fmtDiff(modelData.difficulty)
                        fw: 0.16
                        muted: true
                        mono: true
                    }
                    BlockCell {
                        value: panel.shortAddr(modelData.miner_address)
                        fw: 0.26
                        muted: true
                        mono: true
                        align: Text.AlignLeft
                    }
                }
                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: 1
                    color: Theme.border
                    opacity: 0.5
                }
            }
        }

        Text {
            visible: panel.blocks.length === 0
            text: qsTr("No blocks returned.")
            color: Theme.fgSubtle
            font.pixelSize: 13
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            Layout.topMargin: 24
            Layout.bottomMargin: 24
        }
    }

    component BlockHeader: Text {
        property string label: ""
        property real fw: 0.1
        property int align: Text.AlignRight
        width: parent.width * fw
        text: label.toUpperCase()
        color: Theme.fgSubtle
        font.pixelSize: 10
        font.weight: Font.DemiBold
        font.letterSpacing: 0.5
        horizontalAlignment: align
        elide: Text.ElideRight
    }

    component BlockCell: Text {
        property string value: ""
        property real fw: 0.1
        property bool bold: false
        property bool muted: false
        property bool mono: false
        property bool accent: false
        property int align: Text.AlignRight
        width: parent.width * fw
        text: value
        color: accent ? Theme.accent : (muted ? Theme.fgMuted : Theme.fg)
        font.family: mono ? Theme.monoFamily : Theme.fontFamily
        font.pixelSize: mono ? 11 : 12
        font.weight: bold ? Font.DemiBold : Font.Normal
        horizontalAlignment: align
        elide: Text.ElideRight
    }
}
