import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string coin: "verium"
    property var mining

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            padding: 24

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14

                SectionHeader {
                    title: qsTr("Mining")
                    subtitle: qsTr("scrypt² CPU mining via node RPC")
                    Layout.fillWidth: true
                    StatusPill {
                        loading: page.mining && page.mining.loading
                        tone: page.mining && page.mining.minerActive ? "success" : "accent"
                        text: page.mining && page.mining.minerActive
                            ? qsTr("Mining") : qsTr("Stopped")
                    }
                }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 3
                    columnSpacing: 24
                    rowSpacing: 14
                    StatTile {
                        label: qsTr("Network hashrate")
                        value: page.mining
                            ? (page.mining.networkHashps / 1000).toLocaleString(Qt.locale(), 'f', 2) + " kH/s"
                            : "—"
                    }
                    StatTile {
                        label: qsTr("Difficulty")
                        value: page.mining
                            ? page.mining.difficulty.toLocaleString(Qt.locale(), 'f', 4)
                            : "—"
                    }
                    StatTile {
                        label: qsTr("Threads")
                        value: page.mining ? String(page.mining.threads) : "—"
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    SpinBox {
                        id: threadSpin
                        from: 1
                        to: 32
                        value: page.mining ? page.mining.threads : 2
                    }
                    AppButton {
                        text: qsTr("Start")
                        enabled: page.mining && !page.mining.minerActive
                        onClicked: if (page.mining) page.mining.startMiner(threadSpin.value)
                    }
                    AppButton {
                        text: qsTr("Stop")
                        variant: "secondary"
                        enabled: page.mining && page.mining.minerActive
                        onClicked: if (page.mining) page.mining.stopMiner()
                    }
                }

                Text {
                    visible: page.mining && page.mining.lastMessage.length > 0
                    text: page.mining ? page.mining.lastMessage : ""
                    color: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
