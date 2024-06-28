mod bfs;
mod board;
mod encoder;
mod loader;

fn main() {
    let (mut board, state) = loader::load_inputs();
    board.show();

    let timer = std::time::Instant::now();

    let step_timer = std::time::Instant::now();
    board.simplify();
    eprintln!("Simplify the board tooks {:?}", step_timer.elapsed());

    board.show();

    let step_timer = std::time::Instant::now();
    let actions = if let Some(result_state) = bfs::solve(&board, &state) {
        result_state.get_actions()
    } else {
        String::from("")
    };
    eprintln!("Finding the solution tooks {:?}", step_timer.elapsed());

    let step_timer = std::time::Instant::now();
    let output = encoder::encode_actions(&actions);
    eprintln!("Encoding the solution tooks {:?}", step_timer.elapsed());

    eprintln!("Total Time: {:?}", timer.elapsed());

    print!("{}", output)
}