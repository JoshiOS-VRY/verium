import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningControlsCard — thread + behavior configuration.
Card {
    id: card
    property bool autoAdjust: true
    property int manualThreads: 2
    property int suggestedThreads: 2
    property int maxThreads: 8
    property int displayThreads: 2
    property bool isMining: false
    property bool controlsDisabled: false
    property bool autoMineOnOpen: false
    property bool playSoundOnBlock: false
    property string rewardMode: "dynamic"
    property string rewardAddress: ""

    signal autoAdjustToggled(bool checked)
    signal threadsEdited(int threads)
    signal autoMineOnOpenToggled(bool checked)
    signal playSoundToggled(bool checked)
    signal rewardModePicked(string nextMode)
    signal rewardAddressEdited(string address)

    function selectRewardMode(nextMode) {
        card.rewardModePicked(nextMode)
    }

    padding: 20

    ColumnLayout {
        spacing: 16
        Layout.fillWidth: true

        ColumnLayout {
            spacing: 4
            Text {
                text: qsTr("Configuration")
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.weight: Font.DemiBold
            }
            Text {
                text: qsTr("Thread count and reward destination. Stop the miner before changing threads or address mode.")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }
        }

        MiningThreadControls {
            Layout.fillWidth: true
            autoAdjust: card.autoAdjust
            manualThreads: card.manualThreads
            suggestedThreads: card.suggestedThreads
            maxThreads: card.maxThreads
            activeThreads: card.displayThreads
            isMining: card.isMining
            disabled: card.controlsDisabled
            onAutoAdjustToggled: (v) => card.autoAdjustToggled(v)
            onThreadsEdited: (n) => card.threadsEdited(n)
        }

        Rectangle {
            Layout.fillWidth: true
            radius: Theme.radiusMd
            color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.30)
            border.color: Theme.border
            implicitHeight: advCol.implicitHeight + 24

            ColumnLayout {
                id: advCol
                anchors.fill: parent
                anchors.margins: 12
                spacing: 12

                Text {
                    text: qsTr("Mining configuration")
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 13
                    font.weight: Font.Medium
                }

                Text {
                    text: qsTr("BEHAVIOR")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.letterSpacing: 0.6
                }

                Repeater {
                    model: [
                        {
                            title: qsTr("Auto-mine on open"),
                            desc: qsTr("Start the CPU miner when you open the wallet (unless you stopped it manually)."),
                            checked: card.autoMineOnOpen,
                            key: "autoMine"
                        },
                        {
                            title: qsTr("Block mined sound"),
                            desc: qsTr("Play a short chime when a block you mined is accepted."),
                            checked: card.playSoundOnBlock,
                            key: "sound"
                        }
                    ]
                    delegate: Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        radius: Theme.radiusMd
                        color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.50)
                        border.color: Theme.border
                        implicitHeight: row.implicitHeight + 20
                        RowLayout {
                            id: row
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 10
                            CheckBox {
                                checked: modelData.checked
                                enabled: !card.controlsDisabled || modelData.key === "sound"
                                onToggled: {
                                    if (modelData.key === "autoMine")
                                        card.autoMineOnOpenToggled(checked)
                                    else
                                        card.playSoundToggled(checked)
                                }
                            }
                            ColumnLayout {
                                spacing: 2
                                Layout.fillWidth: true
                                Text {
                                    text: modelData.title
                                    color: Theme.fg
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.Medium
                                }
                                Text {
                                    text: modelData.desc
                                    color: Theme.fgMuted
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 11
                                    wrapMode: Text.Wrap
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }

                Text {
                    text: qsTr("REWARD ADDRESS")
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.letterSpacing: 0.6
                }

                RowLayout {
                    spacing: 8
                    Repeater {
                        model: [
                            { id: "dynamic", label: qsTr("Dynamic") },
                            { id: "static", label: qsTr("Static") }
                        ]
                        delegate: AppButton {
                            required property var modelData
                            text: modelData.label
                            size: "sm"
                            variant: card.rewardMode === modelData.id ? "primary" : "secondary"
                            enabled: !card.controlsDisabled
                            onClicked: card.selectRewardMode(modelData.id)
                        }
                    }
                }

                TextField {
                    visible: card.rewardMode === "static"
                    Layout.fillWidth: true
                    text: card.rewardAddress
                    enabled: !card.controlsDisabled
                    placeholderText: qsTr("Static mining reward address")
                    color: Theme.fg
                    placeholderTextColor: Theme.fgSubtle
                    font.family: Theme.monoFamily
                    font.pixelSize: 12
                    padding: 10
                    background: Rectangle {
                        radius: Theme.radiusMd
                        color: Theme.bgPanel
                        border.color: parent.activeFocus ? Theme.accent : Theme.border
                    }
                    onEditingFinished: card.rewardAddressEdited(text.trim())
                }
            }
        }
    }
}
