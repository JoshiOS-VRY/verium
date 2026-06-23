import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Tauri Mining.tsx parity — solo + pool modes, hero, stats, controls, chart.
Item {
    id: page
    property string coin: "verium"
    property var mining
    property var poolMiner
    property var node
    property var wallet
    property var settings
    property var walletMode
    property var dashboard
    property var explorer
    property var poolStats
    property var soundCtrl
    property var parseJson: function(s, fb) { try { return JSON.parse(s) } catch(e) { return fb || {} } }

    property string revenuePeriod: "day"

    readonly property var explorerStats: page.explorer
        ? page.parseJson(page.explorer.statsJson, {})
        : {}
    readonly property bool minerBooting: {
        if (!page.minerActive || page.localHashrate > 0) return false
        var started = page.mining ? page.mining.minerStartedAt : 0
        if (!started || started <= 0) return page.minerActive
        return (Date.now() / 1000 - started) < 120
    }
    readonly property bool minerLive: page.minerActive || page.minerBooting
    readonly property real marketPriceUsd: explorerStats.price_usd !== undefined
        ? explorerStats.price_usd : NaN
    readonly property bool usingCustomVrmPrice: prefs.mining_vrm_price_usd !== undefined
        && prefs.mining_vrm_price_usd !== null && Number(prefs.mining_vrm_price_usd) > 0
    readonly property real revenuePriceUsd: usingCustomVrmPrice
        ? Number(prefs.mining_vrm_price_usd) : marketPriceUsd
    readonly property var dailyEstimate: page.estimateDailyMining()
    readonly property string networkStatsSource: explorerStats.network_hash != null ? "explorer" : "local"

    readonly property var prefs: page.settings
        ? page.parseJson(page.settings.prefsJson, {})
        : {}
    readonly property bool isLight: walletMode ? walletMode.isLight : false
    property string activeMode: page.isLight ? "pool" : (page.prefs.mining_mode || "pool")
    readonly property bool autoAdjustThreads: prefs.auto_adjust_mine_threads !== false
    readonly property int manualThreads: prefs.auto_mine_threads !== undefined ? prefs.auto_mine_threads : 2

    readonly property bool nodeConnected: node && node.connected
    readonly property int syncTarget: node ? Math.max(node.headers, node.blocks) : 0
    readonly property int blocksBehind: node ? Math.max(0, syncTarget - node.blocks) : 0
    readonly property bool chainSynced: nodeConnected && node.verificationProgress >= 0.9999 && blocksBehind === 0
    readonly property bool ibd: nodeConnected && !chainSynced
    readonly property bool syncStalled: false

    readonly property bool minerActive: mining && mining.minerActive
    readonly property real localHashrate: mining ? mining.hashrate : 0
    readonly property real networkHashHs: mining ? mining.networkHashps : 0
    readonly property real networkKhm: networkHashHs > 0 ? (networkHashHs * 60.0) / 1000.0 : 0
    readonly property real networkShare: localHashrate > 0 && networkHashHs > 0
        ? (localHashrate / (networkHashHs * 60.0)) * 100.0 : 0
    readonly property int displayThreads: minerActive && mining ? mining.threads : page.resolvedThreads

    readonly property int resolvedThreads: autoAdjustThreads
        ? Math.min(poolMiner ? poolMiner.suggestedThreads : 2, poolMiner ? poolMiner.maxThreads : 8)
        : manualThreads

    readonly property bool staticAddressMissing: (prefs.mining_reward_address_mode === "static")
        && !(prefs.mining_reward_address && String(prefs.mining_reward_address).trim().length > 0)

    readonly property string difficultyText: mining && mining.difficulty > 0
        ? mining.difficulty.toLocaleString(Qt.locale(), 'f', 7) : "—"

    readonly property var estHoursPerBlock: {
        if (localHashrate <= 0 || networkHashHs <= 0 || !mining || mining.blocksPerHour <= 0)
            return null
        var share = localHashrate / (networkHashHs * 60.0)
        if (share <= 0) return null
        return 1.0 / (share * mining.blocksPerHour)
    }

    property var hashSamples: []

    Component.onCompleted: {
        page.refreshMining()
        if (page.explorer) page.explorer.refresh()
    }
    onVisibleChanged: {
        if (visible) {
            page.refreshMining()
            if (page.explorer) page.explorer.refresh()
        }
    }

    function refreshMining() {
        if (page.mining) page.mining.refresh()
        if (page.poolMiner) {
            page.poolMiner.detect()
            page.poolMiner.refresh()
        }
    }

    function savePref(key, value) {
        if (!page.settings) return
        var p = Object.assign({}, page.prefs)
        p[key] = value
        page.settings.savePrefs(JSON.stringify(p))
    }

    function estimateDailyMining() {
        var hr = page.localHashrate
        var networkHs = page.networkHashHs
        var blocksPerHour = page.mining ? page.mining.blocksPerHour : 0
        var blockReward = page.mining ? page.mining.blockReward : 0
        if (hr <= 0 || networkHs <= 0 || blocksPerHour <= 0 || blockReward <= 0)
            return null
        var networkHm = networkHs * 60.0
        var share = hr / networkHm
        var blocksPerDay = share * blocksPerHour * 24.0
        var vrmPerDay = blocksPerDay * blockReward
        var hoursPerBlock = blocksPerDay > 0 ? 24.0 / blocksPerDay : null
        var priceUsd = page.revenuePriceUsd
        var priceBtc = page.explorerStats.price_btc
        return {
            blocksPerDay: blocksPerDay,
            vrmPerDay: vrmPerDay,
            usdPerDay: !isNaN(priceUsd) ? vrmPerDay * priceUsd : null,
            btcPerDay: priceBtc !== undefined && priceBtc !== null ? vrmPerDay * priceBtc : null,
            hoursPerBlock: hoursPerBlock
        }
    }

    function setMiningMode(mode) {
        page.activeMode = mode
        page.savePref("mining_mode", mode)
    }

    function appendSample(rate) {
        var now = Date.now()
        var copy = page.hashSamples.slice(0)
        copy.push({ t: now, hashrate: rate })
        if (copy.length > 60)
            copy = copy.slice(copy.length - 60)
        page.hashSamples = copy
    }

    WalletUnlockGate {
        anchors.fill: parent
        wallet: page.wallet
        coin: page.coin
        title: qsTr("Unlock to mine")
        description: page.isLight
            ? qsTr("Enter your wallet passphrase to mine on the public Verium pool.")
            : qsTr("Enter your wallet passphrase to access solo CPU mining or mine on the public Verium pool.")

        ScrollView {
            anchors.fill: parent
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

            ColumnLayout {
                width: page.width
                spacing: 16
                Item { Layout.preferredHeight: 8; Layout.fillWidth: true }

                MiningStatusBanner {
                    visible: !page.isLight
                    Layout.fillWidth: true
                    Layout.leftMargin: 24
                    Layout.rightMargin: 24
                    syncStalled: page.syncStalled
                    chainSynced: page.chainSynced
                    ibd: page.ibd
                    localBlocks: node ? node.blocks : 0
                    syncTarget: page.syncTarget
                    blocksBehind: page.blocksBehind
                    immatureBalance: wallet ? wallet.immature : 0
                }

                MiningModeToggle {
                    visible: !page.isLight
                    Layout.leftMargin: 24
                    mode: page.activeMode
                    onModePicked: (m) => page.setMiningMode(m)
                }

                ColumnLayout {
                    visible: page.activeMode === "pool" || page.isLight
                    Layout.fillWidth: true
                    spacing: 16

                    PoolStatsStrip {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        poolStats: page.poolStats
                        parseJson: page.parseJson
                        pollEnabled: page.coin === "verium"
                    }

                    PoolMiningControls {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        payoutAddress: prefs.pool_payout_address || ""
                        workerName: prefs.pool_worker_name || "wallet"
                        running: poolMiner && poolMiner.running
                        sidecarFound: poolMiner && poolMiner.sidecarFound
                        nodeConnected: page.nodeConnected
                        chainSynced: page.chainSynced
                        syncStalled: page.syncStalled
                        hashrateHm: poolMiner ? poolMiner.hashrateHm : 0
                        connectionState: poolMiner ? poolMiner.connectionState : ""
                        lastMessage: poolMiner ? poolMiner.lastMessage : ""
                        acceptedShares: poolMiner ? poolMiner.acceptedShares : 0
                        rejectedShares: poolMiner ? poolMiner.rejectedShares : 0
                        autoAdjust: page.autoAdjustThreads
                        manualThreads: page.manualThreads
                        suggestedThreads: poolMiner ? poolMiner.suggestedThreads : 2
                        maxThreads: poolMiner ? poolMiner.maxThreads : 8
                        activeThreads: poolMiner ? poolMiner.activeThreads : 0
                        onPayoutChanged: (addr) => page.savePref("pool_payout_address", addr)
                        onWorkerChanged: (name) => page.savePref("pool_worker_name", name)
                        onAutoAdjustToggled: (v) => page.savePref("auto_adjust_mine_threads", v)
                        onThreadsEdited: (n) => page.savePref("auto_mine_threads", n)
                        onStartRequested: (username) => {
                            if (!poolMiner || !username || username.length === 0) return
                            if (page.minerActive && page.mining) page.mining.stopMiner()
                            var threads = page.resolvedThreads
                            if (poolMiner.sidecarFound && !page.chainSynced)
                                threads = Math.max(1, Math.floor(threads / 2))
                            poolMiner.startPool(
                                "stratum+tcp://mine.vericonomy.com:3333",
                                username,
                                "x",
                                threads
                            )
                        }
                        onStopRequested: if (poolMiner) poolMiner.stopPool()
                    }

                    Card {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        padding: 20
                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                text: qsTr("Pool dashboard")
                                color: Theme.fg
                                font.family: Theme.fontFamily
                                font.pixelSize: 16
                                font.weight: Font.DemiBold
                                Layout.fillWidth: true
                            }
                            AppButton {
                                text: qsTr("Open pool.vericonomy.com")
                                onClicked: HostLinks.open("https://pool.vericonomy.com")
                            }
                        }
                    }

                    MiningHashrateChart {
                        visible: poolMiner && poolMiner.running
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        samples: page.hashSamples
                        active: poolMiner && poolMiner.running
                    }

                    PoolDisclaimer {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                    }
                }

                ColumnLayout {
                    visible: !page.isLight && page.activeMode === "solo"
                    Layout.fillWidth: true
                    spacing: 16

                    MiningHero {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        active: page.minerActive
                        minerBooting: page.minerBooting
                        localHashrate: page.localHashrate
                        displayThreads: page.displayThreads
                        chainSynced: page.chainSynced
                        syncStalled: page.syncStalled
                        staticAddressMissing: page.staticAddressMissing
                        blocksBehind: page.blocksBehind
                        errorText: mining ? mining.lastMessage : ""
                        onStartRequested: {
                            if (!mining) return
                            if (poolMiner && poolMiner.running) poolMiner.stopPool()
                            mining.startMiner(page.resolvedThreads)
                        }
                        onStopRequested: if (mining) mining.stopMiner()
                    }

                    MiningHashrateChart {
                        visible: page.minerLive
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        samples: page.hashSamples
                        active: page.minerActive
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        spacing: 12

                        MiningStatTile {
                            Layout.fillWidth: true
                            label: qsTr("Hashrate")
                            iconName: "cpu"
                            highlight: page.minerActive
                            value: page.localHashrate > 0
                                ? page.localHashrate.toLocaleString(Qt.locale(), 'f', 2) : "—"
                            unit: "H/m"
                        }
                        MiningStatTile {
                            Layout.fillWidth: true
                            label: qsTr("Network hashrate")
                            iconName: "network"
                            value: page.networkKhm > 0
                                ? page.networkKhm.toLocaleString(Qt.locale(), 'f', 2) : "—"
                            unit: "kH/m"
                        }
                        MiningStatTile {
                            Layout.fillWidth: true
                            label: qsTr("Difficulty")
                            iconName: "gauge"
                            value: page.difficultyText
                        }
                        MiningStatTile {
                            Layout.fillWidth: true
                            label: qsTr("Est. next block")
                            iconName: "scroll-text"
                            value: page.estHoursPerBlock != null
                                ? page.estHoursPerBlock.toLocaleString(Qt.locale(), 'f', 1) : "—"
                            unit: page.estHoursPerBlock != null ? "h" : ""
                            hint: mining && mining.blockReward > 0
                                ? qsTr("reward") + " " + mining.blockReward.toLocaleString(Qt.locale(), 'f', 4) + " VRM"
                                : ""
                        }
                    }

                    Rectangle {
                        visible: page.networkShare > 0 && page.localHashrate > 0
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        radius: Theme.radiusMd
                        color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.50)
                        border.color: Theme.border
                        implicitHeight: shareCol.implicitHeight + 24
                        ColumnLayout {
                            id: shareCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 8
                            RowLayout {
                                Layout.fillWidth: true
                                Text {
                                    text: qsTr("Your network share")
                                    color: Theme.fgMuted
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 13
                                    Layout.fillWidth: true
                                }
                                Text {
                                    text: page.networkShare.toLocaleString(Qt.locale(), 'f', 2) + "%"
                                    color: Theme.fg
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.DemiBold
                                }
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                height: 8
                                radius: 4
                                color: Theme.bgPanel
                                Rectangle {
                                    width: parent.width * Math.min(1, page.networkShare / 100.0)
                                    height: parent.height
                                    radius: 4
                                    color: Theme.accent
                                }
                            }
                        }
                    }

                    MiningControlsCard {
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        autoAdjust: page.autoAdjustThreads
                        manualThreads: page.manualThreads
                        suggestedThreads: poolMiner ? poolMiner.suggestedThreads : 2
                        maxThreads: poolMiner ? poolMiner.maxThreads : 8
                        displayThreads: page.displayThreads
                        isMining: page.minerActive
                        controlsDisabled: page.minerActive
                        autoMineOnOpen: prefs.auto_mine_on_open === true
                        playSoundOnBlock: prefs.play_sound_on_block_mined === true
                        rewardMode: prefs.mining_reward_address_mode || "dynamic"
                        rewardAddress: prefs.mining_reward_address || ""
                        onAutoAdjustToggled: (v) => page.savePref("auto_adjust_mine_threads", v)
                        onThreadsEdited: (n) => page.savePref("auto_mine_threads", n)
                        onAutoMineOnOpenToggled: (v) => page.savePref("auto_mine_on_open", v)
                        onPlaySoundToggled: (v) => {
                            page.savePref("play_sound_on_block_mined", v)
                            if (v && page.soundCtrl) page.soundCtrl.playBlockChime()
                        }
                        onRewardModePicked: (m) => page.savePref("mining_reward_address_mode", m)
                        onRewardAddressEdited: (a) => page.savePref("mining_reward_address", a)
                    }

                    MiningEconomicsCard {
                        visible: page.dailyEstimate !== null && page.localHashrate > 0
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        dailyEstimate: page.dailyEstimate
                        revenuePeriod: page.revenuePeriod
                        marketPriceUsd: page.marketPriceUsd
                        usingCustomVrmPrice: page.usingCustomVrmPrice
                        statsSource: page.networkStatsSource
                        prefs: page.prefs
                        savePref: page.savePref
                        onRevenuePeriodPicked: (p) => page.revenuePeriod = p
                    }

                    MiningHashrateChart {
                        visible: !page.minerLive
                        Layout.fillWidth: true
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        samples: page.hashSamples
                        active: false
                        emptyWhenIdle: true
                    }
                }

                Item { Layout.preferredHeight: 24; Layout.fillWidth: true }
            }
        }
    }

    Connections {
        target: page.mining
        function onMiningRefreshed(ok) {
            if (ok && page.minerActive && page.localHashrate > 0)
                page.appendSample(page.localHashrate)
        }
    }

    Connections {
        target: page.poolMiner
        function onPoolRefreshed(ok) {
            if (ok && page.poolMiner && page.poolMiner.running && page.poolMiner.hashrateHm > 0)
                page.appendSample(page.poolMiner.hashrateHm)
        }
    }

    Connections {
        target: page.settings
        function onPrefsLoaded(ok) {
            if (ok && !page.isLight)
                page.activeMode = page.prefs.mining_mode || "pool"
        }
        function onPrefsSaved(ok) {
            if (ok && !page.isLight)
                page.activeMode = page.prefs.mining_mode || "pool"
        }
    }

    Timer {
        interval: 5000
        running: page.visible && ((page.minerActive) || (poolMiner && poolMiner.running))
        repeat: true
        onTriggered: page.refreshMining()
    }
}
