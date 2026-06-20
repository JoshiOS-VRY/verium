import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string coin: "vericoin"
    property var staking

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
                    title: qsTr("Staking")
                    subtitle: qsTr("Proof-of-Stake-Time (Vericoin)")
                    Layout.fillWidth: true
                    StatusPill {
                        loading: page.staking && page.staking.loading
                        tone: page.staking && page.staking.enabled ? "success" : "accent"
                        text: page.staking && page.staking.enabled ? qsTr("Active") : qsTr("Inactive")
                    }
                }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 24
                    rowSpacing: 14
                    StatTile {
                        label: qsTr("Stake weight")
                        value: page.staking
                            ? page.staking.stakeWeight.toLocaleString(Qt.locale(), 'f', 4)
                            : "—"
                    }
                    StatTile {
                        label: qsTr("Expected time")
                        value: page.staking
                            ? Math.round(page.staking.expectedTime / 3600) + " h"
                            : "—"
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    AppButton {
                        text: qsTr("Start staking")
                        enabled: page.staking && !page.staking.enabled
                        onClicked: if (page.staking) page.staking.startStaking()
                    }
                    AppButton {
                        text: qsTr("Stop staking")
                        variant: "secondary"
                        enabled: page.staking && page.staking.enabled
                        onClicked: if (page.staking) page.staking.stopStaking()
                    }
                }

                Text {
                    visible: page.staking && page.staking.lastMessage.length > 0
                    text: page.staking ? page.staking.lastMessage : ""
                    color: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
