use hecs::World;

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}


#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

fn main() {
    let mut world = hecs::World::new();

    let start = std::time::Instant::now();

    for i in 0..10000 {
        let entity = world.spawn((i, Position { x: i as f32, y: i as f32 }));

        if i % 2 == 0 {
            world.insert_one(entity, Velocity { x: i as f32, y: i as f32 });
        }
    }

    let add_time_taken = start.elapsed();

    println!("Add time taken: {:?}", add_time_taken);

    for _ in 0..100 {
        run(&world);
    }
}

// 10000 components and iter with hecs
fn run(world: &World) {
   
    let start = std::time::Instant::now();

    let mut query_iter = world.query::<(&Position, &i32)>();

    let query_time_taken = start.elapsed();
    let start = std::time::Instant::now();

    let mut sum = 0;

    for (pos, i) in query_iter.iter() {
        sum += *i.1;
    }

    let iter_time_taken = start.elapsed();

    println!(
        "hecs, query: {:?}, iter: {:?}, sum: {}",
        query_time_taken, iter_time_taken, sum
    );
}