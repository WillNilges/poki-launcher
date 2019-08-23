import QtQuick 2.6
import QtQuick.Window 2.2
import Qt.labs.platform 1.1
import PokiLauncher 1.0

Window {
    id: window
    visible: apps_model.visible
    width: 500
    height: 500
    // maximumHeight: height
    // minimumHeight: height
    // maximumWidth: width
    // minimumWidth: width

    AppsModel {
        id: apps_model
    }

    SystemTrayIcon {
        visible: true
        icon.source: "firefox.png"

        onActivated: {
            window.show()
            window.raise()
            window.requestActivate()
        }

        menu: Menu {
            MenuItem {
                text: qsTr("Quit")
                onTriggered: Qt.quit()
            }
        }
    }

    title: qsTr("Poki Launcher")

    MainForm {
        anchors.fill: parent
    }
}
