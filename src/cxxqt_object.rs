#[allow(clippy::unnecessary_box_returns)]
#[cxx_qt::bridge]
pub mod my_object {

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Hello = super::HelloRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        pub fn say_hello(self: &Hello);
    }
}

#[derive(Default)]
pub struct HelloRust {}

impl my_object::Hello {
    pub fn say_hello(&self) {
        println!("Hello world!");
    }
}
