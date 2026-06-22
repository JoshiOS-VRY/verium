import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri BootstrapProgressPanel parity.
Item {
    id: panel
    property var progress
    property string fallbackMessage: qsTr("Preparing bootstrap…")

    readonly property real percent: progress && progress.percent != null ? progress.percent : 0
    readonly property string phase: progress && progress.phase ? progress.phase : "starting"
    readonly property bool indeterminate: progress
        && progress.phasePercent == null
        && (progress.phase === "extracting" || progress.phase === "downloading")
    readonly property string message: progress && progress.message ? progress.message : fallbackMessage
    readonly property bool showSourceUrl: progress
        && typeof progress.sourceUrl === "string"
        && progress.sourceUrl.length > 0
        && progress.phase === "downloading"

    readonly property var phaseLabels: ({
        "stopping": qsTr("Stopping node"),
        "resolving": qsTr("Finding bootstrap"),
        "local": qsTr("Local archive"),
        "downloading": qsTr("Downloading"),
        "validating": qsTr("Validating"),
        "extracting": qsTr("Extracting"),
        "applying": qsTr("Installing"),
        "restarting": qsTr("Restarting"),
        "done": qsTr("Complete"),
        "cancelled": qsTr("Cancelled"),
        "error": qsTr("Failed")
    })

    implicitHeight: col.implicitHeight + 24

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusMd
        color: Theme.bgSubtle
        border.color: Theme.border
        border.width: 1
    }

    ColumnLayout {
        id: col
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text {
                text: phaseLabels[phase] || phase
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 12
                font.weight: Font.Medium
            }
            Text {
                visible: !indeterminate && progress
                text: Math.round(percent) + "%"
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
            Item { Layout.fillWidth: true }
        }

        Text {
            Layout.fillWidth: true
            text: message
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
        }

        Text {
            visible: showSourceUrl
            Layout.fillWidth: true
            text: showSourceUrl ? progress.sourceUrl : ""
            color: Theme.fgSubtle
            font.family: Theme.monoFamily
            font.pixelSize: 10
            elide: Text.ElideMiddle
        }

        Rectangle {
            id: progressTrack
            Layout.fillWidth: true
            height: 8
            radius: 4
            color: Theme.border
            Rectangle {
                visible: !indeterminate
                height: parent.height
                width: parent.width * Math.max(0, Math.min(1, percent / 100))
                radius: 4
                color: Theme.accent
            }
            Rectangle {
                id: indeterminateBar
                visible: indeterminate
                height: parent.height
                width: progressTrack.width > 0 ? progressTrack.width * 0.33 : 0
                radius: 4
                color: Theme.accent
                x: 0
                NumberAnimation on x {
                    running: indeterminate && progressTrack.width > indeterminateBar.width
                    from: 0
                    to: progressTrack.width - indeterminateBar.width
                    duration: 1200
                    loops: Animation.Infinite
                }
            }
        }
    }
}
