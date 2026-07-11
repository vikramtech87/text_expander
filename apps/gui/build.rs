fn main() {
    slint_build::compile("src/appwindow.slint").unwrap();
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile(
            "assets/windows_manifest.rc",
            embed_resource::NONE
        ).manifest_optional().unwrap();
    }
}