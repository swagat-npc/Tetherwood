mod engine;
mod game;

fn main() {
    // env_logger::init();
    env_logger::Builder::from_default_env()
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Off)
        .init();
    engine::run();
}
