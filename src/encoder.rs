use std::{cmp::min, collections::HashMap, time::Instant};

fn find_substrings<'a>(s: &'a str) -> HashMap<&'a str, i32> {
    let mut substr_count = HashMap::new();
    let length = s.len();
    // Plus cher n'est plus un problème sans branchement : fenêtre élargie
    // pour capter de plus gros motifs en un seul coup.
    const WINDOW: usize = 30;

    for start in 0..length {
        let offset = min(length, start + WINDOW);
        for end in start + 1..offset {
            let substr = &s[start..end];
            // On autorise les chiffres (appels de macros déjà définies) pour
            // permettre la composition hiérarchique de motifs.
            // Exclus : ';' (séparateur) et '0' (réservé, voir extract_recursive_tail).
            if substr.len() >= 2 && substr.chars().all(|c| c != ';' && c != '0') {
                *substr_count.entry(substr).or_insert(0) += 1;
            }
        }
    }
    substr_count
}

fn sort_substring<'a>(counter: &HashMap<&'a str, i32>) -> Vec<(&'a str, i32)> {
    let mut ans: Vec<(&str, i32)> = counter
        .iter()
        .map(|(&subset, &occurrences)| {
            let gain = (subset.len() as i32) * (1 - occurrences) + 2;
            (subset, gain)
        })
        .filter(|&(_, gain)| gain < 0)
        .collect();

    ans.sort_by_key(|&(_, gain)| gain);
    ans
}

pub fn stringify(s: &str, macros: &[String]) -> String {
    [s.to_string()]
        .iter()
        .chain(macros.iter())
        .cloned()
        .collect::<Vec<String>>()
        .join(";")
}

/// Compression gloutonne sans branchement : à chaque étape, on prend LA
/// meilleure sous-chaîne (plus grand gain), on remplace toutes ses
/// occurrences par le prochain digit disponible, et on répète.
/// Pas de récursion/branchement => O(max_macros * n * WINDOW), donc
/// aucun besoin de deadline ou de cache ici.
fn _compress_greedy(s: &str, max_macros: usize) -> (String, Vec<String>) {
    let mut current = s.to_string();
    let mut macros: Vec<String> = Vec::new();

    for depth in 1..=max_macros {
        let substr_count = find_substrings(&current);
        let sorted_count = sort_substring(&substr_count);

        match sorted_count.first() {
            None => break,
            Some((subset, _)) => {
                let s_subset = subset.to_string();
                current = current.replace(&s_subset, &depth.to_string());
                macros.push(s_subset);
            }
        }
    }

    (current, macros)
}

/// Détecte un motif répété à la toute fin du chemin et le transforme en
/// petite fonction récursive. C'est sûr précisément parce que c'est la FIN
/// du programme entier : l'exécution s'arrête dès que Fry est atteint, peu
/// importe combien de fois la fonction doit encore se rappeler elle-même
/// -- les répétitions en trop ne s'exécutent tout simplement jamais.
/// Retourne la chaîne raccourcie (avec un sentinel '0' à l'endroit de
/// l'appel) et le corps de la fonction (motif + '0' pour l'auto-appel),
/// ou None si rien d'intéressant n'a été trouvé.
fn extract_recursive_tail(s: &str) -> (String, Option<String>) {
    let bytes = s.as_bytes();
    let n = bytes.len();
    if n < 4 {
        return (s.to_string(), None);
    }

    let max_period = min(n / 2, 8);
    let mut best: Option<(usize, usize, i32)> = None; // (period, matched_len, gain)

    for period in 1..=max_period {
        let pattern = &bytes[n - period..];
        let mut matched = 0;
        while matched + period <= n && &bytes[n - matched - period..n - matched] == pattern {
            matched += period;
        }
        if matched < 2 * period {
            continue; // il faut au moins 2 répétitions pour que ça vaille le coup
        }

        // Coût avant : `matched` caractères littéraux.
        // Coût après : 1 char (l'appel) dans le core + `period` chars et
        // 1 char d'auto-appel dans la définition + 1 séparateur ';'.
        let gain = matched as i32 - (1 + period as i32 + 1 + 1);
        let is_better = match best {
            Some((_, _, best_gain)) => gain > best_gain,
            None => true,
        };
        if gain > 0 && is_better {
            best = Some((period, matched, gain));
        }
    }

    best.map(|(period, matched, _)| {
        let core = format!("{}0", &s[..n - matched]);
        let definition = format!("{}0", &s[n - period..n]);
        (core, Some(definition))
    })
    .unwrap_or((s.to_string(), None))
}

/// Une étape de compression gloutonne pure (pas de branchement) : prend la
/// meilleure sous-chaîne au sens du gain, répète jusqu'à max_macros ou plus
/// rien à gagner. Utilisée pour "terminer" chaque branche de l'exploration.
fn compress_greedy_from(
    current: &str,
    macros_so_far: &[String],
    max_macros: usize,
) -> (String, Vec<String>) {
    let mut current = current.to_string();
    let mut macros: Vec<String> = macros_so_far.to_vec();
    let start_depth = macros.len() + 1;

    for depth in start_depth..=max_macros {
        let substr_count = find_substrings(&current);
        let sorted_count = sort_substring(&substr_count);

        match sorted_count.first() {
            None => break,
            Some((subset, _)) => {
                let s_subset = subset.to_string();
                current = current.replace(&s_subset, &depth.to_string());
                macros.push(s_subset);
            }
        }
    }

    (current, macros)
}

fn cost(core: &str, macros: &[String]) -> usize {
    core.len() + macros.iter().map(|m| m.len() + 1).sum::<usize>()
}

/// Explore les `branch_depth` premiers choix de macro avec un beam de
/// `beam_width` candidats, puis termine chaque branche en glouton pur.
/// Coût borné (pas d'explosion exponentielle) car le branchement s'arrête
/// après `branch_depth` niveaux.
fn compress_hybrid(
    s: &str,
    max_macros: usize,
    branch_depth: usize,
    beam_width: usize,
    deadline: Instant,
) -> (String, Vec<String>) {
    // best trouvé jusqu'ici, initialisé avec une complétion gloutonne pure
    // (équivalent à branch_depth=0), garantit qu'on ne fait jamais pire.
    let mut best = compress_greedy_from(s, &[], max_macros);

    fn explore(
        current: &str,
        macros_so_far: &[String],
        remaining_branch_depth: usize,
        max_macros: usize,
        beam_width: usize,
        deadline: Instant,
        best: &mut (String, Vec<String>),
    ) {
        if Instant::now() >= deadline {
            return;
        }

        // Toujours évaluer la complétion gloutonne à partir d'ici,
        // même si on va aussi brancher plus loin.
        let completed = compress_greedy_from(current, macros_so_far, max_macros);
        if cost(&completed.0, &completed.1) < cost(&best.0, &best.1) {
            *best = completed;
        }

        if remaining_branch_depth == 0 || macros_so_far.len() >= max_macros {
            return;
        }

        let substr_count = find_substrings(current);
        let sorted_count = sort_substring(&substr_count);

        let depth = macros_so_far.len() + 1;
        for (subset, _) in sorted_count.iter().take(beam_width) {
            if Instant::now() >= deadline {
                return;
            }
            let s_subset = subset.to_string();
            let next = current.replace(&s_subset, &depth.to_string());
            let mut next_macros = macros_so_far.to_vec();
            next_macros.push(s_subset);

            explore(
                &next,
                &next_macros,
                remaining_branch_depth - 1,
                max_macros,
                beam_width,
                deadline,
                best,
            );
        }
    }

    explore(
        s,
        &[],
        branch_depth,
        max_macros,
        beam_width,
        deadline,
        &mut best,
    );

    best
}

pub fn quick_compress(s: &str) -> (String, Vec<String>) {
    compress_greedy_from(s, &[], 9)
}

pub fn deep_compress(s: &str, deadline: Instant) -> String {
    const BRANCH_DEPTH: usize = 2;
    const BEAM_WIDTH: usize = 5;

    let (core, macros) = compress_hybrid(s, 9, BRANCH_DEPTH, BEAM_WIDTH, deadline);
    let without_tail = stringify(&core, &macros);

    let (core_with_sentinel, tail_def) = extract_recursive_tail(s);
    let with_tail = tail_def.map(|tail| {
        let (core, mut macros) =
            compress_hybrid(&core_with_sentinel, 8, BRANCH_DEPTH, BEAM_WIDTH, deadline);
        let digit = (macros.len() + 1).to_string();
        let core = core.replacen('0', &digit, 1);
        let tail = tail.replacen('0', &digit, 1);
        macros.push(tail);
        stringify(&core, &macros)
    });

    match with_tail {
        Some(w) if w.len() < without_tail.len() => w,
        _ => without_tail,
    }
}

pub fn cost_of(core: &str, macros: &[String]) -> usize {
    cost(core, macros)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const PATH: &str = "DDDDRRDDDDDUUUUULLUUUULLLLUUDDRRRRDDDDRRDDDDDDDDDDDDLLUUUULLUUUUULURUULLLLRRRRDDLDRRRDDDDDLLLLUUUULRDDDDRRRRDDDDLLUDRRUUUULLUUUUULULLLLLUULLLLLLLLDDDDDDRRRRUDRRRRRRDLLDDDRRDDLLLLLLUUUULLLLUUUUUUUURRRRRRRRDRDRRRRRDRRDDDDDDDDDRRUUUUUUUUUUUULLUUUULLLLLLLLUUL";

    /// Expands a program ("core;m1;m2;...") into at most `limit` actions.
    /// Recursion-safe: stops as soon as `limit` actions are produced.
    fn expand(program: &str, limit: usize) -> String {
        let parts: Vec<&str> = program.split(';').collect();
        let mut out = String::new();
        // pile d'exécution : (fonction, index)
        let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
        while let Some(&(f, i)) = stack.last() {
            if out.len() >= limit {
                break;
            }
            let bytes = parts[f].as_bytes();
            if i >= bytes.len() {
                stack.pop();
                continue;
            }
            stack.last_mut().unwrap().1 += 1;
            let c = bytes[i] as char;
            if let Some(d) = c.to_digit(10) {
                stack.push((d as usize, 0));
            } else {
                out.push(c);
            }
        }
        out
    }

    fn check(program: &str, path: &str) {
        assert!(program.len() < path.len(), "pas de compression: {program}");
        assert!(
            expand(program, path.len()) == path,
            "le programme ne reproduit pas le chemin: {program}"
        );
    }

    #[test]
    fn test_greedy() {
        let (core, macros) = quick_compress(PATH);
        let program = stringify(&core, &macros);
        eprintln!("greedy: {} chars -> {program}", program.len());
        check(&program, PATH);
    }

    #[test]
    fn test_beam_search() {
        let deadline = Instant::now() + Duration::from_millis(800);
        let program = deep_compress(PATH, deadline);
        eprintln!("beam: {} chars -> {program}", program.len());
        check(&program, PATH);
    }

    #[test]
    fn test_beam_not_worse_than_greedy() {
        let (core, macros) = quick_compress(PATH);
        let greedy = stringify(&core, &macros);
        let deadline = Instant::now() + Duration::from_millis(800);
        let beam = deep_compress(PATH, deadline);
        eprintln!("greedy={} beam={}", greedy.len(), beam.len());
        assert!(beam.len() <= greedy.len());
    }
}
