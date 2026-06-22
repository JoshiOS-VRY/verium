import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import com.vericonomy.verium

// Address book. Mirrors desktop/verium-app/src/pages/AddressBook.tsx (desktop layout).
Item {
    id: page
    property var addressBook
    property string coin: "verium"
    property var parseJson: function(s, fb) { return fb || [] }

    property string filter: "send"
    property var draft: null
    property bool saving: false

    readonly property var entries: page.addressBook
        ? page.parseJson(page.addressBook.entriesJson, [])
        : []

    readonly property var filtered: {
        var rows = []
        for (var i = 0; i < entries.length; i++) {
            if (entries[i].category === page.filter)
                rows.push(entries[i])
        }
        rows.sort(function(a, b) {
            return (a.label || "").localeCompare(b.label || "")
        })
        return rows
    }

    function emptyDraft(category) {
        return {
            id: "",
            address: "",
            label: "",
            notes: "",
            category: category || page.filter
        }
    }

    function startDraft(category) {
        page.draft = page.emptyDraft(category || page.filter)
    }

    function saveDraft() {
        if (!page.addressBook || !page.draft || page.draft.address.trim().length === 0)
            return
        page.saving = true
        page.addressBook.upsert(JSON.stringify(page.draft))
    }

    Component.onCompleted: if (page.addressBook) page.addressBook.refresh()
    onCoinChanged: {
        if (page.addressBook) {
            page.addressBook.coin = page.coin
            page.addressBook.refresh()
        }
        page.draft = null
    }

    Connections {
        target: page.addressBook
        function onEntrySaved(ok) {
            page.saving = false
            if (ok) {
                page.filter = page.draft ? page.draft.category : page.filter
                page.draft = null
            }
        }
        function onEntryDeleted(ok) { /* list refreshes via controller */ }
    }

    Flickable {
        anchors.fill: parent
        contentWidth: width
        contentHeight: col.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds

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
                    spacing: 14

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4
                            Text {
                                text: qsTr("Address book")
                                color: Theme.fg
                                font.family: Theme.fontFamily
                                font.pixelSize: 16
                                font.weight: Font.DemiBold
                            }
                            Text {
                                text: qsTr("Saved sending and receiving addresses. Stored locally only.")
                                color: Theme.fgMuted
                                font.family: Theme.fontFamily
                                font.pixelSize: 12
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }
                        }
                        AppButton {
                            text: qsTr("New entry")
                            size: "sm"
                            onClicked: page.startDraft(page.filter)
                        }
                    }

                    Rectangle {
                        Layout.preferredWidth: 160
                        implicitHeight: 36
                        radius: Theme.radiusMd
                        color: Theme.bgSubtle
                        border.color: Theme.border
                        border.width: 1
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 4
                            spacing: 2
                            Repeater {
                                model: ["send", "receive"]
                                delegate: Rectangle {
                                    required property string modelData
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    radius: Theme.radiusSm
                                    color: page.filter === modelData ? Theme.accent : "transparent"
                                    Text {
                                        anchors.centerIn: parent
                                        text: modelData === "send" ? qsTr("Send") : qsTr("Receive")
                                        color: page.filter === modelData ? Theme.accentFg : Theme.fgMuted
                                        font.family: Theme.fontFamily
                                        font.pixelSize: 12
                                        font.capitalization: Font.Capitalize
                                    }
                                    TapHandler { onTapped: page.filter = modelData }
                                }
                            }
                        }
                    }

                    Rectangle {
                        visible: page.draft !== null
                        Layout.fillWidth: true
                        radius: Theme.radiusMd
                        color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.05)
                        border.color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.4)
                        border.width: 1
                        implicitHeight: draftCol.implicitHeight + 24
                        ColumnLayout {
                            id: draftCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 10
                            DraftForm {
                                draft: page.draft
                                onDraftUpdated: function(d) { page.draft = d }
                            }
                            Text {
                                visible: page.addressBook && page.addressBook.lastMessage.length > 0 && !page.saving
                                text: page.addressBook ? page.addressBook.lastMessage : ""
                                color: Theme.danger
                                font.pixelSize: 11
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                Item { Layout.fillWidth: true }
                                AppButton {
                                    text: qsTr("Cancel")
                                    variant: "ghost"
                                    size: "sm"
                                    enabled: !page.saving
                                    onClicked: page.draft = null
                                }
                                AppButton {
                                    text: page.saving ? qsTr("Saving…") : qsTr("Save")
                                    size: "sm"
                                    enabled: page.draft && page.draft.address.trim().length > 0 && !page.saving
                                    onClicked: page.saveDraft()
                                }
                            }
                        }
                    }

                    Text {
                        visible: page.addressBook && page.addressBook.loading
                        text: qsTr("Loading…")
                        color: Theme.fgMuted
                        font.pixelSize: 13
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        Layout.topMargin: 24
                        Layout.bottomMargin: 24
                    }

                    Text {
                        visible: !page.addressBook || (!page.addressBook.loading && page.filtered.length === 0)
                        text: page.filter === "send"
                            ? qsTr("No send addresses yet.")
                            : qsTr("No receive addresses yet.")
                        color: Theme.fgSubtle
                        font.pixelSize: 13
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        Layout.topMargin: 32
                        Layout.bottomMargin: 32
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        visible: page.filtered.length > 0
                        Repeater {
                            model: page.filtered
                            delegate: EntryRow {
                                required property var modelData
                                entry: modelData
                                onEditRequested: page.draft = Object.assign({}, modelData)
                                onDeleteRequested: if (page.addressBook)
                                    page.addressBook.remove(modelData.id)
                            }
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 16; Layout.fillWidth: true }
        }
    }

    component DraftForm: ColumnLayout {
        property var draft: null
        signal draftUpdated(var draft)
        spacing: 10
        Layout.fillWidth: true

        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text { text: qsTr("Label"); color: Theme.fgMuted; font.pixelSize: 12 }
                TextField {
                    Layout.fillWidth: true
                    text: draft ? draft.label : ""
                    placeholderText: qsTr("e.g. Exchange deposit")
                    color: Theme.fg
                    onTextChanged: if (draft) {
                        var d = Object.assign({}, draft)
                        d.label = text
                        draftUpdated(d)
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Text { text: qsTr("Type"); color: Theme.fgMuted; font.pixelSize: 12 }
                ComboBox {
                    Layout.fillWidth: true
                    model: [
                        { value: "send", label: qsTr("Send to") },
                        { value: "receive", label: qsTr("Receive at") }
                    ]
                    textRole: "label"
                    currentIndex: draft && draft.category === "receive" ? 1 : 0
                    onActivated: function(i) {
                        if (!draft) return
                        var d = Object.assign({}, draft)
                        d.category = model[i].value
                        draftUpdated(d)
                    }
                }
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: qsTr("Address"); color: Theme.fgMuted; font.pixelSize: 12 }
            TextField {
                Layout.fillWidth: true
                text: draft ? draft.address : ""
                placeholderText: "VTDns…"
                font.family: Theme.monoFamily
                color: Theme.fg
                onTextChanged: if (draft) {
                    var d = Object.assign({}, draft)
                    d.address = text
                    draftUpdated(d)
                }
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: qsTr("Notes (optional)"); color: Theme.fgMuted; font.pixelSize: 12 }
            TextArea {
                Layout.fillWidth: true
                text: draft ? draft.notes : ""
                placeholderText: ""
                color: Theme.fg
                wrapMode: TextArea.Wrap
                onTextChanged: if (draft) {
                    var d = Object.assign({}, draft)
                    d.notes = text
                    draftUpdated(d)
                }
            }
        }
    }

    component EntryRow: Rectangle {
        property var entry
        signal editRequested()
        signal deleteRequested()

        Layout.fillWidth: true
        radius: Theme.radiusMd
        color: Qt.rgba(Theme.bgSubtle.r, Theme.bgSubtle.g, Theme.bgSubtle.b, 0.4)
        border.color: Theme.border
        border.width: 1
        implicitHeight: inner.implicitHeight + 20

        RowLayout {
            id: inner
            anchors.fill: parent
            anchors.margins: 12
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                RowLayout {
                    spacing: 8
                    Text {
                        text: entry.label && entry.label.length ? entry.label : qsTr("(no label)")
                        color: Theme.fg
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }
                    Badge { text: entry.category; tone: "neutral" }
                }
                Text {
                    text: entry.address
                    color: Theme.fgMuted
                    font.family: Theme.monoFamily
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
                Text {
                    visible: entry.notes && entry.notes.length > 0
                    text: entry.notes
                    color: Theme.fgSubtle
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }

            RowLayout {
                spacing: 4
                AppButton {
                    text: qsTr("Edit")
                    variant: "ghost"
                    size: "sm"
                    onClicked: editRequested()
                }
                AppButton {
                    text: qsTr("Delete")
                    variant: "ghost"
                    size: "sm"
                    onClicked: deleteRequested()
                }
            }
        }
    }
}
