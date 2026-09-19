fn main() {
    #[cfg(unix)]
    {
        use build_rs::input::{cargo_cfg_target_arch, cargo_cfg_unix, cargo_cfg_windows};

        if cargo_cfg_windows() {
            if cargo_cfg_target_arch() == "x86_64" {
                println!("cargo::rustc-link-search=/usr/x86_64-w64-mingw32/lib");
            } else if cargo_cfg_target_arch() == "i686" {
                println!("cargo::rustc-link-search=/usr/i686-w64-mingw32/lib");
            }

            // All of these are used somewhere so this is the only way to make it link unfortunately /shrug
            println!("cargo::rustc-link-lib=strmiids");
            println!("cargo::rustc-link-lib=uuid");
            println!("cargo::rustc-link-lib=crypt32");
            println!("cargo::rustc-link-lib=ncrypt");
            println!("cargo::rustc-link-lib=ole32");
            println!("cargo::rustc-link-lib=bcrypt");
            println!("cargo::rustc-link-lib=avicap32");
            println!("cargo::rustc-link-lib=schannel");
            println!("cargo::rustc-link-lib=security");
            println!("cargo::rustc-link-lib=mincore");
            println!("cargo::rustc-link-lib=shcore");
        } else if cargo_cfg_unix() {
            if cargo_cfg_target_arch() == "aarch64" {
                println!("cargo::rustc-link-lib=png16");
            }
        }
    }
}
