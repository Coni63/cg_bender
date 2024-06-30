use std::{
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
};

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
                if (current != start) & (board.get_cell(current) != &Cell::Empty) {
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
fn prepare(board: &Board, state: &State) -> HashMap<usize, HashMap<usize, String>> {
    let mut G: HashMap<usize, HashMap<usize, String>> = HashMap::new();

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
        G.insert(*s, HashMap::new());
        let mut sub = G.get_mut(s).unwrap();
        for t in all_targets.iter() {
            if s == t {
                continue;
            }

            if let Some(path) = bfs(*s, *t, board, state) {
                sub.insert(*t, path);
            }
        }
    }

    G
}

// Using the Graph G, solve the problem
fn find_path(
    graph: &HashMap<usize, HashMap<usize, String>>,
    board: &Board,
    state: &State,
) -> Vec<State> {
    let mut ans: Vec<State> = Vec::new();

    let mut queue: BinaryHeap<State> = BinaryHeap::new();

    let mut visited: HashSet<u64> = HashSet::new();

    queue.push(state.clone());

    let mut start_time = std::time::Instant::now();

    loop {
        // queue.sort_by_cached_key(|a| a.fitness());
        // eprintln!("Queue: {:?}", queue.len());
        // if queue.len() > 100000 {
        //     // eprintln!("Queue: {:?}", queue.len());
        //     return None;
        // }

        // let tmp = queue.iter().map(|x| x.fitness()).collect::<Vec<usize>>();
        // eprintln!("Fitness: {:?}", tmp);
        match queue.pop() {
            None => return ans,
            Some(current_state) => {
                // turns += 1;
                // if turns == 100 {
                //     return None;
                // }
                // eprintln!(
                //     "Current: {:?} - Actions {:?}",
                //     current_state.get_current_pos(),
                //     current_state.get_actions()
                // );
                // eprintln!("{}", current_state.get_actions());
                if current_state.get_current_pos() == board.get_target() {
                    // eprintln!("Found the target");
                    // return Some(current_state);
                    ans.push(current_state.clone());

                    if (ans.len() == 100) | (start_time.elapsed().as_millis() > 900) {
                        return ans;
                    }
                }

                if current_state.get_actions().len() > 1000 {
                    // eprintln!("Too long path, skipping it");
                    continue;
                }

                let targets = graph.get(&current_state.get_current_pos());
                if targets.is_none() {
                    // eprintln!("No target found for {}", current_state.get_current_pos());
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                current_state.hash(&mut hasher);
                let signature = hasher.finish();
                if visited.contains(&signature) {
                    // eprintln!("Already visited: {}", signature);
                    continue;
                }
                visited.insert(signature);

                for (target, path) in targets.unwrap() {
                    // eprintln!("checking target: {} - ", target);
                    let mut new_state = current_state.clone();
                    match board.get_cell(*target) {
                        Cell::Switch(id) => {
                            new_state.toggle_magnetic_field(*id);
                        }
                        Cell::MagneticField(id) => {
                            if current_state.is_magnetic_field_on(*id) {
                                // eprintln!("Magnetic field {} is on: {} -- Skipping", id, target);
                                continue;
                            }
                        }
                        Cell::Empty => (), // This is the case we we reach the target
                        _ => continue,
                    }

                    // eprintln!("Adding action: {}", target);
                    new_state.add_actions(path);
                    new_state.set_current_pos(*target);
                    queue.push(new_state);
                }
            }
        }
    }
}

pub fn solve(board: &Board, initial_state: &State) -> Vec<State> {
    // eprintln!(
    //     "Solving the board ({} -> {})",
    //     board.get_start(),
    //     board.get_target()
    // );

    let graph = prepare(board, initial_state);
    // eprintln!("{:?}", graph);

    find_path(&graph, board, initial_state)
}
