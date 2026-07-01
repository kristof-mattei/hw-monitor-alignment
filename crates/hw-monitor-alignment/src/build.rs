fn main() {
    windows_reactor_setup::as_framework_dependent();

    let output_path = "src/bindings.rs";

    let args = [
        "--in",
        "../../winmd",
        "--out",
        output_path,
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

    let status = Command::new("rustfmt")
        .arg(output_path)
        .status()
        .expect("Failed to execute rustfmt process");

    if !status.success() {
        println!(
            "cargo:warning=Failed to format {}: rustfmt exited with {:?}",output_path
            status.code()
        );
    }
}
