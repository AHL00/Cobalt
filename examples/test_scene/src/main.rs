use std::f32::consts::PI;

use cobalt::{
    components::{Camera, Renderable, Transform},
    ecs::Entity,
    graphics::{wgpu::PresentMode, window::WindowConfig},
    input::KeyCode,
    maths::Vec3,
    plugins::debug_gui::DebugGUIPlugin,
    renderer::{
        camera::{AspectRatio, Projection}, deferred::Material, renderables::MeshRenderable, Mesh
    },
    runtime::{
        engine::{EngineRunner, InitialEngineConfig},
        plugins::PluginBuilder,
        App,
    },
    types::resource::Resource,
};
use simple_logger::SimpleLogger;

// asset name, scale
#[rustfmt::skip]
const MODELS: [(&'static str, f32); 4] = [
    ("teapot", 0.04), 
    ("dragon", 0.002), 
    ("bunny", 1.1),
    ("beetle", 0.3)
];

struct Game {
    main_camera: Entity,
    models: Vec<Entity>,
}

impl App for Game {
    fn initialize(engine: &mut cobalt::runtime::engine::Engine) -> Self
    where
        Self: Sized,
    {
        let cam_ent = engine.scene.world.create_entity();

        engine.scene.world.add_component(
            cam_ent,
            Camera::new(
                true,
                Projection::Perspective {
                    fov: 70.0 * (PI / 180.0),
                    aspect: AspectRatio::Auto,
                    near: 0.001,
                    far: 100.0,
                },
            ),
        );

        engine
            .scene
            .world
            .add_component(cam_ent, Transform::with_position([0.0, 0.0, -1.0].into()));

        if engine
            .graphics()
            .available_present_modes()
            .contains(&PresentMode::Mailbox)
        {
            engine.graphics_mut().current_present_mode = PresentMode::Mailbox;
        } else if engine
            .graphics()
            .available_present_modes()
            .contains(&PresentMode::Fifo)
        {
            engine.graphics_mut().current_present_mode = PresentMode::Fifo;
        }

        Self {
            main_camera: cam_ent,
            models: vec![],
        }
    }

    fn on_start(
        &mut self,
        engine: &mut cobalt::runtime::engine::Engine,
        _plugins: &mut cobalt::runtime::plugins::PluginManager,
    ) {
        for (i, model) in MODELS.iter().enumerate() {
            let ent = engine.scene.world.create_entity();

            let mesh_asset_id = engine.assets().find_asset_by_name(model.0).unwrap();

            let mesh = engine.load_asset::<Mesh>(mesh_asset_id).unwrap();

            engine
                .scene
                .world
                .add_component(ent, 
                        Renderable::Mesh(MeshRenderable::new(mesh)));

            let mut transform = Transform::with_position(
                [(i as f32 - MODELS.len() as f32 / 2.0 + 0.5) * 0.5, 0.0, 0.0].into(),
            );

            *transform.scale_mut() = Vec3::from([model.1, model.1, model.1]);

            engine.scene.world.add_component(ent, transform);

            let mat = Resource::new(Material::default(engine.graphics_arc()));

            mat.borrow_mut()
                .set_albedo(Some([1.0, 1.0, 1.0, 1.0]), None)
                .unwrap();

            engine.scene.world.add_component(ent, mat);

            self.models.push(ent);
        }
    }

    fn on_update(
        &mut self,
        _engine: &mut cobalt::runtime::engine::Engine,
        _plugins: &mut cobalt::runtime::plugins::PluginManager,
        dt: f32,
    ) {
        {
            let transform = _engine
                .scene
                .world
                .query_entity_mut::<Transform>(self.main_camera);

            if let None = transform {
                return;
            }

            let transform = transform.unwrap();

            let keyboard = _engine.input.get_keyboard();

            let mut forwards = 0.0;
            let mut right = 0.0;
            let mut up = 0.0;

            if keyboard.is_key_down(KeyCode::KeyW) {
                forwards += 1.0;
            }

            if keyboard.is_key_down(KeyCode::KeyS) {
                forwards -= 1.0;
            }

            if keyboard.is_key_down(KeyCode::KeyA) {
                right -= 1.0;
            }

            if keyboard.is_key_down(KeyCode::KeyD) {
                right += 1.0;
            }

            if keyboard.is_key_down(KeyCode::Space) {
                up += 1.0;
            }

            if keyboard.is_key_down(KeyCode::ControlLeft) {
                up -= 1.0;
            }

            let forward_vector = transform.forward();
            let right_vector = transform.right();
            let up_vector = transform.up();

            let mut movement = forward_vector * forwards + right_vector * right + up_vector * up;

            if movement.mag() > 0.0 {
                movement.normalize();
            }

            transform.translate(movement * 0.5 * dt);

            let rotate_x = keyboard.is_key_down(KeyCode::ArrowUp) as i32
                - keyboard.is_key_down(KeyCode::ArrowDown) as i32;

            let rotate_y = keyboard.is_key_down(KeyCode::ArrowRight) as i32
                - keyboard.is_key_down(KeyCode::ArrowLeft) as i32;

            transform.pitch(rotate_x as f32 * dt);
            transform.yaw(rotate_y as f32 * dt);
        }

        {
            // Rotate model slowly about the y-axis
            for model in self.models.iter() {
                let m_transform = _engine.scene.world.query_entity_mut::<Transform>(*model);

                if let Some(m_transform) = m_transform {
                    m_transform.yaw(0.2 * dt);
                }
            }
        }
    }
}

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .init()
        .unwrap();

    let mut runner = EngineRunner::<Game>::builder()
        .with_plugins(vec![PluginBuilder {
            plugin: Box::new(DebugGUIPlugin::default()),
            run_priority: 0,
        }])
        .with_config(InitialEngineConfig {
            window_config: WindowConfig {
                title: "Test Scene".to_string(),
                size: (1280, 720),
            },
            ..Default::default()
        })
        .build();

    runner.run().unwrap();
}
