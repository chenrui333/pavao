use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    match pkg_config::find_library("smbclient") {
        Ok(_) => {
            if cfg!(target_os = "macos") {
                println!("cargo:rustc-flags=-L /usr/local/lib -l smbclient");
            } else {
                println!("cargo:rustc-flags=-l smbclient");
            }
        }
        Err(e) => {
            println!(
                "error: SMB Client library not found! Probably libsmbclient is not installed."
            );
            panic!("{}", e);
        }
    };
}
