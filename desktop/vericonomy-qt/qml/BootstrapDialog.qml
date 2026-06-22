import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import com.vericonomy.verium

Dialog {
    id: dialog
    modal: true
    focus: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.NoButton
    width: Math.min(520, parent ? parent.width - 48 : 520)

    property string coin: "verium"
    property var bootstrap
    property var parseJson
    property var node

    readonly property string displayName: coin === "vericoin" ? qsTr("Vericoin") : qsTr("Verium")
    readonly property string cdnBase: coin === "vericoin"
        ? "https://files.vericonomy.com/vrc/bootstrap"
        : "https://files.vericonomy.com/vrm/bootstrap"
    readonly property var progress: bootstrap ? parseJson(bootstrap.progressJson, null) : null
    readonly property var bootstrapResult: bootstrap ? parseJson(bootstrap.resultJson, null) : null
    readonly property bool running: bootstrap && bootstrap.loading
    readonly property bool succeeded: bootstrapResult && bootstrapResult.success === true && !running
    readonly property bool failed: bootstrap && !running && bootstrap.lastMessage.length > 0
        && !succeeded && (progress ? progress.phase === "error" : true)
    readonly property bool cancelled: progress && progress.phase === "cancelled"
    readonly property bool canCancel: running && progress
        && (progress.cancellable === true
            || progress.phase === "stopping"
            || progress.phase === "resolving"
            || progress.phase === "downloading")

    property string selectedLocalPath: ""

    FileDialog {
        id: zipDialog
        title: qsTr("Choose %1 bootstrap zip").arg(dialog.displayName)
        nameFilters: ["Zip archive (*.zip)", "All files (*)"]
        onAccepted: {
            var path = selectedFile.toString()
            if (path.startsWith("file://"))
                path = decodeURIComponent(path.slice(7))
            dialog.selectedLocalPath = path
            if (dialog.bootstrap)
                dialog.bootstrap.importBootstrap(path)
        }
    }

    background: Rectangle {
        radius: Theme.radiusLg
        color: Theme.bgPanel
        border.color: Theme.border
        border.width: 1
    }

    header: Rectangle {
        height: 52
        color: "transparent"
        Text {
            anchors.left: parent.left
            anchors.leftMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Import chain bootstrap")
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 16
            font.weight: Font.DemiBold
        }
    }

    contentItem: ColumnLayout {
        spacing: 14
        width: dialog.width - 40

        Text {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
            text: qsTr("Imports the official %1 bootstrap archive into your data directory, replacing existing blocks/ and chainstate/. Downloads from %2 or uses a local zip if found.")
                .arg(displayName).arg(cdnBase)
        }

        Text {
            visible: selectedLocalPath.length > 0 && !running && !succeeded
            Layout.fillWidth: true
            text: qsTr("Selected: ") + selectedLocalPath
            color: Theme.fgSubtle
            font.family: Theme.monoFamily
            font.pixelSize: 10
            elide: Text.ElideMiddle
        }

        BootstrapProgressPanel {
            visible: running
            Layout.fillWidth: true
            progress: dialog.progress
            fallbackMessage: qsTr("Stopping node and preparing bootstrap…")
        }

        Rectangle {
            visible: cancelled && !running
            Layout.fillWidth: true
            radius: Theme.radiusMd
            color: Theme.bgSubtle
            border.color: Theme.border
            implicitHeight: cancelledText.implicitHeight + 16
            Text {
                id: cancelledText
                anchors.fill: parent
                anchors.margins: 10
                text: qsTr("Bootstrap import was cancelled. Your existing chain data was not replaced.")
                color: Theme.fgMuted
                font.family: Theme.fontFamily
                font.pixelSize: 11
                wrapMode: Text.Wrap
            }
        }

        Rectangle {
            visible: succeeded
            Layout.fillWidth: true
            radius: Theme.radiusMd
            color: Theme.success
            opacity: 0.12
            border.color: Theme.success
            implicitHeight: successText.implicitHeight + 16
            Text {
                id: successText
                anchors.fill: parent
                anchors.margins: 10
                text: bootstrap ? bootstrap.lastMessage : ""
                color: Theme.success
                font.family: Theme.fontFamily
                font.pixelSize: 11
                wrapMode: Text.Wrap
            }
        }

        Rectangle {
            visible: failed && !cancelled
            Layout.fillWidth: true
            radius: Theme.radiusMd
            color: Theme.danger
            opacity: 0.12
            border.color: Theme.danger
            implicitHeight: failText.implicitHeight + 16
            Text {
                id: failText
                anchors.fill: parent
                anchors.margins: 10
                text: bootstrap ? bootstrap.lastMessage : ""
                color: Theme.danger
                font.family: Theme.fontFamily
                font.pixelSize: 11
                wrapMode: Text.Wrap
            }
        }
    }

    footer: RowLayout {
        spacing: 10
        Item { Layout.fillWidth: true }
        AppButton {
            text: canCancel ? qsTr("Cancel download") : (succeeded || failed || cancelled ? qsTr("Close") : qsTr("Cancel"))
            variant: "secondary"
            size: "sm"
            enabled: !running || canCancel
            onClicked: {
                if (canCancel && bootstrap) {
                    bootstrap.cancel()
                } else {
                    dialog.close()
                }
            }
        }
        AppButton {
            visible: !running && !succeeded && !cancelled
            text: qsTr("Choose local zip…")
            variant: "secondary"
            size: "sm"
            onClicked: zipDialog.open()
        }
        AppButton {
            visible: failed && !cancelled
            text: qsTr("Retry")
            size: "sm"
            onClicked: if (bootstrap) bootstrap.importBootstrap(selectedLocalPath)
        }
        AppButton {
            visible: !running && !succeeded && !cancelled
            text: running ? qsTr("Bootstrapping…") : qsTr("Start bootstrap")
            size: "sm"
            enabled: bootstrap && !running
            onClicked: if (bootstrap) bootstrap.importBootstrap("")
        }
    }

    Connections {
        target: dialog.bootstrap
        function onBootstrapCompleted(ok) {
            if (ok && dialog.node)
                dialog.node.refresh()
        }
    }
}
