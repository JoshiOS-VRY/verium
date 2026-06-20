import QtQuick
import QtQuick.Layouts
import com.vericonomy.verium

// Label/value pair used in stat grids (mirrors DashboardHero MiniStat).
ColumnLayout {
    property string label: ""
    property string value: "—"
    property color valueColor: Theme.fg
    spacing: 2

    Text {
        text: label.toUpperCase()
        color: Theme.fgSubtle
        font.family: Theme.fontFamily
        font.pixelSize: 10
        font.weight: Font.Medium
        font.letterSpacing: 0.5
    }
    Text {
        text: value
        color: valueColor
        font.family: Theme.fontFamily
        font.pixelSize: 15
        font.weight: Font.DemiBold
        elide: Text.ElideRight
        Layout.fillWidth: true
    }
}
