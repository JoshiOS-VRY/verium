import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri PoolStatsStrip — public pool stats card (placeholder until pool API wired).
Card {
    id: strip
    property bool loading: false

    padding: 20

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 16

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: qsTr("Public Verium Mining Pool")
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
            AppButton {
                text: qsTr("Mining Pool Dashboard")
                variant: "secondary"
                size: "sm"
                onClicked: HostLinks.open("https://pool.vericonomy.com")
            }
        }

        Text {
            text: qsTr("Pool stats unavailable. Live pool metrics will appear here when the pool stats API is connected.")
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 13
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 5
            columnSpacing: 12
            rowSpacing: 12
            Repeater {
                model: [
                    qsTr("Pool hashrate"),
                    qsTr("Active miners"),
                    qsTr("Active workers"),
                    qsTr("Blocks found"),
                    qsTr("Pool fee")
                ]
                delegate: ColumnLayout {
                    required property int index
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        text: modelData.toUpperCase()
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.letterSpacing: 0.5
                    }
                    Text {
                        text: "—"
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                }
            }
        }
    }
}
