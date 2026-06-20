import QtQuick
import com.vericonomy.verium

// Address book. Mirrors desktop/verium-app/src/pages/AddressBook.tsx.
PlaceholderPage {
    title: qsTr("Address book")
    subtitle: qsTr("Saved labels for sending & receiving")
    glyph: "\u2637"
    features: [
        qsTr("Labelled send/receive contacts"),
        qsTr("Address validation & coin scoping"),
        qsTr("Import / export"),
        qsTr("Quick-fill into the Send form")
    ]
}
