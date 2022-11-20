fn main() {
    if std::env::var("TARGET").unwrap_or_default().contains("ios") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        println!("cargo:rerun-if-changed=QrCodeScanner.m");
        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=qrcodescanner");
        println!("cargo:rustc-link-search=native={}", out_dir);

        //println!("cargo:rustc-link-lib=framework=MLKitVision");
        //println!("cargo:rustc-link-lib=framework=MLKitBarcodeScanning");
        //println!("cargo:rustc-link-search=framework=/home/dvc/rosetta-wallet/dioxus-wallet/swift/Pods/MLKitBarcodeScanning/Frameworks");
        //println!("cargo:rustc-link-search=framework=/home/dvc/rosetta-wallet/dioxus-wallet/swift/Pods/MLKitVision/Frameworks");
        cc::Build::new()
            .cargo_metadata(false)
            .flag("-fmodules")
            //.link_lib_modifier("+whole-archive")
            //.cpp_link_stdlib("stdc++")
            //.include("swift/Pods/Headers/Public")
            .include("swift/Pods/MLKitVision/Frameworks/MLKitVision.framework/Headers")
            .include(
                "swift/Pods/MLKitBarcodeScanning/Frameworks/MLKitBarcodeScanning.framework/Headers",
            )
            .file("QrCodeScanner.m")
            .compile("qrcodescanner");
    }
}
