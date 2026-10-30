fn main() {
    // Embeds assets/icon.ico as icon resource #1 (tray icon + Explorer file icon).
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_optional()
        .expect("failed to compile app.rc (is the Windows SDK / rc.exe installed?)");
}
