import QtQuick
import com.vericonomy.verium

// Pulsing placeholder block (Tailwind animate-pulse).
Rectangle {
    radius: Theme.radiusSm
    color: Theme.bgSubtle

    SequentialAnimation on opacity {
        loops: Animation.Infinite
        running: visible
        NumberAnimation { from: 0.4; to: 0.85; duration: 800; easing.type: Easing.InOutQuad }
        NumberAnimation { from: 0.85; to: 0.4; duration: 800; easing.type: Easing.InOutQuad }
    }
}
