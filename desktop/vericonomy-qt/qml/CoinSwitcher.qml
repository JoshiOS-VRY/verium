import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri CoinSwitcher parity — logo, name, symbol badge, tagline, dropdown.
Item {
    id: switcher
    property string coin: "verium"
    signal coinSelected(string coinId)

    readonly property var coins: [
        {
            id: "verium",
            name: qsTr("Verium"),
            symbol: "VRM",
            tagline: qsTr("Reserve"),
            logo: "qrc:/qt/qml/com/vericonomy/verium/assets/verium-logo.svg",
            accentBg: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.15),
            accentFg: Theme.accent,
            accentBorder: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.30)
        },
        {
            id: "vericoin",
            name: qsTr("Vericoin"),
            symbol: "VRC",
            tagline: qsTr("Currency"),
            logo: "qrc:/qt/qml/com/vericonomy/verium/assets/vericoin-logo.svg",
            accentBg: Theme.bgPanel,
            accentFg: Theme.fgMuted,
            accentBorder: Theme.borderStrong
        }
    ]

    readonly property var active: {
        for (var i = 0; i < coins.length; i++)
            if (coins[i].id === switcher.coin) return coins[i]
        return coins[0]
    }

    implicitWidth: 208
    implicitHeight: 52
    height: implicitHeight

    function selectCoin(coinId) {
        switcher.coinSelected(coinId)
        popup.close()
    }

    Rectangle {
        id: trigger
        anchors.left: parent.left
        anchors.right: parent.right
        height: 52
        radius: Theme.radiusMd
        color: triggerArea.containsMouse || popup.visible
            ? Qt.rgba(Theme.bgPanel.r, Theme.bgPanel.g, Theme.bgPanel.b, 0.6)
            : "transparent"
        border.color: triggerArea.containsMouse || popup.visible ? Theme.border : "transparent"
        border.width: 1

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 4
            anchors.rightMargin: 6
            spacing: 10

            Item {
                Layout.preferredWidth: 36
                Layout.preferredHeight: 36
                Rectangle {
                    anchors.fill: parent
                    radius: 8
                    color: "transparent"
                    clip: true
                    Image {
                        source: switcher.active.logo
                        sourceSize.width: 72
                        sourceSize.height: 72
                        anchors.fill: parent
                        fillMode: Image.PreserveAspectFit
                        smooth: true
                        antialiasing: true
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 1
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        text: switcher.active.name
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }
                    Rectangle {
                        radius: 4
                        color: switcher.active.accentBg
                        border.color: switcher.active.accentBorder
                        border.width: 1
                        implicitHeight: symLabel.implicitHeight + 4
                        implicitWidth: symLabel.implicitWidth + 10
                        Text {
                            id: symLabel
                            anchors.centerIn: parent
                            text: switcher.active.symbol
                            color: switcher.active.accentFg
                            font.family: Theme.fontFamily
                            font.pixelSize: 10
                            font.weight: Font.DemiBold
                            font.letterSpacing: 0.6
                        }
                    }
                }
                Text {
                    text: switcher.active.tagline
                    color: Theme.fgSubtle
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                }
            }

            Text {
                text: popup.visible ? "\u25B2" : "\u25BC"
                color: Theme.fgSubtle
                font.pixelSize: 10
            }
        }

        MouseArea {
            id: triggerArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.visible ? popup.close() : popup.open()
        }
    }

    Popup {
        id: popup
        x: 0
        y: trigger.height + 6
        width: switcher.width
        padding: 0
        modal: true
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            radius: Theme.radiusMd
            color: Theme.bgPanel
            border.color: Theme.border
            border.width: 1
        }

        contentItem: Column {
            width: parent.width
            spacing: 0

            Repeater {
                model: switcher.coins
                delegate: Rectangle {
                    required property var modelData
                    width: popup.width
                    height: 58
                    color: modelData.id === switcher.coin
                        ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.10)
                        : (rowArea.containsMouse ? Theme.bgSubtle : "transparent")

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        spacing: 10

                        Item {
                            Layout.preferredWidth: 28
                            Layout.preferredHeight: 28
                            Layout.topMargin: 2
                            Rectangle {
                                anchors.fill: parent
                                radius: 6
                                color: "transparent"
                                clip: true
                                Image {
                                    source: modelData.logo
                                    sourceSize.width: 56
                                    sourceSize.height: 56
                                    anchors.fill: parent
                                    fillMode: Image.PreserveAspectFit
                                    smooth: true
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 1
                            RowLayout {
                                spacing: 6
                                Text {
                                    text: modelData.name
                                    color: Theme.fg
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.DemiBold
                                }
                                Rectangle {
                                    radius: 4
                                    color: modelData.accentBg
                                    border.color: modelData.accentBorder
                                    border.width: 1
                                    implicitHeight: optSym.implicitHeight + 4
                                    implicitWidth: optSym.implicitWidth + 10
                                    Text {
                                        id: optSym
                                        anchors.centerIn: parent
                                        text: modelData.symbol
                                        color: modelData.accentFg
                                        font.family: Theme.fontFamily
                                        font.pixelSize: 10
                                        font.weight: Font.DemiBold
                                        font.letterSpacing: 0.6
                                    }
                                }
                            }
                            Text {
                                text: modelData.tagline
                                color: Theme.fgSubtle
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                            }
                        }

                        Text {
                            visible: modelData.id === switcher.coin
                            text: "\u2713"
                            color: Theme.accent
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                        }
                    }

                    MouseArea {
                        id: rowArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: switcher.selectCoin(modelData.id)
                    }
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.border
            }

            Text {
                width: parent.width - 24
                x: 12
                y: 8
                wrapMode: Text.Wrap
                text: qsTr("Switch between Verium and Vericoin wallets")
                color: Theme.fgSubtle
                font.family: Theme.fontFamily
                font.pixelSize: 10
            }
            Item { width: 1; height: 10 }
        }
    }
}
