fn main() {
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("assets/windows_manifest.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}