import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningModeToggle — solo/pool segmented control.
Rectangle {
    id: toggle
    property string mode: "pool"
    signal modeSelected(string mode)

    implicitHeight: 40
    implicitWidth: row.implicitWidth + 8
    radius: Theme.radiusMd
    color: Theme.bgSubtle
    border.color: Theme.border
    border.width: 1

    Row {
        id: row
        anchors.centerIn: parent
        spacing: 0

        Repeater {
            model: [
                { id: "solo", label: qsTr("Solo Mining") },
                { id: "pool", label: qsTr("Pool Mining") }
            ]
            delegate: AppButton {
                required property var modelData
                text: modelData.label
                size: "sm"
                variant: toggle.mode === modelData.id ? "primary" : "ghost"
                onClicked: toggle.modeSelected(modelData.id)
            }
        }
    }
}
