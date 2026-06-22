import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningHashrateChart — session hashrate history (Canvas area chart).
Card {
    id: chart
    property var samples: []
    property bool active: false
    property bool emptyWhenIdle: false

    readonly property bool showEmpty: emptyWhenIdle && !active && samples.length === 0
    readonly property real maxHash: {
        var m = 0
        for (var i = 0; i < samples.length; i++)
            m = Math.max(m, samples[i].hashrate || 0)
        return m > 0 ? m : 1
    }

    padding: 20

    ColumnLayout {
        spacing: 12
        Layout.fillWidth: true

        RowLayout {
            Layout.fillWidth: true
            NavIcon { name: "cpu"; tint: Theme.accent }
            Text {
                text: qsTr("Local hashrate")
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
            AppButton {
                text: qsTr("Profitability calculator")
                variant: "ghost"
                size: "sm"
                onClicked: HostLinks.open("https://explorer.vericonomy.com/profitability")
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 220

            Rectangle {
                anchors.fill: parent
                radius: Theme.radiusMd
                visible: showEmpty
                color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.35)
                border.color: Theme.border
                border.width: 1
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 8
                    NavIcon { name: "cpu"; tint: Theme.fgSubtle; Layout.alignment: Qt.AlignHCenter }
                    Text {
                        text: qsTr("Start mining to see hashrate history for this session.")
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 13
                        horizontalAlignment: Text.AlignHCenter
                        Layout.preferredWidth: 280
                        wrapMode: Text.Wrap
                    }
                }
            }

            Canvas {
                id: canvas
                anchors.fill: parent
                visible: !showEmpty
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    if (samples.length < 2)
                        return

                    var padL = 8, padR = 8, padT = 8, padB = 24
                    var w = width - padL - padR
                    var h = height - padT - padB

                    ctx.strokeStyle = Qt.rgba(Theme.border.r, Theme.border.g, Theme.border.b, 0.8)
                    ctx.lineWidth = 1
                    ctx.beginPath()
                    ctx.moveTo(padL, padT + h)
                    ctx.lineTo(padL + w, padT + h)
                    ctx.stroke()

                    ctx.fillStyle = Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.15)
                    ctx.strokeStyle = Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.9)
                    ctx.lineWidth = 2
                    ctx.beginPath()
                    for (var i = 0; i < samples.length; i++) {
                        var x = padL + (i / Math.max(1, samples.length - 1)) * w
                        var y = padT + h - ((samples[i].hashrate || 0) / maxHash) * h
                        if (i === 0)
                            ctx.moveTo(x, y)
                        else
                            ctx.lineTo(x, y)
                    }
                    ctx.lineTo(padL + w, padT + h)
                    ctx.lineTo(padL, padT + h)
                    ctx.closePath()
                    ctx.fill()
                    ctx.beginPath()
                    for (var j = 0; j < samples.length; j++) {
                        var x2 = padL + (j / Math.max(1, samples.length - 1)) * w
                        var y2 = padT + h - ((samples[j].hashrate || 0) / maxHash) * h
                        if (j === 0)
                            ctx.moveTo(x2, y2)
                        else
                            ctx.lineTo(x2, y2)
                    }
                    ctx.stroke()
                }
            }

            Text {
                anchors.centerIn: parent
                visible: !showEmpty && samples.length === 0
                text: qsTr("No samples yet.")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 13
            }
        }
    }

    onSamplesChanged: canvas.requestPaint()
    onActiveChanged: canvas.requestPaint()
}
