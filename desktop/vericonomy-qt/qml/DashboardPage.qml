import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var dashboard
    property var explorer
    property string coin: "verium"
    property var parseJson: function(s, fb) { try { return JSON.parse(s) } catch(e) { return fb || {} } }

    readonly property var snap: page.dashboard
        ? page.parseJson(page.dashboard.snapshotJson, {})
        : {}
    readonly property var blocks: page.explorer
        ? page.parseJson(page.explorer.blocksJson, [])
        : []

    Component.onCompleted: page.refreshAll()
    onCoinChanged: page.refreshAll()

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        ColumnLayout {
            width: page.width
            spacing: 20
            Item { Layout.preferredHeight: 8; Layout.fillWidth: true }

            DashboardHero {
                Layout.fillWidth: true
                Layout.preferredHeight: implicitHeight
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                coin: page.coin
                snap: page.snap
                loading: page.dashboard ? page.dashboard.loading : false
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 0
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.margins: 20
                        Layout.bottomMargin: 12
                        spacing: 12
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4
                            Text {
                                text: qsTr("RECENT BLOCKS")
                                color: Theme.fg
                                font.pixelSize: 13
                                font.weight: Font.DemiBold
                                font.letterSpacing: 0.6
                            }
                        }
                        AppButton {
                            text: qsTr("All blocks")
                            variant: "ghost"
                            size: "sm"
                        }
                    }
                    ExplorerRecentBlocks {
                        Layout.fillWidth: true
                        blocks: page.blocks
                        coin: page.coin
                    }
                    Item { Layout.preferredHeight: 8; Layout.fillWidth: true }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                visible: page.snap.is_light !== true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    SectionHeader { title: qsTr("Backup health"); Layout.fillWidth: true }
                    Text {
                        text: qsTr("Scheduled backups and wallet.dat exports — configure in Settings → Security.")
                        color: Theme.fgMuted
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }
            }

            Item { Layout.preferredHeight: 16; Layout.fillWidth: true }
        }
    }

    function refreshAll() {
        if (page.dashboard) page.dashboard.refresh()
        if (page.explorer) page.explorer.refresh()
    }
}
