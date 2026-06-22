import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

Item {
    id: page
    property var security
    property string coin: "verium"
    property var parseJson: function(s, fb) { return fb || {} }

    readonly property var twoFactor: security
        ? parseJson(security.twoFactorJson, {})
        : {}
    readonly property var autoLock: security
        ? parseJson(security.autoLockJson, {})
        : {}
    readonly property var spending: security
        ? parseJson(security.spendingJson, {})
        : {}
    readonly property var exportResult: security
        ? parseJson(security.exportResultJson, {})
        : {}

    Component.onCompleted: if (security) security.refresh()
    onCoinChanged: if (security) {
        security.coin = coin
        security.refresh()
    }

    Flickable {
        anchors.fill: parent
        contentWidth: width
        contentHeight: col.implicitHeight
        clip: true

        ColumnLayout {
            id: col
            width: parent.width
            spacing: 16
            Item { Layout.preferredHeight: 8; Layout.fillWidth: true }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader {
                        title: qsTr("Recovery phrase export")
                        subtitle: qsTr("Export your BIP39 phrase or HD xprv (light wallet).")
                        Layout.fillWidth: true
                    }
                    TextField {
                        id: exportPass
                        Layout.fillWidth: true
                        placeholderText: qsTr("Wallet passphrase")
                        echoMode: TextInput.Password
                        color: Theme.fg
                    }
                    AppButton {
                        text: qsTr("Export recovery material")
                        onClicked: if (security) security.exportRecovery(exportPass.text)
                    }
                    Text {
                        visible: exportResult.kind === "mnemonic"
                        text: exportResult.mnemonic || ""
                        color: Theme.fg
                        font.family: Theme.monoFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    Text {
                        visible: exportResult.kind === "hd_master_xprv"
                        text: exportResult.xprv || ""
                        color: Theme.fg
                        font.family: Theme.monoFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader { title: qsTr("Two-factor authentication"); Layout.fillWidth: true }
                    StatusPill {
                        tone: twoFactor.enabled === true ? "success" : "neutral"
                        text: twoFactor.enabled === true ? qsTr("Enabled") : qsTr("Disabled")
                    }
                    RowLayout {
                        spacing: 8
                        TextField {
                            id: tfaCode
                            Layout.fillWidth: true
                            placeholderText: qsTr("6-digit code")
                            color: Theme.fg
                        }
                        AppButton {
                            text: qsTr("Start setup")
                            variant: "secondary"
                            visible: twoFactor.enabled !== true
                            onClicked: if (security) security.startTwoFactor()
                        }
                        AppButton {
                            text: qsTr("Confirm")
                            visible: twoFactor.enabled !== true
                            onClicked: {
                                var enroll = exportResult
                                if (security && enroll.secret_base32)
                                    security.confirmTwoFactor(tfaCode.text, enroll.secret_base32)
                            }
                        }
                        AppButton {
                            text: qsTr("Disable")
                            variant: "danger"
                            visible: twoFactor.enabled === true
                            onClicked: if (security) security.disableTwoFactor(tfaCode.text)
                        }
                    }
                    Text {
                        visible: typeof exportResult.otpauth_uri === "string"
                            && exportResult.otpauth_uri.length > 0
                        text: exportResult.otpauth_uri || ""
                        color: Theme.fgMuted
                        font.pixelSize: 10
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader { title: qsTr("Auto-lock"); Layout.fillWidth: true }
                    CheckBox {
                        id: autoLockEnabled
                        text: qsTr("Lock wallet after idle period")
                        checked: autoLock.enabled === true
                    }
                    TextField {
                        id: idleSeconds
                        Layout.fillWidth: true
                        text: String(autoLock.idle_seconds || 900)
                        placeholderText: qsTr("Idle seconds")
                        color: Theme.fg
                    }
                    AppButton {
                        text: qsTr("Save auto-lock")
                        onClicked: {
                            if (!security) return
                            var payload = {
                                enabled: autoLockEnabled.checked,
                                idle_seconds: parseInt(idleSeconds.text) || 900,
                                lock_on_blur: false,
                                lock_on_sleep: false
                            }
                            security.saveAutoLock(JSON.stringify(payload))
                        }
                    }
                }
            }

            Card {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                padding: 20
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    SectionHeader { title: qsTr("Spending controls"); Layout.fillWidth: true }
                    CheckBox {
                        id: allowlistOnly
                        text: qsTr("Allowlist only (send book “send” entries)")
                        checked: spending.allowlist_only === true
                    }
                    CheckBox {
                        id: firstSendConfirm
                        text: qsTr("Extra confirmation for new recipients")
                        checked: spending.require_first_send_confirmation !== false
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        TextField {
                            id: capVrm
                            Layout.fillWidth: true
                            placeholderText: qsTr("Daily cap VRM")
                            text: spending.daily_spend_cap_vrm != null ? String(spending.daily_spend_cap_vrm) : ""
                            color: Theme.fg
                        }
                        TextField {
                            id: capVrc
                            Layout.fillWidth: true
                            placeholderText: qsTr("Daily cap VRC")
                            text: spending.daily_spend_cap_vrc != null ? String(spending.daily_spend_cap_vrc) : ""
                            color: Theme.fg
                        }
                    }
                    AppButton {
                        text: qsTr("Save spending controls")
                        onClicked: {
                            if (!security) return
                            var payload = Object.assign({}, spending, {
                                allowlist_only: allowlistOnly.checked,
                                require_first_send_confirmation: firstSendConfirm.checked,
                                daily_spend_cap_vrm: capVrm.text.length ? parseFloat(capVrm.text) : null,
                                daily_spend_cap_vrc: capVrc.text.length ? parseFloat(capVrc.text) : null
                            })
                            security.saveSpending(JSON.stringify(payload))
                        }
                    }
                }
            }

            Text {
                visible: security && security.lastMessage.length > 0
                text: security ? security.lastMessage : ""
                color: Theme.fgMuted
                font.pixelSize: 11
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }

            Item { Layout.preferredHeight: 16; Layout.fillWidth: true }
        }
    }
}
