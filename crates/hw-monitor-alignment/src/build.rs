fn main() {
    windows_reactor_setup::as_framework_dependent();

    let args = [
        "--in",
        "../../winmd",
        "--out",
        "src/bindings.rs",
        "--implement",
        "Microsoft.UI.Xaml.IApplicationOverrides",
        "Microsoft.UI.Xaml.Markup.IXamlMetadataProvider",
        "--minimal",
        "--flat",
        "--filter",
        "Microsoft.UI.Xaml.ISizeChangedEventArgs::{get_NewSize}",
        "Microsoft.UI.Xaml.IFrameworkElement::{add_SizeChanged}",
    ];

    windows_bindgen::bindgen(args);
}
