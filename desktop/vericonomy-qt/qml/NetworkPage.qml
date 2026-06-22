import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var node
    property var network
    property var bootstrap
    property var parseJson
    readonly property var peers: network ? parseJson(network.peersJson, []) : []

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            padding: 20

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 14
                SectionHeader { title: qsTr("Network"); Layout.fillWidth: true }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 4
                    columnSpacing: 24
                    rowSpacing: 14
                    StatTile { label: qsTr("Peers"); value: page.node ? page.node.connections.toString() : "—" }
                    StatTile { label: qsTr("Blocks"); value: page.node ? page.node.blocks.toLocaleString(Qt.locale()) : "—" }
                    StatTile { label: qsTr("Headers"); value: page.node ? page.node.headers.toLocaleString(Qt.locale()) : "—" }
                    StatTile {
                        label: qsTr("Status")
                        value: page.node ? (page.node.connected ? page.node.stateLabel : "Offline") : "—"
                        valueColor: page.node && page.node.connected ? Theme.success : Theme.fgMuted
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    AppButton {
                        text: qsTr("Start node")
                        enabled: page.node && !page.node.daemonRunning
                        onClicked: if (page.node) page.node.startDaemon()
                    }
                    AppButton {
                        text: qsTr("Stop node")
                        variant: "secondary"
                        enabled: page.node
                        onClicked: if (page.node) page.node.stopDaemon()
                    }
                    AppButton {
                        text: qsTr("Restart")
                        variant: "secondary"
                        enabled: page.node
                        onClicked: if (page.node) page.node.restartDaemon()
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: page.node && page.node.daemonMessage.length > 0
                        text: page.node ? page.node.daemonMessage : ""
                        color: Theme.fgSubtle
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                    }
                }
            }
        }

        Card {
            Layout.fillWidth: true
            padding: 20
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 10
                SectionHeader { title: qsTr("Chain bootstrap"); Layout.fillWidth: true }
                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    text: qsTr("Download the official chain snapshot to sync quickly instead of full block download from the network.")
                }
                AppButton {
                    text: qsTr("Import bootstrap…")
                    enabled: page.bootstrap
                    onClicked: bootstrapDialog.open()
                }
            }
        }

        Card {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 8

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                SectionHeader {
                    title: qsTr("Peers")
                    subtitle: qsTr("getpeerinfo")
                    Layout.fillWidth: true
                    Layout.margins: 14
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.peers
                    delegate: Item {
                        required property var modelData
                        width: ListView.view.width
                        height: 52
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16
                            spacing: 12
                            Badge {
                                text: modelData.inbound ? qsTr("In") : qsTr("Out")
                                tone: modelData.inbound ? "accent" : "neutral"
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Text {
                                    text: modelData.addr || "—"
                                    color: Theme.fg
                                    font.family: Theme.monoFamily
                                    font.pixelSize: 12
                                }
                                Text {
                                    text: modelData.subver || ""
                                    color: Theme.fgSubtle
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 11
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }
                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width; height: 1
                            color: Theme.border; opacity: 0.4
                        }
                    }
                }
            }
        }
    }

    BootstrapDialog {
        id: bootstrapDialog
        coin: page.bootstrap ? page.bootstrap.coin : "verium"
        bootstrap: page.bootstrap
        parseJson: page.parseJson
        node: page.node
    }
}
