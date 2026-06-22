import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningHero — solo CPU miner hero card with start/stop CTA.
Card {
    id: hero
    property bool active: false
    property real localHashrate: 0
    property int displayThreads: 2
    property bool chainSynced: true
    property bool syncStalled: false
    property bool staticAddressMissing: false
    property int blocksBehind: 0
    property bool startPending: false
    property bool stopPending: false
    property string errorText: ""

    signal startRequested()
    signal stopRequested()

    readonly property bool live: active
    readonly property bool canStart: chainSynced && !syncStalled && !staticAddressMissing

    padding: 20
    color: live
        ? Qt.rgba(Theme.bgPanel.r, Theme.bgPanel.g, Theme.bgPanel.b, 1)
        : Theme.bgPanel
    border.color: live ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.25) : Theme.border

    function blockerText() {
        if (syncStalled)
            return qsTr("Sync is stalled.")
        if (!chainSynced) {
            var t = qsTr("Wait for the node to reach the network tip")
            if (blocksBehind > 0)
                t += " (~" + blocksBehind.toLocaleString(Qt.locale(), 'f', 0) + qsTr(" blocks behind)")
            return t + "."
        }
        if (staticAddressMissing)
            return qsTr("Choose a reward address in Advanced settings (static mode).")
        return ""
    }

    ColumnLayout {
        spacing: 16
        Layout.fillWidth: true

        RowLayout {
            Layout.fillWidth: true
            spacing: 16

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12

                RowLayout {
                    spacing: 8
                    NavIcon { name: "cpu"; tint: Theme.accent }
                    Text {
                        text: qsTr("CPU miner")
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 17
                        font.weight: Font.DemiBold
                    }
                    Badge {
                        tone: live ? "success" : "neutral"
                        text: live ? qsTr("Running") : qsTr("Stopped")
                    }
                }

                ColumnLayout {
                    visible: live
                    spacing: 4
                    Text {
                        text: qsTr("LIVE HASHRATE")
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.weight: Font.DemiBold
                        font.letterSpacing: 0.6
                    }
                    RowLayout {
                        spacing: 6
                        Text {
                            text: localHashrate > 0
                                ? localHashrate.toLocaleString(Qt.locale(), 'f', 2)
                                : "—"
                            color: Theme.fg
                            font.family: Theme.fontFamily
                            font.pixelSize: 32
                            font.weight: Font.DemiBold
                        }
                        Text {
                            text: "H/m"
                            color: Theme.fgSubtle
                            font.family: Theme.fontFamily
                            font.pixelSize: 16
                            Layout.alignment: Qt.AlignBottom
                            Layout.bottomMargin: 4
                        }
                    }
                    RowLayout {
                        spacing: 16
                        Text {
                            text: qsTr("Threads") + " "
                                + String(displayThreads)
                            color: Theme.fg
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                        }
                    }
                }

                Rectangle {
                    visible: !live
                    Layout.fillWidth: true
                    radius: Theme.radiusMd
                    color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.40)
                    border.color: Theme.border
                    border.width: 1
                    implicitHeight: idleCol.implicitHeight + 24
                    ColumnLayout {
                        id: idleCol
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8
                        Text {
                            text: qsTr("Solo CPU mining on this wallet. Configure threads below, then start when your node is synced.")
                            color: Theme.fgMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                        Text {
                            visible: hero.blockerText().length > 0
                            text: hero.blockerText()
                            color: Theme.warning
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }
                    }
                }
            }

            ColumnLayout {
                spacing: 8
                AppButton {
                    visible: live
                    text: stopPending ? qsTr("Stopping…") : qsTr("Stop mining")
                    variant: "danger"
                    size: "lg"
                    enabled: !stopPending
                    onClicked: hero.stopRequested()
                }
                AppButton {
                    visible: !live
                    text: startPending ? qsTr("Starting…") : qsTr("Start mining")
                    size: "lg"
                    enabled: canStart && !startPending
                    onClicked: hero.startRequested()
                }
                Text {
                    visible: !live && hero.blockerText().length > 0
                    text: hero.blockerText()
                    color: Theme.warning
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    horizontalAlignment: Text.AlignHCenter
                    Layout.preferredWidth: 168
                }
            }
        }

        Text {
            visible: errorText.length > 0
            text: errorText
            color: Theme.danger
            font.family: Theme.fontFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        RowLayout {
            visible: !live
            spacing: 8
            NavIcon { name: "cpu"; tint: Theme.fgMuted; width: 14; height: 14 }
            Text {
                text: qsTr("Built-in Verium CPU miner — rewards pay to your wallet per reward address settings.")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 11
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }
        }
    }
}
