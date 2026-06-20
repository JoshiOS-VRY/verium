#include <QFontDatabase>

extern "C" void verium_qt_load_fonts()
{
    QFontDatabase::addApplicationFont(
        ":/qt/qml/com/vericonomy/verium/assets/fonts/Inter-Regular.ttf");
    QFontDatabase::addApplicationFont(
        ":/qt/qml/com/vericonomy/verium/assets/fonts/Inter-SemiBold.ttf");
}
