use cobalt::runtime::App;

struct Game {}

impl App for Game {

}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut game_app = Game {};

    let mut runner = cobalt::runtime::engine::EngineRunner::builder()
        .with_app(&mut game_app)
        .build();

    runner.run().unwrap();
}
