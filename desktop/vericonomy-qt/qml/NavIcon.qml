import QtQuick
import QtQuick.Effects
import com.vericonomy.verium

// Lucide nav icon (16×16) with theme tint.
// SVG assets use white strokes (#FFFFFF); MultiEffect colorization maps luminance → tint.
Item {
    id: icon
    property string name: "gauge"
    property color tint: Theme.fgMuted

    width: 16
    height: 16

    Image {
        id: src
        anchors.fill: parent
        source: "qrc:/qt/qml/com/vericonomy/verium/assets/icons/nav/" + icon.name + ".svg"
        sourceSize: Qt.size(24, 24)
        fillMode: Image.PreserveAspectFit
        smooth: true
        visible: false
    }

    MultiEffect {
        anchors.fill: parent
        source: src
        saturation: 0
        colorization: 1.0
        colorizationColor: icon.tint
    }
}
