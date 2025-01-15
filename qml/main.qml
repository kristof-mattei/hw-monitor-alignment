import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12
import QtQuick.Window 2.12


// This must match the qml_uri and qml_version
// specified with the #[cxx_qt::qobject] macro in Rust.
import hw_monitor_alignment 1.0

ApplicationWindow {
// Window {

    title: qsTr("HwMonitorAlignment")
    visible: true
    // flags: Qt.Window
    //        | Qt.WindowMinimizeButtonHint
    //        | Qt.WindowCloseButtonHint
    //        | Qt.CustomizeWindowHint
    //        | Qt.MSWindowsFixedSizeDialogHint

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

    // ColumnLayout {
    //     id: columnLayout
    //     anchors.fill: parent

    //     Rectangle {
    //         color: "#87d4d4"
    //         Layout.fillHeight: true
    //         Layout.fillWidth: true
    //     }

    //     // GroupBox {
    //     //     title: "Monitor Setup Information"
    //     //     Layout.fillWidth: true
    //     //     //                 anchors.fill: parent

    //     //     Layout.alignment: Qt.AlignHCenter | Qt.AlignBottom


    //     //     RowLayout {
    //     //         uniformCellSizes: true
    //     //         anchors.fill: parent

    //     //         GroupBox {
    //     //             Layout.fillWidth: true

    //     //             title: "Monitor Information"

    //     //             Text {
    //     //                 text: "FOO TE2222XT"
    //     //             }
    //     //         }
    //     //         GroupBox {
    //     //             Layout.fillWidth: true

    //     //             title: "Monitor Information"
    //     //             Text {
    //     //                 text: "bar text"
    //     //             }
    //     //         }
    //     //         GroupBox {
    //     //             Layout.fillWidth: true

    //     //             title: "Monitor Information"
    //     //             Text {
    //     //                 text: "bar text"
    //     //             }
    //     //         }

    //     //     }
    //     // }

    //     DialogButtonBox {
    //         Layout.bottomMargin: 6

    //         Layout.alignment: Qt.AlignHCenter | Qt.AlignBottom

    //         contentItem: RowLayout {  }
    //         contentWidth: 980



    //         Button {
    //             text: "Adjust"


    //             // width: implicitWidth
    //             // height: implicitHeight
    //             onClicked: {
    //                 console.log("Adjust clicked")
    //             }

    //         }


    //         Button {
    //             text: "Accept"
    //             // width: implicitWidth
    //             // height: implicitHeight
    //             // anchors.right: closeButton4.left
    //             onClicked: {
    //                 console.log("Accept clicked")
    //             }


    //         }

    //         Button {
    //             id: closeButton4
    //             text: qsTr("Close")
    //             // width: implicitWidth
    //             // height: implicitHeight
    //             // anchors.right: parent.right
    //             Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
    //             onClicked: {
    //                 console.log("Close clicked")
    //             }
    //         }



    //     }



    // }

}




