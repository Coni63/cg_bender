use crate::board::{Board, Cell, State};
use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub fn load_inputs() -> (Board, State) {
    let mut board = Board::new();
    let mut state = State::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let _width = parse_input!(inputs[0], i32);
    let height = parse_input!(inputs[1], i32);

    // load the board
    for row in 0..height as usize {
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        eprintln!("{}", input_line);
        for (col, letter) in input_line.trim_matches('\n').chars().enumerate() {
            eprintln!("{}", letter);
            match letter {
                '#' => board.set_cell(col, row, Cell::Wall),
                '.' => board.set_cell(col, row, Cell::Empty),
                '+' => {
                    state.add_garbage_ball(row * 21 + col);
                    board.set_cell(col, row, Cell::Empty);
                }
                _ => (),
            };
        }
    }

    // load the start and target positions
    input_line.clear();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let start_x = parse_input!(inputs[0], i32);
    let start_y = parse_input!(inputs[1], i32);

    input_line.clear();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let target_x = parse_input!(inputs[0], i32);
    let target_y = parse_input!(inputs[1], i32);

    board.set_start(start_x as usize, start_y as usize);
    board.set_target(target_x as usize, target_y as usize);

    // load the magnetic fields
    input_line.clear();
    io::stdin().read_line(&mut input_line).unwrap();
    let switch_count = parse_input!(input_line, i32);
    for i in 0..switch_count as usize {
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let switch_x = parse_input!(inputs[0], usize);
        let switch_y = parse_input!(inputs[1], usize);
        let block_x = parse_input!(inputs[2], usize);
        let block_y = parse_input!(inputs[3], usize);
        let initial_state = parse_input!(inputs[4], i32); // 1 if blocking, 0 otherwise

        board.set_cell(switch_x, switch_y, Cell::Switch(i));
        board.set_cell(block_x, block_y, Cell::MagneticField(i));
        state.set_magnetic_field(i, initial_state == 1);
    }

    (board, state)
}
