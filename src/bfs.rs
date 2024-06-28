use std::collections::{HashMap, VecDeque};

use crate::board::{Board, Cell, State};

fn bfs(start: usize, target: usize, board: &Board, state: &State) -> Option<String> {
    let mut queue: VecDeque<(usize, String)> = VecDeque::new();
    let mut visited = vec![false; 441];
    let offset: [(i32, char); 4] = [(-1, 'L'), (1, 'R'), (-21, 'U'), (21, 'D')];

    queue.push_back((start, String::new()));
    loop {
        // eprintln!("Queue: {:?}", queue.len());
        match queue.pop_front() {
            None => return None,
            Some((current, path)) => {
                // eprintln!("Current: {} Path: {}", current, path);
                // check if we reached the target
                if current == target {
                    // eprintln!("Found the target: {}", path);
                    return Some(path);
                }

                // disregaard switch and magnetic field
                if (current != start) & (board.get_cell_idx(current) != &Cell::Empty) {
                    // eprintln!("Found a wall: {}", current);
                    continue;
                }

                // disregard balls
                if state.is_garbage_ball(current) {
                    // eprintln!("Found a ball: {}", current);
                    continue;
                }

                // memorize the current position
                if visited[current] {
                    // eprintln!("Already visited: {}", current);
                    continue;
                }
                visited[current] = true;

                // explore the neighbors
                for (o, dir) in offset.iter() {
                    let new_idx = (current as i32 + o) as usize;
                    // eprintln!("Old idx: {} + {} - New idx: {}", current, o, new_idx);
                    queue.push_back((new_idx, path.clone() + &dir.to_string()));
                }
            }
        }
    }
}

// Compute the shortest path between all pairs of start and target
// A start is either the start position, a magnetic field or a switch
// A target is either the target position, a magnetic field or a switch
// This allows after to have a way faster BFS
fn prepare(board: &Board, state: &State) -> HashMap<(usize, usize), String> {
    let mut G: HashMap<(usize, usize), String> = HashMap::new();

    let all_switches = board.get_all_switches();
    let all_magnetic_fields = board.get_all_magnetic_fields();
    let start = board.get_start();
    let target = board.get_target();

    let all_starts: Vec<usize> = [start]
        .iter()
        .chain(all_magnetic_fields.iter())
        .chain(all_switches.iter())
        .filter(|x| **x != 0)
        .cloned()
        .collect();
    let all_targets: Vec<usize> = [target]
        .iter()
        .chain(all_magnetic_fields.iter())
        .chain(all_switches.iter())
        .filter(|x| **x != 0)
        .cloned()
        .collect();

    for s in all_starts.iter() {
        for t in all_targets.iter() {
            if s == t {
                continue;
            }

            if let Some(path) = bfs(*s, *t, board, state) {
                G.insert((*s, *t), path);
            }
        }
    }

    G
}

pub fn solve(board: &Board, initial_state: &State) -> Option<State> {
    let G = prepare(board, initial_state);
    // eprintln!("{:?}", G);

    Some(State::new())
}
