import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri DashboardHero parity — wallet / market / mining-staking + network footer.
Rectangle {
    id: hero
    width: parent ? parent.width : implicitWidth
    implicitHeight: inner.implicitHeight + 48
    radius: Theme.radiusLg
    color: Theme.bgPanel
    border.color: Theme.border

    property var snap: ({})
    property string coin: "verium"
    property bool loading: false
    readonly property string ticker: coin === "vericoin" ? "VRC" : "VRM"
    readonly property bool isVerium: coin !== "vericoin"
    readonly property bool showSyncBar: !hero.snap.synced && hero.snap.sync_target > (hero.snap.local_blocks || 0)
        && hero.snap.local_blocks != null

    function fmt(n, d) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        return Number(n).toLocaleString(Qt.locale(), 'f', d !== undefined ? d : 4)
    }
    function fmtUsd(n) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        var v = Number(n)
        if (v >= 1000000) return "$" + hero.fmt(v / 1000000, 2) + "M"
        if (v >= 1000) return "$" + hero.fmt(v / 1000, 2) + "K"
        return "$" + hero.fmt(v, 4)
    }
    function fmtDiff(n) {
        if (n === undefined || n === null || isNaN(n)) return "—"
        var v = Number(n)
        return v >= 0.0001 ? hero.fmt(v, 4) : hero.fmt(v, 6)
    }
    function hasText(v) {
        return v !== undefined && v !== null && String(v).length > 0
    }

    // Top accent gradient line
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 1
        opacity: 0.5
        gradient: Gradient {
            GradientStop { position: 0.0; color: "transparent" }
            GradientStop { position: 0.5; color: Theme.accent }
            GradientStop { position: 1.0; color: "transparent" }
        }
    }

    ColumnLayout {
        id: inner
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 24
        spacing: 16

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            StatusPill {
                loading: hero.loading
                tone: hero.snap.synced === true ? "success" : "accent"
                text: hero.hasText(hero.snap.state_label) ? hero.snap.state_label : "—"
            }
            StatusPill {
                tone: "neutral"
                text: hero.snap.network_mode === "testnet" ? qsTr("Testnet") : qsTr("Mainnet")
            }
        }

        Text {
            visible: hero.hasText(hero.snap.activity_title)
            text: hero.snap.activity_title || ""
            color: Theme.fgMuted
            font.pixelSize: 12
            Layout.fillWidth: true
        }

        ColumnLayout {
            visible: hero.showSyncBar
            Layout.fillWidth: true
            spacing: 6
            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: qsTr("of ~") + hero.fmt(hero.snap.sync_target, 0) + qsTr(" network tip")
                        + (hero.snap.behind > 0 ? (" · ~" + hero.fmt(hero.snap.behind, 0) + qsTr(" blocks behind")) : "")
                    color: Theme.fgMuted
                    font.pixelSize: 11
                    Layout.fillWidth: true
                }
                Text {
                    text: hero.fmt((hero.snap.verification_progress || 0) * 100, 0) + "%"
                    color: Theme.fgMuted
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                }
            }
            Rectangle {
                Layout.fillWidth: true
                height: 6
                radius: 3
                color: Theme.border
                Rectangle {
                    width: parent.width * Math.min(1, (hero.snap.verification_progress || 0))
                    height: parent.height
                    radius: 3
                    color: Theme.accent
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            HeroMiniPanel {
                Layout.fillWidth: true
                title: qsTr("Wallet")
                s1l: qsTr("Balance")
                s1v: hero.fmt(hero.snap.wallet ? hero.snap.wallet.balance : null)
                s2l: qsTr("Immature")
                s2v: hero.fmt(hero.snap.wallet ? hero.snap.wallet.immature_balance : null)
                s3l: hero.isVerium ? qsTr("Unconfirmed") : qsTr("Stake weight")
                s3v: hero.isVerium
                    ? hero.fmt(hero.snap.wallet ? hero.snap.wallet.unconfirmed_balance : null)
                    : hero.fmt(hero.snap.stake_weight, 0)
                s4l: qsTr("Transactions")
                s4v: hero.snap.wallet ? String(hero.snap.wallet.txcount || 0) : "0"
            }
            HeroMiniPanel {
                Layout.fillWidth: true
                title: qsTr("Market")
                s1l: hero.ticker
                s1v: hero.fmtUsd(hero.snap.explorer ? hero.snap.explorer.price_usd : null)
                s2l: qsTr("24h vol")
                s2v: hero.fmtUsd(hero.snap.explorer ? hero.snap.explorer.volume_24h_usd : null)
                s3l: hero.isVerium ? qsTr("Block reward") : qsTr("Interest rate")
                s3v: hero.isVerium
                    ? (hero.snap.explorer && hero.snap.explorer.block_reward != null
                        ? hero.fmt(hero.snap.explorer.block_reward, 4) + " " + hero.ticker : "—")
                    : (hero.snap.explorer && hero.snap.explorer.stake_interest != null
                        ? hero.fmt(hero.snap.explorer.stake_interest, 2) + "%" : "—")
                s4l: qsTr("Supply")
                s4v: hero.snap.explorer && hero.snap.explorer.supply != null
                    ? hero.fmt(hero.snap.explorer.supply, 0) : "—"
            }
            HeroMiniPanel {
                Layout.fillWidth: true
                title: hero.isVerium ? qsTr("Your mining") : qsTr("Your staking")
                s1l: hero.isVerium ? qsTr("Blocks found") : qsTr("Stake rewards")
                s1v: String(hero.snap.blocks_found || 0)
                s2l: hero.isVerium ? qsTr("Hashrate") : qsTr("Interest rate")
                s2v: hero.isVerium
                    ? (hero.snap.local_hashrate > 0 ? hero.fmt(hero.snap.local_hashrate, 0) + " H/m" : "—")
                    : (hero.snap.explorer && hero.snap.explorer.stake_interest != null
                        ? hero.fmt(hero.snap.explorer.stake_interest, 2) + "%" : "—")
                s3l: hero.isVerium ? qsTr("Network share") : qsTr("Network staked")
                s3v: hero.isVerium
                    ? (hero.snap.network_share_percent > 0
                        ? hero.fmt(hero.snap.network_share_percent, 2) + "%" : "—")
                    : (hero.snap.net_stake_weight > 0 && hero.snap.explorer && hero.snap.explorer.supply
                        ? hero.fmt((hero.snap.net_stake_weight / hero.snap.explorer.supply) * 100, 2) + "%" : "—")
                s4l: hero.isVerium ? qsTr("Est. daily") : qsTr("Stake share")
                s4v: hero.isVerium
                    ? (hero.snap.est_daily_vrm > 0
                        ? hero.fmt(hero.snap.est_daily_vrm, 3) + " VRM" : "—")
                    : (hero.snap.stake_weight > 0 && hero.snap.net_stake_weight > 0
                        ? hero.fmt((hero.snap.stake_weight / hero.snap.net_stake_weight) * 100, 2) + "%" : "—")
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.border
            opacity: 0.5
        }

        Text {
            text: (hero.isVerium ? "Verium" : "Vericoin") + " " + qsTr("network")
            color: Theme.fgSubtle
            font.pixelSize: 10
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 5
            columnSpacing: 14
            StatTile {
                label: hero.isVerium ? qsTr("Network hashrate") : qsTr("PoS difficulty")
                value: hero.isVerium
                    ? (hero.snap.network_hash_khm > 0
                        ? hero.fmt(hero.snap.network_hash_khm, 1) + " kH/m" : "—")
                    : hero.fmtDiff(hero.snap.explorer ? hero.snap.explorer.difficulty : null)
            }
            StatTile {
                visible: hero.isVerium
                label: qsTr("Difficulty")
                value: hero.fmtDiff(hero.snap.explorer ? hero.snap.explorer.difficulty : null)
            }
            StatTile {
                visible: !hero.isVerium
                label: qsTr("Network staked")
                value: hero.snap.net_stake_weight > 0 && hero.snap.explorer && hero.snap.explorer.supply
                    ? hero.fmt((hero.snap.net_stake_weight / hero.snap.explorer.supply) * 100, 2) + "%" : "—"
            }
            StatTile {
                label: hero.isVerium ? qsTr("Avg. block time") : qsTr("Block time")
                value: hero.snap.block_time_min > 0
                    ? hero.fmt(hero.snap.block_time_min, 1) + " min" : "—"
            }
            StatTile {
                label: qsTr("Mempool")
                value: hero.snap.mempool > 0 ? String(hero.snap.mempool) : "—"
            }
            StatTile {
                label: qsTr("Peers · ") + (hero.hasText(hero.snap.peer_status) ? hero.snap.peer_status : "Offline")
                value: hero.snap.connected === true ? String(hero.snap.connections || 0) : "—"
            }
        }
    }

    component HeroMiniPanel: Rectangle {
        property string title: ""
        property string s1l: ""
        property string s1v: ""
        property string s2l: ""
        property string s2v: ""
        property string s3l: ""
        property string s3v: ""
        property string s4l: ""
        property string s4v: ""
        implicitHeight: col.implicitHeight + 28
        radius: Theme.radiusMd
        color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.35)
        border.color: Qt.rgba(Theme.border.r, Theme.border.g, Theme.border.b, 0.45)
        ColumnLayout {
            id: col
            anchors.fill: parent
            anchors.margins: 14
            spacing: 10
            Text {
                text: title
                color: Theme.fg
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            GridLayout {
                columns: 2
                columnSpacing: 10
                rowSpacing: 10
                Layout.fillWidth: true
                StatTile { label: s1l; value: s1v }
                StatTile { label: s2l; value: s2v }
                StatTile { label: s3l; value: s3v }
                StatTile { label: s4l; value: s4v }
            }
        }
    }
}
