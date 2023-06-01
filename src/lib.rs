mod patchouli;
mod updater;
mod utils;

use log::{info, Level};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

///
/// initiation function
///
#[wasm_bindgen(start)]
pub fn wasm_main() {
    utils::set_panic_hook();
    init_log();
    info!("Updater Initialized");
}

#[cfg(feature = "debug-log")]
fn init_log() {
    console_log::init_with_level(Level::Trace).expect("error initializing log");
}

#[cfg(not(feature = "debug-log"))]
fn init_log() {
    console_log::init_with_level(Level::Info).expect("error initializing log");
}
