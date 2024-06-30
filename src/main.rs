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

    let actions = match solve(&board, &state) {
        Some(state) => {
            eprintln!("Finding the solution tooks {:?} ", step_timer.elapsed());
            state.get_actions().clone()
        }
        None => {
            eprintln!("No solution found");
            String::new()
        }
    };

    let step_timer = std::time::Instant::now();
    let encoded = encode_actions(&actions);
    eprintln!("Encoding the solution tooks {:?}", step_timer.elapsed());

    eprintln!("Total Time: {:?}", timer.elapsed());

    println!("{}", encoded)
}
