// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_msvc_rc() -> Option<PathBuf> {
    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return Some(PathBuf::from("rc.exe"));
    }
    let kits_root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if let Ok(entries) = fs::read_dir(kits_root) {
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path().join("x64").join("rc.exe"))
            .filter(|p| p.is_file())
            .collect();
        candidates.sort();
        if let Some(latest) = candidates.pop() {
            return Some(latest);
        }
    }
    None
}

fn find_gnu_windres() -> Option<&'static str> {
    for candidate in ["windres", "x86_64-w64-mingw32-windres"] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return Some(candidate);
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/calclab.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let ico_path = manifest_dir.join("assets").join("calclab.ico");
    let ico_rc_path = ico_path.to_string_lossy().replace('\\', "/");

    let rc_content = format!(
        r#"1 ICON "{ico}"

1 VERSIONINFO
FILEVERSION 0,5,0,0
PRODUCTVERSION 0,5,0,0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName", "CRV Science Network\0"
            VALUE "FileDescription", "Calc-Lab - Mathematical Mastery & Scientific Calculus Workbench\0"
            VALUE "FileVersion", "0.5.0\0"
            VALUE "InternalName", "calc-app\0"
            VALUE "LegalCopyright", "Copyright CRV Science Network (GPL-3.0-or-later)\0"
            VALUE "OriginalFilename", "calc-app.exe\0"
            VALUE "ProductName", "Calc-Lab\0"
            VALUE "ProductVersion", "0.5.0\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#,
        ico = ico_rc_path
    );

    let rc_path = out_dir.join("app.rc");
    fs::write(&rc_path, rc_content).expect("write app.rc");

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        if let Some(windres) = find_gnu_windres() {
            let res_obj = out_dir.join("app_res.o");
            let status = Command::new(windres)
                .arg("-i")
                .arg(&rc_path)
                .arg("-o")
                .arg(&res_obj)
                .arg("-O")
                .arg("coff")
                .status()
                .expect("invoke windres");
            if status.success() {
                println!("cargo:rustc-link-arg={}", res_obj.display());
            } else {
                println!(
                    "cargo:warning=windres failed with status {status}; skipping Windows resource embedding"
                );
            }
        } else {
            println!("cargo:warning=windres not found; skipping Windows resource embedding");
        }
    } else if target_env == "msvc" {
        if let Some(rc_exe) = find_msvc_rc() {
            let res_file = out_dir.join("app_res.res");
            let status = Command::new(rc_exe)
                .arg("/nologo")
                .arg("/fo")
                .arg(&res_file)
                .arg(&rc_path)
                .status()
                .expect("invoke rc.exe");
            if status.success() {
                println!("cargo:rustc-link-arg={}", res_file.display());
            } else {
                println!(
                    "cargo:warning=rc.exe failed with status {status}; skipping Windows resource embedding"
                );
            }
        } else {
            println!("cargo:warning=rc.exe not found; skipping Windows resource embedding");
        }
    }
}
