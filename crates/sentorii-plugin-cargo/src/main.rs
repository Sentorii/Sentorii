#![forbid(unsafe_code)]

mod error;
mod loader;
mod manifest;
mod plugin;
mod validator;

use crate::loader::Loader;
use sentorii_pdk::run_plugin_with_init;

fn main() {
    run_plugin_with_init(Loader::load);
}
