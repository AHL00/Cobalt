use cobalt::ecs::{component::Component, World};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Position {
    x: f32,
    y: f32,
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {}
impl Component for Position {}

fn main() {
    for _ in 0..100 {
        run();
    }
}

fn run() {
    let mut world = World::with_capacity(10000);

    let start = std::time::Instant::now();

    for i in 0..10000 {
        let entity = world.create_entity();

        world.add_component(entity, i).unwrap();

        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Velocity {
                        x: i as f32,
                        y: i as f32,
                    },
                )
                .unwrap();
        }
    }

    let add_time_taken = start.elapsed();
    let start = std::time::Instant::now();

    let query_iter = world.query::<(Position, i32)>().unwrap();

    let query_creation_time = start.elapsed();
    let start = std::time::Instant::now();

    let mut sum = 0;
    for (e, (pos, int)) in query_iter {
        sum += *int;
    }

    let query_time = start.elapsed();

    println!(
        "Add time taken: {:?}\nQuery creation time: {:?}\nQuery time: {:?}\nSum: {}",
        add_time_taken, query_creation_time, query_time, sum
    );
}
