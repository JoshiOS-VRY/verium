import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Desktop transaction history table with pagination (Tauri historySection parity).
Item {
    id: table
    property var transactions
    property var parseJson
    property string coin: "verium"
    property var poolPayoutTxids: []

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property int pageSize: 25
    readonly property int listCap: 500

    property int page: 0

    readonly property var allRows: transactions ? parseJson(transactions.rowsJson, []) : []
    readonly property int totalItems: allRows.length
    readonly property int totalPages: totalItems <= 0 ? 1 : Math.ceil(totalItems / pageSize)
    readonly property int effectivePage: Math.min(page, Math.max(0, totalPages - 1))
    readonly property int rangeFrom: totalItems === 0 ? 0 : effectivePage * pageSize + 1
    readonly property int rangeTo: Math.min(totalItems, (effectivePage + 1) * pageSize)
    readonly property var pageRows: allRows.slice(effectivePage * pageSize, rangeTo)

    readonly property bool isInitialLoading: transactions && transactions.loading && totalItems === 0
    readonly property bool isLoading: isInitialLoading
    readonly property bool isError: transactions && transactions.hasError
    readonly property bool showEmpty: !isLoading && !isError && totalItems === 0
    readonly property bool historyCapped: transactions && transactions.historyCapped

    function formatCoinAmount(amount) {
        if (!isFinite(amount)) return "0 " + ticker
        var abs = Math.abs(amount)
        var frac = abs >= 1 ? 4 : 8
        return amount.toFixed(frac) + " " + ticker
    }

    function formatNumber(n) {
        return n.toLocaleString(Qt.locale())
    }

    function isPoolPayout(txid) {
        return poolPayoutTxids.indexOf(txid) >= 0
    }

    onCoinChanged: page = 0

    implicitWidth: 640
    implicitHeight: col.implicitHeight

    ColumnLayout {
        id: col
        anchors.fill: parent
        spacing: 0

        // Card header
        ColumnLayout {
            Layout.fillWidth: true
            Layout.margins: 20
            spacing: 4
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                Text {
                    text: qsTr("Recent transactions")
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 16
                    font.weight: Font.DemiBold
                }
                BusyIndicator {
                    visible: table.isLoading
                    running: visible
                    implicitWidth: 16
                    implicitHeight: 16
                }
            }
            Text {
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
                text: {
                    if (table.isLoading) return qsTr("Loading your wallet transaction history…")
                    if (table.isError) return qsTr("Could not load transaction history from the wallet.")
                    if (table.showEmpty) return qsTr("Transactions you send or receive will appear here.")
                    return qsTr("Newest first.")
                }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }

        Item {
            id: body
            Layout.fillWidth: true
            Layout.preferredHeight: table.isLoading ? 280 : (table.showEmpty ? 220 : Math.min(480, headerRow.height + scroll.height))

            // Sticky-style header
            Item {
                id: headerRow
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                height: 32
                visible: !table.showEmpty
                z: 2

                Rectangle {
                    anchors.fill: parent
                    color: Theme.bgPanel
                    border.color: Theme.border
                    border.width: 0
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: Theme.border
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 0

                    Repeater {
                        model: [
                            { label: qsTr("When"), w: 0.16, align: Text.AlignLeft },
                            { label: qsTr("Type"), w: 0.18, align: Text.AlignLeft },
                            { label: qsTr("Address"), w: 0.26, align: Text.AlignLeft },
                            { label: qsTr("Amount"), w: 0.16, align: Text.AlignRight },
                            { label: qsTr("Confs"), w: 0.14, align: Text.AlignRight },
                            { label: qsTr("Explorer"), w: 0.10, align: Text.AlignRight }
                        ]
                        delegate: Text {
                            required property var modelData
                            Layout.preferredWidth: headerRow.width * modelData.w
                            text: modelData.label.toUpperCase()
                            color: Theme.fgSubtle
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            font.weight: Font.DemiBold
                            horizontalAlignment: modelData.align
                            leftPadding: 16
                            rightPadding: 16
                        }
                    }
                }
            }

            Flickable {
                id: scroll
                anchors.top: headerRow.bottom
                anchors.left: parent.left
                anchors.right: parent.right
                height: table.showEmpty ? 0 : Math.min(448, rowsCol.height)
                contentHeight: rowsCol.height
                clip: true
                boundsBehavior: Flickable.StopAtBounds

            Column {
                id: rowsCol
                width: scroll.width

                // Loading skeleton rows
                Repeater {
                    model: table.isLoading ? 6 : 0
                    delegate: Rectangle {
                        width: rowsCol.width
                        height: 36
                        color: index % 2 === 1 ? Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.3) : "transparent"
                        border.color: Theme.border
                        border.width: 0
                        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.border }
                        Skeleton {
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 32
                            height: 14
                        }
                    }
                }

                // Data rows
                Repeater {
                    model: !table.isLoading && !table.showEmpty ? table.pageRows : []
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: rowsCol.width
                        height: 44
                        color: index % 2 === 1 ? Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.3) : "transparent"
                        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.border }

                        RowLayout {
                            anchors.fill: parent
                            spacing: 0

                            Text {
                                Layout.preferredWidth: parent.width * 0.16
                                leftPadding: 16
                                text: modelData.time_display || "—"
                                color: Theme.fgMuted
                                font.family: Theme.fontFamily
                                font.pixelSize: 12
                                elide: Text.ElideRight
                            }

                            Item {
                                Layout.preferredWidth: parent.width * 0.18
                                Layout.fillHeight: true
                                RowLayout {
                                    anchors.left: parent.left
                                    anchors.leftMargin: 16
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: 6
                                    TransactionCategoryBadge { category: modelData.category }
                                    Badge {
                                        visible: table.coin === "verium" && table.isPoolPayout(modelData.txid)
                                        text: qsTr("Pool payout")
                                        tone: "neutral"
                                    }
                                }
                            }

                            Text {
                                Layout.preferredWidth: parent.width * 0.26
                                leftPadding: 16
                                text: modelData.address || "—"
                                color: Theme.fg
                                font.family: Theme.fontFamily
                                font.pixelSize: 12
                                elide: Text.ElideRight
                            }

                            Text {
                                Layout.preferredWidth: parent.width * 0.16
                                rightPadding: 16
                                horizontalAlignment: Text.AlignRight
                                text: table.formatCoinAmount(modelData.amount)
                                color: Theme.fg
                                font.family: Theme.fontFamily
                                font.pixelSize: 13
                            }

                            ConfirmationProgress {
                                Layout.preferredWidth: parent.width * 0.14
                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: 16
                                confirmations: modelData.confirmations
                                category: modelData.category
                            }

                            ExplorerTxLink {
                                Layout.preferredWidth: parent.width * 0.10
                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: 16
                                coin: table.coin
                                txid: modelData.txid
                            }
                        }
                    }
                }
            }
            }
            ColumnLayout {
                anchors.centerIn: parent
                width: Math.min(parent.width - 48, 420)
                visible: table.showEmpty
                spacing: 16

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    opacity: 0.5
                    Repeater {
                        model: 4
                        delegate: RowLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            Skeleton { Layout.preferredWidth: 96; height: 12 }
                            Skeleton { Layout.preferredWidth: 64; height: 12 }
                            Skeleton { Layout.fillWidth: true; height: 12 }
                            Skeleton { Layout.preferredWidth: 56; height: 12 }
                        }
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        text: qsTr("This wallet has not made any transactions yet.")
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 13
                        font.weight: Font.Medium
                        wrapMode: Text.Wrap
                    }
                    Text {
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        text: qsTr("Use Send or Receive above to move %1. Your history will show up here once activity is recorded in the wallet.").arg(table.ticker)
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                    }
                }
            }
        }

        // Pagination footer
        Rectangle {
            id: footer
            Layout.fillWidth: true
            implicitHeight: footerCol.implicitHeight + 24
            visible: !table.isLoading && !table.showEmpty && !table.isError
            color: "transparent"
            border.color: Theme.border
            border.width: 0
            Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: Theme.border }

            ColumnLayout {
                id: footerCol
                anchors.fill: parent
                anchors.margins: 16
                spacing: 8

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        text: qsTr("Showing %1–%2 of %3")
                            .arg(table.formatNumber(table.rangeFrom))
                            .arg(table.formatNumber(table.rangeTo))
                            .arg(table.formatNumber(table.totalItems))
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                    }
                    Text {
                        visible: table.historyCapped
                        text: qsTr("Showing the %1 most recent wallet entries.").arg(table.formatNumber(table.listCap))
                        color: Theme.fgSubtle
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }

                RowLayout {
                    Layout.alignment: Qt.AlignRight
                    spacing: 8
                    AppButton {
                        text: "\u2039  " + qsTr("Previous")
                        variant: "secondary"
                        size: "sm"
                        enabled: table.effectivePage > 0
                        onClicked: table.page = Math.max(0, table.page - 1)
                    }
                    Text {
                        text: qsTr("Page %1 of %2").arg(table.effectivePage + 1).arg(table.totalPages)
                        color: Theme.fgMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                    }
                    AppButton {
                        text: qsTr("Next") + "  \u203A"
                        variant: "secondary"
                        size: "sm"
                        enabled: table.effectivePage < table.totalPages - 1
                        onClicked: table.page = table.page + 1
                    }
                }
            }
        }
    }
}
