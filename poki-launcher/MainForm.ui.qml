import QtQuick 2.13
import QtQuick.Layouts 1.13

Rectangle {
    color: "#282a36"

	function run() {
		apps_model.run();
		input.clear();
		window.close();
	}

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 0
        spacing: 0

		Rectangle {
			id: input_box
			color: "#44475a"
			radius: 0
			Layout.preferredWidth: window.width
			Layout.preferredHeight: window.height * 0.1
			Layout.alignment: Qt.AlignHCenter

			TextInput {
				id: input
				focus: true
				color: "#f8f8f2"
				padding: 10
				anchors.verticalCenter: input_box.verticalCenter
				font.pixelSize: window.height * 0.1 * 0.4
				onTextChanged: apps_model.search(text)
				Keys.onUpPressed: apps_model.up()
				Keys.onDownPressed: apps_model.down()
				Keys.onReturnPressed: run()
				Keys.onEscapePressed: window.close()
			}
		}

        ListView {
            id: app_list
			Layout.alignment: Qt.AlignHCenter
			Layout.preferredWidth: window.width
			Layout.preferredHeight: window.height * 0.9
			interactive: false

			model: apps_model
			delegate: Item {
				height: app_list.height * 0.2
				width: window.width

				Rectangle {
					anchors.fill: parent
					anchors.topMargin: 1
					anchors.bottomMargin: 1
					id: item
					color: (uuid == apps_model.selected) ? "#44475a" : "#282a36"
					RowLayout {
						anchors.fill: parent

						Image {
							asynchronous: true
							Layout.preferredWidth: item.height * 0.8
							Layout.preferredHeight: item.height * 0.8
							Layout.alignment: Qt.AlignLeft
							fillMode: Image.PreserveAspectFit
							source: "file:///" + apps_model.get_icon(icon)
						}

						Text {
							Layout.alignment: Qt.AlignLeft
							color: "#f8f8f2"
							text: name
							font.pixelSize: item.height * 0.4
						}
					}
				}

				Rectangle {
					height: 1
					color: "#bd93f9"
					anchors {
						left: item.left
						right: item.right
						bottom: item.top
					}
				}
			}
        }
    }
}
