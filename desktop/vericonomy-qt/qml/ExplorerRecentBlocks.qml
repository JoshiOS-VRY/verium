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
    readonly property int hPad: 16
    readonly property int colCount: 7
    readonly property var colWidths: [0.13, 0.13, 0.08, 0.14, 0.10, 0.16, 0.26]
    readonly property var colAlign: [
        Text.AlignLeft,
        Text.AlignRight,
        Text.AlignRight,
        Text.AlignRight,
        Text.AlignRight,
        Text.AlignRight,
        Text.AlignLeft
    ]
    readonly property var headerLabels: [
        qsTr("Height"),
        qsTr("Time"),
        qsTr("Txs"),
        qsTr("Out"),
        qsTr("Size"),
        qsTr("Difficulty"),
        qsTr("Extracted by")
    ]

    readonly property real contentWidth: Math.max(0, width - hPad * 2)

    implicitHeight: tableCol.implicitHeight

    property int ageTick: 0
    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: panel.ageTick++
    }

    function cellWidth(colIndex) {
        if (contentWidth <= 0)
            return 0
        if (colIndex === colCount - 1) {
            var used = 0
            for (var i = 0; i < colCount - 1; i++)
                used += Math.floor(contentWidth * colWidths[i])
            return Math.max(0, contentWidth - used)
        }
        return Math.floor(contentWidth * colWidths[colIndex])
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

    function explorerBase() {
        return "https://explorer.vericonomy.com/" + (coin === "vericoin" ? "vrc" : "vrm")
    }

    function blockUrl(block) {
        if (!block) return ""
        var id = block.hash || block.height
        if (id === undefined || id === null) return ""
        return explorerBase() + "/block/" + encodeURIComponent(String(id))
    }

    function rowValues(block) {
        return [
            String(block.height),
            formatBlockAge(block.time),
            block.n_tx !== undefined ? String(block.n_tx) : "—",
            fmtOutput(block),
            block.size != null ? block.size + " B" : "—",
            fmtDiff(block.difficulty),
            shortAddr(block.miner_address)
        ]
    }

    Column {
        id: tableCol
        width: panel.width
        spacing: 0

        Rectangle {
            width: parent.width
            height: 36
            color: Theme.bgPanel

            Row {
                x: panel.hPad
                width: panel.contentWidth
                height: parent.height
                spacing: 0

                Repeater {
                    model: panel.headerLabels.length
                    delegate: TableCell {
                        required property int index
                        cellW: panel.cellWidth(index)
                        rowH: 36
                        text: panel.headerLabels[index].toUpperCase()
                        header: true
                        align: panel.colAlign[index]
                    }
                }
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

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var url = panel.blockUrl(modelData)
                        if (url.length > 0) HostLinks.open(url)
                    }
                }

                Row {
                    x: panel.hPad
                    width: panel.contentWidth
                    height: parent.height
                    spacing: 0

                    Repeater {
                        model: panel.rowValues(modelData)
                        delegate: TableCell {
                            required property int index
                            required property var modelData
                            cellW: panel.cellWidth(index)
                            rowH: 44
                            text: modelData
                            header: false
                            align: panel.colAlign[index]
                            accent: index === 0
                            bold: index === 0
                            muted: index !== 0
                            mono: index === 5 || index === 6
                        }
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
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: qsTr("No blocks returned.")
            color: Theme.fgSubtle
            font.pixelSize: 13
            topPadding: 24
            bottomPadding: 24
        }
    }

    component TableCell: Item {
        property real cellW: 0
        property int rowH: 44
        property string text: ""
        property bool header: false
        property int align: Text.AlignRight
        property bool bold: false
        property bool muted: false
        property bool mono: false
        property bool accent: false

        width: cellW
        height: rowH
        clip: true

        Text {
            anchors.fill: parent
            anchors.rightMargin: parent.align === Text.AlignRight ? 4 : 0
            anchors.leftMargin: parent.align === Text.AlignLeft ? 0 : 4
            text: parent.text
            color: parent.accent ? Theme.accent
                : (parent.muted ? Theme.fgMuted : (parent.header ? Theme.fgSubtle : Theme.fg))
            font.family: parent.mono ? Theme.monoFamily : Theme.fontFamily
            font.pixelSize: parent.header ? 10 : (parent.mono ? 11 : 12)
            font.weight: parent.bold || parent.header ? Font.DemiBold : Font.Normal
            font.letterSpacing: parent.header ? 0.5 : 0
            horizontalAlignment: parent.align
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
    }
}
