import QtQuick
import com.vericonomy.verium

// Opens a transaction on the Vericonomy explorer (Tauri ExplorerLink parity).
Text {
    id: link
    property string coin: "verium"
    property string txid: ""
    property string label: qsTr("View")

    readonly property string chainPath: coin === "vericoin" ? "vrc" : "vrm"
    readonly property string url: txid.length > 0
        ? "https://explorer.vericonomy.com/" + chainPath + "/tx/" + txid
        : ""

    text: label
    color: Theme.accent
    font.family: Theme.fontFamily
    font.pixelSize: 12
    font.underline: ma.containsMouse

    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: if (link.url.length > 0) HostLinks.open(link.url)
    }
}
