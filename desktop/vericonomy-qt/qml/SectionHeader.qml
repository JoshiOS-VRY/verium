import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Title + optional right-aligned slot, used at the top of cards/sections.
RowLayout {
    property string title: ""
    property string subtitle: ""
    default property alias trailing: trailingSlot.data
    spacing: 12

    ColumnLayout {
        spacing: 2
        Layout.fillWidth: true
        Text {
            text: title
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 16
            font.weight: Font.DemiBold
        }
        Text {
            visible: subtitle.length > 0
            text: subtitle
            color: Theme.fgMuted
            font.family: Theme.fontFamily
            font.pixelSize: 12
        }
    }

    Item {
        id: trailingSlot
        Layout.preferredWidth: childrenRect.width
        Layout.preferredHeight: childrenRect.height
        Layout.alignment: Qt.AlignVCenter
    }
}
