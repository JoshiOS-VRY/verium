import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Resources / links. Mirrors desktop/verium-app/src/pages/Resources.tsx.
// External links open via the Qt HostBridge (QDesktopServices), not a WebView.
Item {
    id: page
    property var links: [
        { label: "Vericonomy website",  url: "https://vericonomy.com" },
        { label: "Block explorer",      url: "https://explorer.vericoin.info" },
        { label: "Documentation",       url: "https://github.com/vericonomy" },
        { label: "Community / Discord",  url: "https://discord.gg/vericonomy" }
    ]

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 12
        SectionHeader { title: qsTr("Resources"); Layout.fillWidth: true }

        Card {
            Layout.fillWidth: true
            padding: 8
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0
                Repeater {
                    model: page.links
                    delegate: Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        implicitHeight: 48
                        color: hov.hovered ? Theme.bgSubtle : "transparent"
                        radius: Theme.radiusMd
                        HoverHandler { id: hov }
                        TapHandler {
                            onTapped: HostLinks.open(modelData.url)
                        }
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            Text {
                                text: modelData.label
                                color: Theme.fg
                                font.family: Theme.fontFamily
                                font.pixelSize: 13
                                Layout.fillWidth: true
                            }
                            Text { text: "\u2197"; color: Theme.fgSubtle; font.pixelSize: 14 }
                        }
                    }
                }
            }
        }
        Item { Layout.fillHeight: true }
    }
}
