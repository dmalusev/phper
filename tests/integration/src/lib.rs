#![warn(rust_2018_idioms, clippy::dbg_macro, clippy::print_stdout)]

mod values;

use phper::{modules::Module, php_get_module};

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    // ...

    module
}
