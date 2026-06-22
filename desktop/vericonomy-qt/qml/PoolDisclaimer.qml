import QtQuick
import com.vericonomy.verium

// Tauri PoolDisclaimer
Rectangle {
    width: parent ? parent.width : implicitWidth
    implicitHeight: txt.implicitHeight + 24
    radius: Theme.radiusMd
    color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.50)
    border.color: Theme.border
    border.width: 1

    Text {
        id: txt
        anchors.fill: parent
        anchors.margins: 12
        wrapMode: Text.Wrap
        text: qsTr("This pool is provided as-is, without warranties. Mining, payouts, and displayed stats may be delayed, incorrect, or interrupted. You use the pool at your own risk; the operator is not liable for lost rewards, downtime, software errors, network issues, or any other damages.")
        color: Theme.fgMuted
        font.family: Theme.fontFamily
        font.pixelSize: 11
        lineHeight: 1.45
    }
}
