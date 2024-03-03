use cobalt::{
    assets::asset_server,
    engine::{Application, Engine},
    graphics::{texture::Texture, winit_window},
    input::ButtonState,
    renderer::{
        camera::{Camera, Projection},
        sprite::Sprite,
    },
    transform::Transform,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    let res = cobalt::engine::run(App::new());

    if let Err(e) = res {
        log::error!("Exited with error: {}", e);
    }

    log::info!("Exited");
}

struct App {
    last_debug_print: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            last_debug_print: std::time::Instant::now(),
        }
    }
}

impl Application for App {
    fn init(&mut self, engine: &mut Engine) {
        log::info!("Initializing app");

        asset_server().write().set_assets_dir("assets");

        let texture = asset_server().write().load::<Texture>("texture.png");

        for x in -50..50 {
            for y in -0..100 {
                let ent = engine.scene.world.create_entity();

                let transform = Transform::with_position([x as f32 * 1.5, y as f32 * 1.5, 0.0].into());

                engine.scene.world.add_component(ent, transform);
                engine
                    .scene
                    .world
                    .add_component(ent, Sprite::new(texture.clone()));
            }
        }

        let cam_ent = engine.scene.world.create_entity();
        engine.scene.world.add_component(
            cam_ent,
            Camera::new(
                true,
                Projection::Perspective {
                    fov: 70.0,
                    aspect: 16.0 / 9.0,
                    near: 0.1,
                    far: 100.0,
                },
            ),
        );

        let mut cam_transform = Transform::with_position([0.0, 0.0, 5.0].into());

        cam_transform.rotate_y(45.0);

        engine
            .scene
            .world
            .add_component(cam_ent, cam_transform);
    }

    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        if self.last_debug_print.elapsed().as_secs_f32() > 1.0 {
            let stats = &engine.stats;

            log::info!(
                "AVG FPS: {:?}, CPU: {:.2?}, GPU: {:.2?}",
                stats.average_fps(10),
                stats.cpu_render_time_history.last().unwrap(),
                stats.gpu_render_time_history.last().unwrap()
            );

            self.last_debug_print = std::time::Instant::now();
        }

        movement(engine, delta_time);

        if engine
            .input
            .keyboard
            .get_key_state(cobalt::input::KeyCode::F11)
            == &ButtonState::Pressed
        {
            log::info!("Toggled fullscreen");

            let fullscren = engine.window.winit.fullscreen();

            if let Some(fullscreen) = fullscren {
                engine.window.winit.set_fullscreen(None);
            } else {
                engine
                    .window
                    .winit
                    .set_fullscreen(Some(winit_window::Fullscreen::Borderless(None)));
            }
        }
    }
}

fn movement(engine: &mut Engine, delta_time: f32) {
    if engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::Escape)
        == &ButtonState::Pressed
    {
        engine.exit();
    }

    let query = engine
        .scene
        .world
        .query_mut::<(Camera, Transform)>()
        .unwrap();

    let mut left = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyA)
    {
        left = true;
    }

    let mut right = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyD)
    {
        right = true;
    }

    let mut up = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::ShiftLeft)
    {
        up = true;
    }

    let mut down = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::ControlLeft)
    {
        down = true;
    }

    let mut forward = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyW)
    {
        forward = true;
    }

    let mut backward = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyS)
    {
        backward = true;
    }

    let mut mouse_delta = engine.input.mouse.get_delta();

    let query = engine
        .scene
        .world
        .query_mut::<(Camera, Transform)>()
        .unwrap();

    for (entity, (_, transform)) in query {
        let multiplier = 2.5;

        if left {
            let pos = transform.position_mut();
            pos.x -= multiplier * delta_time;
        }

        if right {
            let pos = transform.position_mut();
            pos.x += multiplier * delta_time;
        }

        if up {
            let pos = transform.position_mut();
            pos.y += multiplier * delta_time;
        }

        if down {
            let pos = transform.position_mut();
            pos.y -= multiplier * delta_time;
        }

        if forward {
            let pos = transform.position_mut();
            pos.z -= multiplier * delta_time;
        }

        if backward {
            let pos = transform.position_mut();
            pos.z += multiplier * delta_time;
        }

        if down || up || left || right || forward || backward {
            // log::info!("Camera position: {:?}", pos);
        }
    }

    let mut left = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyJ)
    {
        left = true;
    }

    let mut right = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyL)
    {
        right = true;
    }

    let mut up = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyI)
    {
        up = true;
    }

    let mut down = false;
    if let ButtonState::Held { .. } = engine
        .input
        .keyboard
        .get_key_state(cobalt::input::KeyCode::KeyK)
    {
        down = true;
    }

    let query = engine
        .scene
        .world
        .query_mut::<(Sprite, Transform)>()
        .unwrap();

    for (entity, (sprite, transform)) in query {
        let multiplier = 2.5;

        if left {
            let pos = transform.position_mut();
            pos.x -= multiplier * delta_time;
        }

        if right {
            let pos = transform.position_mut();
            pos.x += multiplier * delta_time;
        }

        if up {
            let pos = transform.position_mut();
            pos.y += multiplier * delta_time;
        }

        if down {
            let pos = transform.position_mut();
            pos.y -= multiplier * delta_time;
        }

        if down || up || left || right {
            let pos = transform.position();
            // log::info!("Sprite position: {:?}", pos);
        }
    }
}

struct TestScript {}

impl cobalt::script::Script for TestScript {
    fn update(&self, engine: &mut Engine, entity: cobalt::ecs::Entity) {}

    fn on_load(&self, engine: &mut Engine, entity: cobalt::ecs::Entity) {
        log::info!("Load script");
    }
}
