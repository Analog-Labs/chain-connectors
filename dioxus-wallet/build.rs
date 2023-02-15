fn main() {
    if std::env::var("TARGET").unwrap_or_default().contains("ios") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        println!("cargo:rerun-if-changed=QrCodeScanner.m");
        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=qrcodescanner");
        println!("cargo:rustc-link-search=native={out_dir}");

        cc::Build::new()
            .cargo_metadata(false)
            .flag("-fmodules")
            .file("QrCodeScanner.m")
            .compile("qrcodescanner");
    }
}
