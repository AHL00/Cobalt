

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    cobalt::runtime::Engine::build().unwrap().run().unwrap();
}