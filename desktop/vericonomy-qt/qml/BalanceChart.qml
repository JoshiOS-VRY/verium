import QtQuick
import com.vericonomy.verium

// Cumulative balance area chart with hover scrub + pinned value.
// Parity with desktop/verium-app/src/components/mobile/explorer/CumulativeBalanceChart.tsx
// (Recharts AreaChart). Uses Canvas so there is no external chart dependency.
Item {
    id: chart
    implicitHeight: 180

    // points: array of { t: <ms>, balance: <number> }
    property var points: []
    property string ticker: "VRM"
    property color lineColor: Theme.accent

    // Scrub state
    property int hoverIndex: -1
    readonly property bool scrubbing: hoverIndex >= 0
    readonly property var activePoint: points.length === 0
        ? null
        : points[scrubbing ? hoverIndex : points.length - 1]

    onPointsChanged: canvas.requestPaint()
    onWidthChanged: canvas.requestPaint()
    onHeightChanged: canvas.requestPaint()

    // Pinned value header
    Column {
        id: pinned
        spacing: 2
        Text {
            text: chart.scrubbing ? qsTr("Selected balance") : qsTr("Current balance")
            color: Theme.fgSubtle
            font.family: Theme.fontFamily
            font.pixelSize: 10
            font.weight: Font.Medium
            font.letterSpacing: 0.5
        }
        Text {
            text: chart.activePoint
                ? chart.activePoint.balance.toLocaleString(Qt.locale(), 'f', 4) + " " + chart.ticker
                : "—"
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 22
            font.weight: Font.Bold
        }
    }

    Canvas {
        id: canvas
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: parent.height - pinned.height - 8

        function bounds() {
            var pts = chart.points
            if (!pts || pts.length === 0) return null
            var min = pts[0].balance, max = pts[0].balance
            var tmin = pts[0].t, tmax = pts[0].t
            for (var i = 1; i < pts.length; i++) {
                min = Math.min(min, pts[i].balance)
                max = Math.max(max, pts[i].balance)
                tmin = Math.min(tmin, pts[i].t)
                tmax = Math.max(tmax, pts[i].t)
            }
            if (max === min) { max = min + 1 }
            if (tmax === tmin) { tmax = tmin + 1 }
            return { min: min, max: max, tmin: tmin, tmax: tmax }
        }

        function xAt(t, b) { return (t - b.tmin) / (b.tmax - b.tmin) * width }
        function yAt(v, b) { return height - (v - b.min) / (b.max - b.min) * (height - 8) - 4 }

        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            var pts = chart.points
            var b = bounds()
            if (!b) return

            // Area fill (gradient)
            ctx.beginPath()
            ctx.moveTo(0, height)
            for (var i = 0; i < pts.length; i++)
                ctx.lineTo(xAt(pts[i].t, b), yAt(pts[i].balance, b))
            ctx.lineTo(width, height)
            ctx.closePath()
            var grad = ctx.createLinearGradient(0, 0, 0, height)
            grad.addColorStop(0, Qt.rgba(chart.lineColor.r, chart.lineColor.g, chart.lineColor.b, 0.28))
            grad.addColorStop(1, Qt.rgba(chart.lineColor.r, chart.lineColor.g, chart.lineColor.b, 0.0))
            ctx.fillStyle = grad
            ctx.fill()

            // Line
            ctx.beginPath()
            for (var j = 0; j < pts.length; j++) {
                var x = xAt(pts[j].t, b), y = yAt(pts[j].balance, b)
                if (j === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y)
            }
            ctx.lineWidth = 2
            ctx.strokeStyle = chart.lineColor
            ctx.stroke()

            // Scrub marker
            if (chart.scrubbing && chart.activePoint) {
                var mx = xAt(chart.activePoint.t, b)
                ctx.beginPath()
                ctx.moveTo(mx, 0); ctx.lineTo(mx, height)
                ctx.strokeStyle = Qt.rgba(0.58, 0.64, 0.72, 0.4)
                ctx.lineWidth = 1
                ctx.stroke()
                ctx.beginPath()
                ctx.arc(mx, yAt(chart.activePoint.balance, b), 4, 0, Math.PI * 2)
                ctx.fillStyle = chart.lineColor
                ctx.fill()
            }
        }

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onPositionChanged: (m) => {
                var pts = chart.points
                if (!pts || pts.length === 0) return
                var idx = Math.round(m.x / canvas.width * (pts.length - 1))
                chart.hoverIndex = Math.max(0, Math.min(pts.length - 1, idx))
                canvas.requestPaint()
            }
            onExited: { chart.hoverIndex = -1; canvas.requestPaint() }
        }
    }
}
