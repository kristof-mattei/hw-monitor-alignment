#[cxx_qt::bridge]
pub mod qobject {

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[namespace = "my_object"]
        type Hello = super::HelloRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "sayHello"]
        pub fn say_hello(self: &Hello);
    }
}

#[derive(Default)]
pub struct HelloRust {}

impl qobject::Hello {
    pub fn say_hello(&self) {
        println!("Hello world!");
    }
}
