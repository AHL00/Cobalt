use std::f32::consts::PI;

use cobalt::{
    components::{Camera, Renderable, Transform},
    ecs::Entity,
    graphics::window::WindowConfig,
    input::KeyCode,
    maths::Vec3,
    plugins::debug_gui::DebugGUIPlugin,
    renderer::{
        camera::{AspectRatio, Projection},
        renderables::MeshRenderable,
        Material, Mesh,
    },
    runtime::{
        engine::{EngineRunner, InitialEngineConfig},
        plugins::PluginBuilder,
        App,
    },
    types::resource::Resource,
};
use simple_logger::SimpleLogger;

struct Game {
    main_camera: Entity,
    model: Option<Entity>,
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
                    near: 0.1,
                    far: 100.0,
                },
            ),
        );

        engine
            .scene
            .world
            .add_component(cam_ent, Transform::with_position([0.0, 0.0, 0.0].into()));

        Self {
            main_camera: cam_ent,
            model: None,
        }
    }

    fn on_start(
        &mut self,
        engine: &mut cobalt::runtime::engine::Engine,
        _plugins: &mut cobalt::runtime::plugins::PluginManager,
    ) {
        let model_ent = engine.scene.world.create_entity();

        let mesh_asset_id = engine.assets().find_asset_by_name("dragon").unwrap();

        let mesh = engine.load_asset::<Mesh>(mesh_asset_id).unwrap();

        engine
            .scene
            .world
            .add_component(model_ent, Renderable::Mesh(MeshRenderable::new(mesh)));

        // engine
        //     .scene
        //     .world
        //     .add_component(plane_ent, Renderable::Plane(Plane::new()));

        let mut transform = Transform::with_position([0.0, 0.0, 2.0].into());
        transform.rotate(transform.position(), [0.0, 0.0, 0.0].into());
        *transform.scale_mut() = Vec3::from([0.01, 0.01, 0.01]);
        engine.scene.world.add_component(model_ent, transform);

        let mat = Resource::new(Material::default(engine.graphics_arc()));

        // let texture_asset_id = engine
        //     .assets()
        //     .find_asset_by_name("logo_compressed")
        //     .unwrap();

        // println!("texture_asset_id: {:?}", texture_asset_id);

        // let texture = engine
        //     .load_asset::<TextureAsset<{ TextureType::RGBA8UnormSrgb }>>(texture_asset_id)
        //     .unwrap();

        // mat.borrow_mut().set_albedo(None, Some(texture)).unwrap();
        mat.borrow_mut()
            .set_albedo(Some([1.0, 1.0, 1.0, 1.0]), None)
            .unwrap();
        engine.scene.world.add_component(model_ent, mat);

        self.model = Some(model_ent);
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
                .query_entity_mut::<Transform>(self.main_camera)
                .expect("Transform not found.");

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
            let model = _engine
                .scene
                .world
                .query_entity_mut::<Transform>(self.model.unwrap())
                .unwrap();
            model.yaw(0.2 * dt);
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
