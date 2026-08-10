mod engine;
mod game;

fn main() {
    // env_logger::init();
    env_logger::Builder::from_default_env()
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Off)
        .init();
    engine::run();
    // TODO: create a basic editor, that allows direct file manipulation through a GUI
    // It should have, drag and drop, changing size & position through mouse events
    // It should allow saving the file for basic default value manipulation
    // The scene will now be a file rather than a function call, to store default behaviour
    // Not a full fledged unity/godot-esq editor, just a minimal GUI
}
