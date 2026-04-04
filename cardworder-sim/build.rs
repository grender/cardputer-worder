fn main() {
    // On Apple Silicon macOS, Homebrew installs to /opt/homebrew.
    // On Intel macOS, it installs to /usr/local.
    // Tell the linker where to find libSDL2.
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-search=/usr/local/lib");
    }
}
