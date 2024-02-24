import QtQuick.Controls
import QtQuick.Window

// This must match the qml_uri and qml_version
// specified with the #[cxx_qt::qobject] macro in Rust.
import hw_monitor_alignment 1.0

Window {
    title: qsTr("Hello App")
    visible: true
    height: 480
    width: 640
    color: "#e4af79"

    Hello {
        id: hello
    }

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        /* space between widget */
        spacing: 10

        Button {
            text: "Say Hello!"
            onClicked: hello.sayHello()
        }
    }
}
