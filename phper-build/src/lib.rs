// Copyright (c) 2022 PHPER Framework Team
// PHPER is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2. You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

#![warn(rust_2018_idioms, missing_docs)]
#![warn(clippy::dbg_macro)]
#![doc = include_str!("../README.md")]

use bindgen::Builder;
use phper_sys::*;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

/// Register all php build relative configure parameters, used in `build.rs`.
pub fn register_all() {
    register_link_args();
    register_configures();
}

/// Register useful rust cfg for project using phper.
pub fn register_configures() {
    // versions
    println!(
        "cargo:rustc-cfg=phper_major_version=\"{}\"",
        PHP_MAJOR_VERSION
    );
    println!(
        "cargo:rustc-cfg=phper_minor_version=\"{}\"",
        PHP_MINOR_VERSION
    );
    println!(
        "cargo:rustc-cfg=phper_release_version=\"{}\"",
        PHP_RELEASE_VERSION
    );

    if PHP_DEBUG != 0 {
        println!("cargo:rustc-cfg=phper_debug");
    }

    if USING_ZTS != 0 {
        println!("cargo:rustc-cfg=phper_zts");
    }
}

/// Register link arguments for os-specified situation.
pub fn register_link_args() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-undefined");
        println!("cargo:rustc-link-arg=dynamic_lookup");
    }
}

/// Includes php bindings for function/method arguments
pub fn generate_php_function_args<P: AsRef<Path>>(output_dir: P, dirs: &[P]) {
    for dir in dirs {
        let walk = WalkDir::new(dir).max_depth(12).follow_links(false);
 
        walk.into_iter()
            .filter_entry(|entry| {
                entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("h"))
            })
            .for_each(|item| println!("{}", item.unwrap().path().display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_function_args() {
        generate_php_function_args(".", &["."]);
    }
}
