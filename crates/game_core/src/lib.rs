mod components;
mod events;
mod pathfinding;
mod resources;
mod systems;
mod world;

use wasm_bindgen::prelude::*;

pub use world::{GameWorld, create_game_world};

#[wasm_bindgen(js_name = getProgramName)]
pub fn get_program_name() -> String {
    "Hello, My Dear World".into()
}

#[wasm_bindgen(js_name = getCoreBuildInfo)]
pub fn get_core_build_info() -> String {
    format!(
        r#"{{"layer":"core","version":"{}"}}"#,
        env!("CORE_BUILD_VERSION")
    )
}
