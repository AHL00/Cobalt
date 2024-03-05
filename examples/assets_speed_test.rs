use rand::Rng;
use walkdir::WalkDir;

fn main() {
    let mut assets = Vec::new();

    // Load every source file as an asset
    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if path.is_file() && file_name.ends_with(".rs") {
            let asset = cobalt::assets::asset_server()
                .write()
                .load::<cobalt::assets::Text>(path).unwrap();

            assets.push(asset);
        }
    }

    let mut rng = rand::thread_rng();

    let access_pattern = (0..assets.len())
        .map(|_| rng.gen_range(0..assets.len()))
        .collect::<Vec<_>>();

    let start = std::time::Instant::now();

    let loop_count = 10000;

    let mut count = 0;

    for _ in 0..loop_count {
        for i in access_pattern.iter() {
            let asset_ref = assets[*i].borrow();

            drop(asset_ref);

            count += 1;
        }
    }

    let time_taken = start.elapsed();

    println!("Assets: {}", assets.len());
    println!("Loops: {}", loop_count);
    println!("Accesses: {}", count);
    println!("Time taken: {:?}", time_taken);
    println!(
        "Time per access: {:?}ns",
        time_taken.as_nanos() as f32 / count as f32
    );
}
