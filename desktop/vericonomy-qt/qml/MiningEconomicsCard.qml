import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri MiningEconomicsCard — solo mining profitability inputs + estimates.
Card {
    id: card
    property var dailyEstimate: null
    property string revenuePeriod: "day"
    property real marketPriceUsd: NaN
    property bool usingCustomVrmPrice: false
    property string statsSource: "local"
    property var prefs: ({})
    property var savePref: function(key, value) {}

    signal revenuePeriodPicked(string nextPeriod)

    function selectRevenuePeriod(nextPeriod) {
        card.revenuePeriodPicked(nextPeriod)
    }

    readonly property var periodDays: ({ "day": 1, "week": 7, "month": 30, "year": 365 })
    readonly property int periodMultiplier: periodDays[revenuePeriod] || 1
    readonly property string periodLabel: revenuePeriod

    readonly property real dailyCost: {
        var watts = prefs.mining_power_watts
        var kwh = prefs.mining_cost_per_kwh
        if (watts === undefined || kwh === undefined || watts <= 0 || kwh <= 0) return NaN
        return (watts / 1000.0) * 24.0 * kwh
    }
    readonly property real periodCost: isNaN(dailyCost) ? NaN : dailyCost * periodMultiplier
    readonly property real periodGrossUsd: dailyEstimate && dailyEstimate.usdPerDay != null
        ? dailyEstimate.usdPerDay * periodMultiplier : NaN
    readonly property real netUsd: !isNaN(periodGrossUsd) && !isNaN(periodCost)
        ? periodGrossUsd - periodCost : NaN

    function fmtUsd(n) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        return "$" + Number(n).toLocaleString(Qt.locale(), 'f', 2)
    }

    function fmtNum(n, d) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        return Number(n).toLocaleString(Qt.locale(), 'f', d !== undefined ? d : 4)
    }

    padding: 20

    ColumnLayout {
        spacing: 16
        Layout.fillWidth: true

        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text {
                    text: qsTr("Solo economics")
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 16
                    font.weight: Font.DemiBold
                }
                Text {
                    text: usingCustomVrmPrice
                        ? qsTr("Using your VRM price assumption for USD estimates.")
                        : (statsSource === "explorer"
                            ? qsTr("Live network stats from explorer.")
                            : qsTr("Network stats from local node — USD/BTC need explorer prices."))
                    color: Theme.fgMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }
            Row {
                spacing: 4
                Repeater {
                    model: ["day", "week", "month", "year"]
                    delegate: AppButton {
                        required property var modelData
                        text: modelData
                        size: "sm"
                        variant: card.revenuePeriod === modelData ? "primary" : "ghost"
                        onClicked: card.selectRevenuePeriod(modelData)
                    }
                }
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 3
            columnSpacing: 12
            rowSpacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text { text: qsTr("VRM price ($)"); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    placeholderText: !isNaN(marketPriceUsd)
                        ? qsTr("Live: $") + fmtNum(marketPriceUsd, 4) : qsTr("e.g. 0.07")
                    text: prefs.mining_vrm_price_usd !== undefined && prefs.mining_vrm_price_usd !== null
                        ? String(prefs.mining_vrm_price_usd) : ""
                    onEditingFinished: {
                        var v = text.trim()
                        card.savePref("mining_vrm_price_usd", v.length > 0 ? Number(v) : null)
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text { text: qsTr("Power (watts)"); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    placeholderText: qsTr("e.g. 65")
                    text: prefs.mining_power_watts !== undefined && prefs.mining_power_watts !== null
                        ? String(prefs.mining_power_watts) : ""
                    onEditingFinished: {
                        var v = text.trim()
                        card.savePref("mining_power_watts", v.length > 0 ? Number(v) : null)
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text { text: qsTr("Cost ($/kWh)"); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    placeholderText: qsTr("e.g. 0.12")
                    text: prefs.mining_cost_per_kwh !== undefined && prefs.mining_cost_per_kwh !== null
                        ? String(prefs.mining_cost_per_kwh) : ""
                    onEditingFinished: {
                        var v = text.trim()
                        card.savePref("mining_cost_per_kwh", v.length > 0 ? Number(v) : null)
                    }
                }
            }
        }

        GridLayout {
            visible: dailyEstimate !== null
            Layout.fillWidth: true
            columns: 4
            columnSpacing: 16
            rowSpacing: 12

            ColumnLayout {
                spacing: 4
                Text {
                    text: qsTr("VRM / ") + periodLabel
                    color: Theme.fgSubtle
                    font.pixelSize: 10
                    font.letterSpacing: 0.5
                }
                Text {
                    text: fmtNum(dailyEstimate ? dailyEstimate.vrmPerDay * periodMultiplier : NaN, 4)
                    color: Theme.fg
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
                Text {
                    text: qsTr("~") + fmtNum(dailyEstimate ? dailyEstimate.blocksPerDay * periodMultiplier : NaN, 3) + qsTr(" blocks")
                    color: Theme.fgSubtle
                    font.pixelSize: 11
                }
            }
            ColumnLayout {
                spacing: 4
                Text {
                    text: qsTr("USD / ") + periodLabel
                    color: Theme.fgSubtle
                    font.pixelSize: 10
                }
                Text {
                    text: dailyEstimate && dailyEstimate.usdPerDay != null
                        ? fmtUsd(dailyEstimate.usdPerDay * periodMultiplier) : "—"
                    color: Theme.fg
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
            }
            ColumnLayout {
                spacing: 4
                Text {
                    text: qsTr("BTC / ") + periodLabel
                    color: Theme.fgSubtle
                    font.pixelSize: 10
                }
                Text {
                    text: dailyEstimate && dailyEstimate.btcPerDay != null
                        ? fmtNum(dailyEstimate.btcPerDay * periodMultiplier, 8) : "—"
                    color: Theme.fg
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
            }
            ColumnLayout {
                spacing: 4
                Text {
                    text: qsTr("Est. block time")
                    color: Theme.fgSubtle
                    font.pixelSize: 10
                }
                Text {
                    text: dailyEstimate && dailyEstimate.hoursPerBlock != null
                        ? fmtNum(dailyEstimate.hoursPerBlock, 1) + " h" : "—"
                    color: Theme.fg
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
            }
        }

        Rectangle {
            visible: !isNaN(periodCost) || !isNaN(netUsd)
            Layout.fillWidth: true
            radius: Theme.radiusMd
            color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.50)
            border.color: Theme.border
            implicitHeight: costGrid.implicitHeight + 24
            GridLayout {
                id: costGrid
                anchors.fill: parent
                anchors.margins: 12
                columns: 3
                columnSpacing: 16
                ColumnLayout {
                    visible: !isNaN(periodCost)
                    spacing: 4
                    Text {
                        text: qsTr("Electricity / ") + periodLabel
                        color: Theme.fgSubtle
                        font.pixelSize: 10
                    }
                    Text {
                        text: fmtUsd(periodCost)
                        color: Theme.fg
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                }
                ColumnLayout {
                    visible: !isNaN(netUsd)
                    spacing: 4
                    Text {
                        text: qsTr("Net USD / ") + periodLabel
                        color: Theme.fgSubtle
                        font.pixelSize: 10
                    }
                    Text {
                        text: fmtUsd(netUsd)
                        color: netUsd >= 0 ? Theme.success : Theme.danger
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                }
                ColumnLayout {
                    visible: !isNaN(periodGrossUsd) && !isNaN(periodCost)
                    spacing: 4
                    Text {
                        text: qsTr("Gross USD / ") + periodLabel
                        color: Theme.fgSubtle
                        font.pixelSize: 10
                    }
                    Text {
                        text: fmtUsd(periodGrossUsd)
                        color: Theme.fg
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            AppButton {
                text: qsTr("Open profitability calculator")
                variant: "ghost"
                size: "sm"
                onClicked: HostLinks.open("https://explorer.vericonomy.com/insights")
            }
        }
    }
}
