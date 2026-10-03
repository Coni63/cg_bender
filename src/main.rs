mod board;
mod encoder;
mod loader;
mod search;
mod sim;

use std::time::{Duration, Instant};

use encoder::{cost_of, deep_compress, quick_compress, split_runs, stringify, widen_variants};
use loader::load_inputs;
use search::{Rng, World};

/// Pipeline : charge -> table de distance -> plus courts chemins variés ->
/// compression -> recuit simulé directement sur le programme.
/// Budget temps total ~850 ms (limite CodinGame : 1 s au premier tour),
/// modifiable via la variable d'environnement BENDER_MS pour les essais hors
/// ligne. Les autres paramètres (`search::param`) se règlent de la même façon.
/// Le programme compressé est écrit sur stdout, les logs sur stderr.
fn main() {
    let (board, state) = load_inputs();
    let timer = Instant::now();
    let budget: u64 = std::env::var("BENDER_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(850);
    let at = |frac: f64| timer + Duration::from_millis((budget as f64 * frac) as u64);

    let world = World::new(&board, &state);
    eprintln!("distance table: {:?}, start dist {}", timer.elapsed(), world.start_dist());

    // Phase 1 : plusieurs plus courts chemins tirés au hasard (biais ligne
    // droite variable), élargis contre les murs, filtrés par le glouton.
    let mut rng = Rng::new((search::param("SEED", 12345.0) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut candidates: Vec<String> = vec![];
    let paths_deadline = at(search::param("F_PATHS", 0.05));
    let mut seen = std::collections::HashSet::new();
    let mut k = 0u32;
    // au moins quelques chemins même si la table de distance a mangé le budget
    while (k < 5 || Instant::now() < paths_deadline) && k < 400 {
        let straight = [256, 230, 200, 160, 128][k as usize % 5];
        k += 1;
        let path = world.random_shortest_path(&mut rng, straight);
        if path.is_empty() || !seen.insert(path.clone()) {
            continue;
        }
        let free = sim::repeat_is_free(&board, &state, &path);
        let runs = split_runs(&path, &free);
        candidates.push(path);
        candidates.extend(widen_variants(&runs, 6));
    }
    eprintln!("{} paths, {} candidates ({:?})", seen.len(), candidates.len(), timer.elapsed());

    let mut scored: Vec<(usize, String)> = vec![];
    let quick_deadline = at(search::param("F_QUICK", 0.08));
    for actions in candidates {
        if Instant::now() >= quick_deadline && !scored.is_empty() {
            break;
        }
        let (core, macros) = quick_compress(&actions);
        scored.push((cost_of(&core, &macros), actions));
    }
    scored.sort_by_key(|(c, _)| *c);
    eprintln!("quick pass on {} ({:?}), best {}", scored.len(), timer.elapsed(), scored[0].0);

    // Phase 2 : compression approfondie (branchement) sur le top-K.
    let top_k = search::param("TOP_K", 3.0) as usize;
    let deep_deadline = at(search::param("F_DEEP", 0.08));
    let mut programs: Vec<String> = vec![];
    for (_, actions) in scored.iter().take(top_k) {
        let (core, macros) = quick_compress(actions);
        programs.push(stringify(&core, &macros));
        if Instant::now() < deep_deadline {
            programs.push(deep_compress(actions, deep_deadline));
        }
    }
    programs.retain(|p| sim::wins(&board, &state, p));
    if programs.is_empty() {
        programs.push(scored[0].1.clone()); // chemin brut, toujours valide
    }
    programs.sort_by_key(|p| p.len());
    eprintln!("deep pass ({:?}), best {}", timer.elapsed(), programs[0].len());

    // Phase 3 : recuit simulé directement sur le programme (le juge n'exige pas
    // de reproduire un chemin, seulement d'atteindre Fry).
    //  - exploration : température haute, pénalité de distance modérée ; la
    //    chaîne rétrécit le programme en passant souvent par des programmes
    //    perdants, on garde chaque programme gagnant plus court croisé en route ;
    //  - affinage : pénalité forte (on reste parmi les programmes gagnants),
    //    depuis le meilleur programme gagnant trouvé ;
    //  - enfin, fonctions fixées, BFS exact sur le main.
    let p = search::param;
    let explore = (p("E_L", 3.0), p("E_T0", 3.0), p("E_T1", 0.5));
    let refine = (p("R_L", 20.0), p("R_T0", 1.0), p("R_T1", 0.1));
    let explore_runs = p("E_RUNS", 2.0) as usize;
    let sa_end = at(1.0);
    let explore_end = Instant::now() + sa_end.saturating_duration_since(Instant::now()).mul_f64(p("E_FRAC", 0.7));
    let mut pool: Vec<search::Prog> = programs.iter().map(|s| search::parse(s)).collect();
    for i in 0..explore_runs {
        let now = Instant::now();
        let end = now + explore_end.saturating_duration_since(now) / (explore_runs - i) as u32;
        let start = pool[i % programs.len()].clone();
        pool.push(search::anneal(&world, &start, explore, end, &mut rng));
    }
    pool.sort_by_key(search::prog_cost);
    let mut prog = search::anneal(&world, &pool[0], refine, sa_end, &mut rng);
    let t = Instant::now();
    let before = search::prog_cost(&prog);
    if let Some(better) = world.best_main(&prog, sa_end + Duration::from_millis(15)) {
        prog = better;
    }
    eprintln!("best_main: {} -> {} ({:?})", before, search::prog_cost(&prog), t.elapsed());
    let out = search::to_string(&prog);
    let mut best = programs[0].clone();
    if out.len() < best.len() && sim::wins(&board, &state, &out) {
        best = out;
    }

    eprintln!("final program valid: {}", sim::wins(&board, &state, &best));
    eprintln!("Total Time: {:?}", timer.elapsed());
    println!("{}", best)
}
