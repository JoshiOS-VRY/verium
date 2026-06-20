import QtQuick
import com.vericonomy.verium

// Lucide nav icon (16×16) with theme tint — no Qt5Compat dependency.
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

    ShaderEffect {
        anchors.fill: parent
        property variant source: src
        property color tintColor: icon.tint

        fragmentShader: "
            uniform lowp sampler2D source;
            uniform lowp vec4 tintColor;
            uniform lowp float qt_Opacity;
            varying highp vec2 qt_TexCoord0;
            void main() {
                lowp vec4 c = texture2D(source, qt_TexCoord0);
                lowp float a = max(c.a, max(c.r, max(c.g, c.b)));
                gl_FragColor = vec4(tintColor.rgb, a * tintColor.a * qt_Opacity);
            }
        "
    }
}
