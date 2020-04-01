use std::time::Instant;

use ddup::algorithm;

fn main() {
    let i = Instant::now();
    algorithm::doit();
    println!("Finished in {} seconds", i.elapsed().as_secs_f32());
}