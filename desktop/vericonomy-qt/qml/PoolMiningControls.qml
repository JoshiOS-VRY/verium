import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri PoolMiningControls — mine on the public pool card.
Card {
    id: panel
    property string payoutAddress: ""
    property string workerName: "wallet"
    property bool running: false
    property bool sidecarFound: false
    property bool nodeConnected: false
    property bool chainSynced: true
    property bool syncStalled: false
    property real hashrateHm: 0
    property string connectionState: ""
    property string lastMessage: ""
    property bool autoAdjust: true
    property int manualThreads: 2
    property int suggestedThreads: 2
    property int maxThreads: 8
    property int activeThreads: 0
    property bool controlsDisabled: false

    signal payoutChanged(string address)
    signal workerChanged(string name)
    signal autoAdjustToggled(bool checked)
    signal threadsEdited(int threads)
    signal startRequested()
    signal stopRequested()

    readonly property string poolUsername: {
        var addr = payoutAddress.trim()
        var worker = workerName.trim().length > 0 ? workerName.trim() : "wallet"
        return addr.length > 0 ? addr + "." + worker : ""
    }
    readonly property bool canStart: sidecarFound && nodeConnected && payoutAddress.trim().length > 0
        && !running && !controlsDisabled

    padding: 20
    border.color: running ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.25) : Theme.border

    ColumnLayout {
        spacing: 16
        Layout.fillWidth: true

        RowLayout {
            spacing: 8
            NavIcon { name: "cpu"; tint: Theme.accent }
            Text {
                text: qsTr("Mine on the public pool")
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
            Badge {
                tone: running ? "success" : "neutral"
                text: running ? qsTr("Mining") : qsTr("Stopped")
            }
        }

        Text {
            visible: !sidecarFound && !running
            text: qsTr("Pool miner sidecar not detected. Check Network page or restart the node.")
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        ColumnLayout {
            spacing: 6
            Layout.fillWidth: true
            Text {
                text: qsTr("Payout address")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
            TextField {
                Layout.fillWidth: true
                text: panel.payoutAddress
                enabled: !running
                placeholderText: qsTr("VRM address")
                color: Theme.fg
                placeholderTextColor: Theme.fgSubtle
                font.family: Theme.monoFamily
                font.pixelSize: 13
                padding: 10
                background: Rectangle {
                    radius: Theme.radiusMd
                    color: Theme.bgPanel
                    border.color: parent.activeFocus ? Theme.accent : Theme.border
                }
                onEditingFinished: panel.payoutChanged(text.trim())
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: 12
            rowSpacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text {
                    text: qsTr("Worker name")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                }
                TextField {
                    Layout.fillWidth: true
                    text: panel.workerName
                    enabled: !running
                    placeholderText: qsTr("wallet")
                    color: Theme.fg
                    placeholderTextColor: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 13
                    padding: 10
                    background: Rectangle {
                        radius: Theme.radiusMd
                        color: Theme.bgPanel
                        border.color: parent.activeFocus ? Theme.accent : Theme.border
                    }
                    onEditingFinished: panel.workerChanged(text.trim())
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text {
                    text: qsTr("Pool username")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 40
                    radius: Theme.radiusMd
                    color: Theme.bgSubtle
                    border.color: Theme.border
                    Text {
                        anchors.fill: parent
                        anchors.margins: 10
                        verticalAlignment: Text.AlignVCenter
                        text: panel.poolUsername.length > 0 ? panel.poolUsername : "—"
                        color: Theme.fg
                        font.family: Theme.monoFamily
                        font.pixelSize: 11
                        elide: Text.ElideMiddle
                    }
                }
            }
        }

        MiningThreadControls {
            Layout.fillWidth: true
            autoAdjust: panel.autoAdjust
            manualThreads: panel.manualThreads
            suggestedThreads: panel.suggestedThreads
            maxThreads: panel.maxThreads
            activeThreads: panel.activeThreads
            isMining: panel.running
            disabled: panel.running
            onAutoAdjustToggled: (v) => panel.autoAdjustToggled(v)
            onThreadsEdited: (n) => panel.threadsEdited(n)
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 16

            ColumnLayout {
                spacing: 4
                Layout.fillWidth: true
                Text {
                    text: qsTr("LOCAL HASHRATE")
                    color: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.letterSpacing: 0.6
                }
                Text {
                    text: running
                        ? hashrateHm.toLocaleString(Qt.locale(), 'f', 2) + " H/m"
                        : "0.00 H/m"
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 24
                    font.weight: Font.DemiBold
                }
                Text {
                    visible: running && connectionState.length > 0
                    text: connectionState
                    color: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                }
            }

            AppButton {
                visible: running
                text: qsTr("Stop pool mining")
                variant: "danger"
                onClicked: panel.stopRequested()
            }
            AppButton {
                visible: !running
                text: qsTr("Start pool mining")
                enabled: canStart
                onClicked: panel.startRequested()
            }
        }

        Text {
            visible: lastMessage.length > 0
            text: lastMessage
            color: Theme.fgSubtle
            font.family: Theme.fontFamily
            font.pixelSize: 11
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        Text {
            visible: !chainSynced && !syncStalled && !running
            text: qsTr("Wait for the node to sync before starting the pool miner.")
            color: Theme.warning
            font.family: Theme.fontFamily
            font.pixelSize: 12
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
    }
}
