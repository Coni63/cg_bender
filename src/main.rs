mod bfs;
mod board;
mod encoder;
mod loader;

use bfs::solve;
use encoder::encode_actions;
use loader::load_inputs;

fn main() {
    let (mut board, mut state) = load_inputs();
    // board.show();

    let timer = std::time::Instant::now();

    // board.show(&state);

    let step_timer = std::time::Instant::now();
    board.simplify(&mut state);
    eprintln!("Simplify the board tooks {:?}", step_timer.elapsed());

    // board.show(&state);

    let step_timer = std::time::Instant::now();
    let states = solve(&board, &state);
    eprintln!("Finding the solution tooks {:?}", step_timer.elapsed());

    let step_timer = std::time::Instant::now();
    let mut shortest_path = String::new();
    let mut min_dist = 1000;
    for state in states.iter() {
        let actions = state.get_actions();
        let encoded = encode_actions(&actions);
        if encoded.len() < min_dist {
            min_dist = encoded.len();
            shortest_path = encoded;
        }
    }
    eprintln!("Encoding the solution tooks {:?}", step_timer.elapsed());

    eprintln!("Total Time: {:?}", timer.elapsed());

    println!("{}", shortest_path)
}
