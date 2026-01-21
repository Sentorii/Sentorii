#![forbid(unsafe_code)]

mod manifest;
mod error;
mod plugin;
mod loader;

use crate::loader::Loader;
use sentorii_pdk::run_plugin_with_init;

fn main() {
    run_plugin_with_init(Loader::load);
}
