import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri WalletBalanceSummary parity.
Rectangle {
    id: summary
    property var wallet
    property string coin: "verium"

    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property int maturity: coin === "vericoin" ? 500 : 100

    visible: wallet !== undefined && wallet !== null
    radius: Theme.radiusMd
    color: Theme.bgSubtle
    border.color: Theme.border
    border.width: 1
    implicitHeight: col.implicitHeight + 24
    implicitWidth: col.implicitWidth + 32

    ColumnLayout {
        id: col
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            Text {
                text: qsTr("Wallet total ")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 13
            }
            Text {
                text: wallet ? wallet.total.toLocaleString(Qt.locale(), "f", 4) + " " + summary.ticker : ""
                color: Theme.fg
                font.family: Theme.fontFamily
                font.pixelSize: 18
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
        }

        Flow {
            Layout.fillWidth: true
            spacing: 16
            Text {
                text: qsTr("Spendable ") + (wallet ? wallet.balance.toLocaleString(Qt.locale(), "f", 4) : "0") + " " + summary.ticker
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
            Text {
                text: qsTr("Unconfirmed ") + (wallet ? wallet.unconfirmed.toLocaleString(Qt.locale(), "f", 4) : "0") + " " + summary.ticker
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
            Text {
                text: qsTr("Immature ") + (wallet ? wallet.immature.toLocaleString(Qt.locale(), "f", 4) : "0") + " " + summary.ticker
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            text: qsTr("Mined or staked rewards stay immature until %1 confirmations.").arg(summary.maturity)
            color: Theme.fgSubtle
            font.family: Theme.fontFamily
            font.pixelSize: 11
        }
    }
}
