mod bfs;
mod board;
mod encoder;
mod loader;
mod sim;

use std::time::{Duration, Instant};

use bfs::solve;
use encoder::{cost_of, deep_compress, quick_compress, split_runs, stringify, tune_lengths, widen_variants};
use loader::load_inputs;

/// Pipeline : charge -> simplifie la grille -> cherche un chemin -> compresse.
/// Budget temps total ~850 ms (Rust non optimisé sur CodinGame).
/// Le programme compressé est écrit sur stdout, les logs sur stderr.
fn main() {
    let (mut board, mut state) = load_inputs();

    let timer = std::time::Instant::now();
    // La simulation exacte tourne sur la carte d'origine (avant simplification).
    let raw_board = board.clone();
    let raw_state = state.clone();

    let step_timer = std::time::Instant::now();
    board.simplify(&mut state);
    eprintln!("Simplify the board tooks {:?}", step_timer.elapsed());

    let step_timer = std::time::Instant::now();
    let states = solve(&board, &state);
    eprintln!("Finding the solution tooks {:?}", step_timer.elapsed());

    let step_timer = std::time::Instant::now();

    // Phase 1 : passe gloutonne rapide sur tous les candidats pour identifier
    // les plus prometteurs (quasi gratuit, sert de filtre de compressibilité).
    let mut candidates: Vec<String> = states.iter().map(|s| s.get_actions().clone()).collect();
    if let Some(base) = candidates.first().cloned() {
        eprintln!("solver path valid: {}", sim::wins(&raw_board, &raw_state, &base));
        let free = sim::repeat_is_free(&raw_board, &raw_state, &base);
        let runs = split_runs(&base, &free);
        eprintln!(
            "runs: {}, extensible: {}",
            runs.len(),
            runs.iter().filter(|r| r.2).count()
        );
        candidates.extend(widen_variants(&runs, 12));
        candidates.push(tune_lengths(&runs, 12, timer + Duration::from_millis(500)));
    }
    let mut scored: Vec<(usize, String)> = candidates
        .into_iter()
        .map(|actions| {
            let (core, macros) = quick_compress(&actions);
            (cost_of(&core, &macros), actions)
        })
        .collect();
    scored.sort_by_key(|(c, _)| *c);

    eprintln!(
        "Quick pass took {:?}, best quick score: {}",
        step_timer.elapsed(),
        scored.first().map(|(c, _)| *c).unwrap_or(0)
    );

    // Phase 2 : branchement approfondi seulement sur le top-K des candidats
    // déjà identifiés comme les plus compressibles.
    const TOP_K: usize = 5;
    let compression_deadline = timer + Duration::from_millis(850);

    let mut shortest_path = String::new();
    let mut min_dist = usize::MAX;

    // On garde d'abord le meilleur résultat de la phase 1 comme baseline,
    // au cas où la phase 2 serait interrompue avant même le premier candidat.
    if let Some((c, s)) = scored.first() {
        let (core, macros) = quick_compress(s);
        shortest_path = stringify(&core, &macros);
        min_dist = *c;
    }

    for (_, actions) in scored.iter().take(TOP_K) {
        if Instant::now() >= compression_deadline {
            eprintln!("Deep compression deadline reached, stopping early");
            break;
        }
        let encoded = deep_compress(actions, compression_deadline);
        if encoded.len() < min_dist {
            min_dist = encoded.len();
            shortest_path = encoded;
        }
    }
    eprintln!("final program valid: {}", sim::wins(&raw_board, &raw_state, &shortest_path));
    eprintln!("Encoding the solution tooks {:?}", step_timer.elapsed());
    eprintln!("Total Time: {:?}", timer.elapsed());

    println!("{}", shortest_path)
}
