import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningThreadControls (simplified).
ColumnLayout {
    id: ctrl
    property bool autoAdjust: true
    property int manualThreads: 2
    property int suggestedThreads: 2
    property int maxThreads: 8
    property int activeThreads: 0
    property bool isMining: false
    property bool disabled: false
    property bool compact: false

    signal autoAdjustToggled(bool checked)
    signal threadsEdited(int threads)

    spacing: compact ? 8 : 12

    Rectangle {
        Layout.fillWidth: true
        visible: !compact
        height: 1
        color: "transparent"
    }

    Rectangle {
        Layout.fillWidth: true
        visible: !compact
        radius: Theme.radiusMd
        color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.35)
        border.color: Theme.border
        border.width: 1
        implicitHeight: inner.implicitHeight + 24

        ColumnLayout {
            id: inner
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: 12
            spacing: 10

            RowLayout {
                spacing: 8
                CheckBox {
                    checked: ctrl.autoAdjust
                    enabled: !ctrl.disabled
                    onToggled: ctrl.autoAdjustToggled(checked)
                }
                Text {
                    text: qsTr("Auto-adjust threads for this device")
                    color: ctrl.compact ? Theme.fgMuted : Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: ctrl.compact ? 11 : 13
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                }
            }

            Text {
                visible: ctrl.autoAdjust
                text: qsTr("Using") + " " + String(ctrl.effectiveThreads) + " "
                    + (ctrl.effectiveThreads === 1 ? qsTr("thread") : qsTr("threads"))
                    + (ctrl.isMining && ctrl.activeThreads > 0 ? qsTr(" (active)") : "")
                color: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 12
                Layout.fillWidth: true
                wrapMode: Text.Wrap
            }

            ColumnLayout {
                visible: !ctrl.autoAdjust
                spacing: 6
                Layout.fillWidth: true
                Text {
                    text: qsTr("Threads")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                }
                SpinBox {
                    from: 1
                    to: Math.max(1, ctrl.maxThreads)
                    value: ctrl.manualThreads
                    enabled: !ctrl.disabled
                    onValueModified: ctrl.threadsEdited(value)
                }
            }
        }
    }

    // Compact mode (no bordered box)
    ColumnLayout {
        visible: compact
        spacing: 8
        Layout.fillWidth: true
        RowLayout {
            spacing: 8
            CheckBox {
                checked: ctrl.autoAdjust
                enabled: !ctrl.disabled
                onToggled: ctrl.autoAdjustToggled(checked)
            }
            Text {
                text: qsTr("Auto-adjust threads for this device")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 11
                Layout.fillWidth: true
            }
        }
        SpinBox {
            visible: !ctrl.autoAdjust
            from: 1
            to: Math.max(1, ctrl.maxThreads)
            value: ctrl.manualThreads
            enabled: !ctrl.disabled
            onValueModified: ctrl.threadsEdited(value)
        }
    }

    readonly property int effectiveThreads: {
        if (autoAdjust) {
            if (isMining && activeThreads > 0)
                return activeThreads
            return Math.min(suggestedThreads, maxThreads)
        }
        return Math.max(1, Math.min(manualThreads, maxThreads))
    }
}
