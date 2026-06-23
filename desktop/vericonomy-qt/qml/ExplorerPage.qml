import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property string coin: "verium"
    property var explorer
    property var parseJson: function(s, fb) { return fb || [] }

    readonly property var stats: page.explorer
        ? page.parseJson(page.explorer.statsJson, {})
        : {}
    readonly property var blocks: page.explorer
        ? page.parseJson(page.explorer.blocksJson, [])
        : []

    Component.onCompleted: if (page.explorer) page.explorer.refresh()
    onCoinChanged: if (page.explorer) page.explorer.refresh()

    function fmtNum(n, d) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        return Number(n).toLocaleString(Qt.locale(), 'f', d !== undefined ? d : 2)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Card {
            Layout.fillWidth: true
            padding: 20
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12
                SectionHeader {
                    title: qsTr("Network stats")
                    subtitle: qsTr("Indexer-backed explorer API")
                    Layout.fillWidth: true
                    AppButton {
                        text: qsTr("Refresh")
                        variant: "ghost"
                        onClicked: if (page.explorer) page.explorer.refresh()
                    }
                }
                GridLayout {
                    columns: 4
                    columnSpacing: 20
                    rowSpacing: 10
                    Layout.fillWidth: true
                    StatTile {
                        label: qsTr("Height")
                        value: page.stats.height !== undefined ? String(page.stats.height) : "—"
                    }
                    StatTile {
                        label: qsTr("Difficulty")
                        value: page.fmtNum(page.stats.difficulty, 2)
                    }
                    StatTile {
                        label: qsTr("Supply")
                        value: page.fmtNum(page.stats.supply, 0)
                    }
                    StatTile {
                        label: qsTr("Price USD")
                        value: page.stats.price_usd != null
                            ? "$" + page.fmtNum(page.stats.price_usd, 4) : "—"
                    }
                }
            }
        }

        Card {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 20
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                SectionHeader { title: qsTr("Recent blocks"); Layout.fillWidth: true }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: 6
                    model: page.blocks
                    delegate: Rectangle {
                        required property var modelData
                        width: ListView.view.width
                        height: 52
                        radius: Theme.radiusSm
                        color: Theme.bgSubtle
                        border.color: Theme.border
                        border.width: 1

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                var chain = page.coin === "vericoin" ? "vrc" : "vrm"
                                var id = modelData.hash || modelData.height
                                if (id !== undefined)
                                    HostLinks.open("https://explorer.vericonomy.com/" + chain + "/block/" + encodeURIComponent(String(id)))
                            }
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 12
                            Text {
                                text: "#" + modelData.height
                                color: Theme.fg
                                font.family: Theme.monoFamily
                                font.pixelSize: 12
                                font.weight: Font.DemiBold
                            }
                            Text {
                                text: modelData.hash ? modelData.hash.slice(0, 16) + "…" : ""
                                color: Theme.fgMuted
                                font.family: Theme.monoFamily
                                font.pixelSize: 11
                                Layout.fillWidth: true
                                elide: Text.ElideRight
                            }
                            Text {
                                text: modelData.n_tx !== undefined ? modelData.n_tx + " tx" : ""
                                color: Theme.fgSubtle
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                            }
                        }
                    }
                }

                Text {
                    visible: page.explorer && page.explorer.lastMessage.length > 0
                    text: page.explorer ? page.explorer.lastMessage : ""
                    color: Theme.fgSubtle
                    font.pixelSize: 11
                    Layout.fillWidth: true
                }
            }
        }
    }
}
