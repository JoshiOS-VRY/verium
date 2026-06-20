import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Confirmation ring + fraction label (Tauri ConfirmationProgress parity).
Item {
    id: root
    property int confirmations: 0
    property string category: ""

    readonly property int required: (category === "immature" || category === "generate") ? 101 : 1
    readonly property real progress: Math.min(1, Math.max(0, confirmations / Math.max(1, required)))
    readonly property bool complete: confirmations >= required
    readonly property int remaining: (category === "immature" || category === "generate")
        ? Math.max(0, required - confirmations) : 0

    readonly property color ringColor: complete ? Theme.success
        : (progress > 0 ? ((category === "immature" || category === "generate") ? Theme.warning : Theme.accent)
                        : Theme.fgSubtle)

    implicitWidth: row.implicitWidth
    implicitHeight: 32

    RowLayout {
        id: row
        anchors.right: parent.right
        spacing: 8

        Item {
            width: 32; height: 32
            Canvas {
                anchors.fill: parent
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    var cx = 16, cy = 16, r = 12.25, stroke = 3.5
                    ctx.lineWidth = stroke
                    ctx.strokeStyle = Theme.border
                    ctx.beginPath()
                    ctx.arc(cx, cy, r, 0, 2 * Math.PI)
                    ctx.stroke()
                    ctx.strokeStyle = root.ringColor
                    ctx.lineCap = "round"
                    ctx.beginPath()
                    ctx.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + 2 * Math.PI * root.progress)
                    ctx.stroke()
                }
                Connections {
                    target: root
                    function onConfirmationsChanged() { parent.requestPaint() }
                    function onCategoryChanged() { parent.requestPaint() }
                }
                Component.onCompleted: requestPaint()
            }
            Text {
                visible: root.complete
                anchors.centerIn: parent
                text: "\u2713"
                color: Theme.success
                font.pixelSize: 12
                font.weight: Font.DemiBold
            }
        }

        ColumnLayout {
            spacing: 0
            Text {
                text: root.confirmations + "/" + root.required
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
                horizontalAlignment: Text.AlignRight
                Layout.alignment: Qt.AlignRight
            }
            Text {
                visible: root.remaining > 0
                text: "\u2212" + root.remaining + qsTr(" left")
                color: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 10
                horizontalAlignment: Text.AlignRight
                Layout.alignment: Qt.AlignRight
            }
        }
    }
}
