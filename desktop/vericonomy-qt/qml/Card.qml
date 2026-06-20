import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Panel surface — sizes from child layout content (never zero-height overflow).
Rectangle {
    id: card
    default property alias content: contentLayout.data
    property int padding: 20

    radius: Theme.radiusXl
    color: Theme.bgPanel
    border.color: Theme.border
    border.width: 1

    implicitHeight: contentLayout.implicitHeight + padding * 2

    ColumnLayout {
        id: contentLayout
        x: card.padding
        y: card.padding
        width: card.width > card.padding * 2 ? card.width - card.padding * 2 : implicitWidth
        height: card.height > card.padding * 2 ? card.height - card.padding * 2 : implicitHeight
    }
}
