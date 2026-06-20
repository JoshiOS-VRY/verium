import QtQuick
import QtQuick.Window
import com.vericonomy.verium

// Application window hosting the desktop shell.
Window {
    id: root
    width: 1180
    height: 760
    minimumWidth: 920
    minimumHeight: 600
    visible: true
    title: qsTr("Vericonomy Wallet")
    color: Theme.bg

    AppShell {
        anchors.fill: parent
    }
}
