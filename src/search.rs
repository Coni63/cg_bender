//! Recherche directe sur les programmes.
//!
//! Le juge ne vérifie pas qu'un programme reproduit un chemin donné : il exécute
//! le programme et regarde si Bender atteint Fry. On peut donc modifier le
//! programme compressé à volonté (supprimer un caractère, changer un appel...)
//! tant que la simulation gagne encore. `World` contient tout ce qu'il faut pour
//! simuler vite, plus une table de distance exacte vers Fry qui sert à noter les
//! programmes perdants (« à combien de coups de Fry est-on passé ? »).

use crate::board::{Board, Cell, State};
use std::cell::RefCell;
use std::collections::HashSet;
use std::time::Instant;

/// Même marge que `sim::MAX_TURNS` (le moteur coupe à 1000 tours).
const MAX_TURNS: usize = 990;
const OFFSETS: [(u8, i32); 4] = [(b'U', -21), (b'D', 21), (b'L', -1), (b'R', 1)];
pub const UNREACHABLE: u16 = u16::MAX;

pub struct World {
    wall: [bool; 441],
    sw: [u16; 441],    // bit du champ togglé en entrant sur la case (0 sinon)
    field: [u16; 441], // bit du champ posé sur la case (0 sinon)
    start: usize,
    target: usize,
    mask0: u16,
    balls0: Vec<u16>,
    nm: usize, // nombre de masques = 2^nb_champs
    /// dist[masque * 441 + pos] = nb minimal de coups jusqu'à Fry, balls = murs.
    dist: Vec<u16>,
    /// Pile d'appels réutilisée par `exec` (évite une allocation par appel).
    stack: RefCell<Vec<(u8, u16)>>,
}

/// Résultat d'une exécution : gagné ou non, et distance minimale à Fry
/// atteinte pendant l'exécution (0 si gagné).
#[derive(Clone, Copy)]
pub struct Eval {
    pub win: bool,
    pub best: u16,
}

impl World {
    /// Construit le monde depuis la carte NON simplifiée, et calcule la table de
    /// distance par BFS arrière depuis tous les états (Fry, masque).
    pub fn new(board: &Board, state: &State) -> World {
        let mut wall = [false; 441];
        let mut sw = [0u16; 441];
        let mut field = [0u16; 441];
        let mut nf = 0;
        for p in 0..441 {
            match board.get_cell(p) {
                Cell::Wall => wall[p] = true,
                Cell::Switch(id) => {
                    sw[p] = 1 << id;
                    nf = nf.max(id + 1);
                }
                Cell::MagneticField(id) => {
                    field[p] = 1 << id;
                    nf = nf.max(id + 1);
                }
                Cell::Empty => {}
            }
        }
        let mask0 = (0..nf)
            .filter(|&i| state.is_magnetic_field_on(i))
            .fold(0u16, |m, i| m | (1 << i));
        let balls0: Vec<u16> = state.get_garbage_balls().iter().map(|&b| b as u16).collect();
        let nm = 1usize << nf;
        let mut world = World {
            wall,
            sw,
            field,
            start: board.get_start(),
            target: board.get_target(),
            mask0,
            balls0,
            nm,
            dist: vec![UNREACHABLE; 441 * nm],
            stack: RefCell::new(Vec::with_capacity(MAX_TURNS + 2)),
        };
        world.backward_bfs();
        world
    }

    fn blocked(&self, p: usize) -> bool {
        self.wall[p] || self.balls0.contains(&(p as u16))
    }

    /// Transition avant : (p, m) --dir--> (q, m ^ sw[q]) si q libre et que le
    /// champ de q est éteint après le toggle. En arrière depuis (q, m2) :
    /// p = q - dir, m = m2 ^ sw[q], (p, m) doit être un état vivant.
    fn backward_bfs(&mut self) {
        let nm = self.nm;
        let t = self.target;
        let mut queue: Vec<u32> = Vec::with_capacity(441 * nm / 4);
        for m in 0..nm {
            if self.field[t] as usize & m == 0 {
                self.dist[m * 441 + t] = 0;
                queue.push((m * 441 + t) as u32);
            }
        }
        let mut blocked = [false; 441];
        for (p, b) in blocked.iter_mut().enumerate() {
            *b = self.blocked(p);
        }
        let mut head = 0;
        while head < queue.len() {
            let s = queue[head] as usize;
            head += 1;
            let q = s % 441;
            let m = (s / 441) as u16 ^ self.sw[q];
            let d = self.dist[s] + 1;
            for &(_, off) in OFFSETS.iter() {
                let p = q as i32 - off;
                if !(0..441).contains(&p) {
                    continue;
                }
                let p = p as usize;
                if blocked[p] || p == t || self.field[p] & m != 0 {
                    continue;
                }
                let idx = m as usize * 441 + p;
                if self.dist[idx] == UNREACHABLE {
                    self.dist[idx] = d;
                    queue.push(idx as u32);
                }
            }
        }
    }

    pub fn start_dist(&self) -> u16 {
        self.dist[self.mask0 as usize * 441 + self.start]
    }

    /// Tire un plus court chemin au hasard en suivant la table de distance.
    /// `straight` = probabilité (sur 256) de garder la direction précédente
    /// quand elle reste optimale : des lignes plus longues compressent mieux.
    pub fn random_shortest_path(&self, rng: &mut Rng, straight: u32) -> String {
        let mut pos = self.start;
        let mut mask = self.mask0;
        let mut out = String::new();
        let mut last: Option<usize> = None;
        while pos != self.target {
            let d = self.dist[mask as usize * 441 + pos];
            if d == UNREACHABLE {
                return String::new();
            }
            let mut good: [usize; 4] = [0; 4];
            let mut n = 0;
            for (k, &(_, off)) in OFFSETS.iter().enumerate() {
                let q = (pos as i32 + off) as usize;
                if self.blocked(q) {
                    continue;
                }
                let m = mask ^ self.sw[q];
                if self.dist[m as usize * 441 + q] == d - 1 {
                    good[n] = k;
                    n += 1;
                }
            }
            let k = match last {
                Some(l) if good[..n].contains(&l) && rng.below(256) < straight => l,
                _ => good[rng.below(n as u32) as usize],
            };
            let q = (pos as i32 + OFFSETS[k].1) as usize;
            mask ^= self.sw[q];
            pos = q;
            out.push(OFFSETS[k].0 as char);
            last = Some(k);
        }
        out
    }

    pub fn initial_pawn(&self) -> Pawn {
        let mut balls = [0u64; 7];
        let mut bh = 0;
        for &b in &self.balls0 {
            balls[b as usize >> 6] |= 1 << (b & 63);
            bh ^= zobrist(b as usize);
        }
        Pawn {
            pos: self.start as u16,
            mask: self.mask0,
            balls,
            bh,
        }
    }

    fn dist_of(&self, pawn: &Pawn) -> u16 {
        self.dist[pawn.mask as usize * 441 + pawn.pos as usize]
    }

    /// Un coup (U/D/L/R) avec les règles du moteur. Return = la partie continue.
    #[inline]
    fn step(&self, pawn: &mut Pawn, off: i32) -> StepOutcome {
        let pos = pawn.pos as i32;
        let next = pos + off;
        let nu = next as usize;
        if self.wall[nu] {
            return StepOutcome::Return;
        }
        if pawn.has_ball(nu) {
            let bt = 2 * next - pos;
            if !(0..441).contains(&bt) || self.wall[bt as usize] || pawn.has_ball(bt as usize) {
                return StepOutcome::Return; // ball bloquée : coup perdu
            }
            pawn.mask ^= self.sw[nu];
            pawn.balls[nu >> 6] ^= 1 << (nu & 63);
            pawn.balls[bt as usize >> 6] |= 1 << (bt & 63);
            pawn.bh ^= zobrist(nu) ^ zobrist(bt as usize);
            pawn.mask ^= self.sw[bt as usize];
            if bt as usize == self.target {
                return StepOutcome::Lose;
            }
        } else {
            pawn.mask ^= self.sw[nu];
        }
        pawn.pos = nu as u16;
        if self.field[nu] & pawn.mask != 0 {
            return StepOutcome::Lose;
        }
        if nu == self.target {
            return StepOutcome::Win;
        }
        StepOutcome::Return
    }

    /// Exécute la fonction `entry` de `prog` jusqu'à son retour (tour du retour
    /// compris), comme l'interpréteur du jeu : chaque caractère et chaque retour
    /// de fonction consomme un tour. `best` suit la distance minimale à Fry.
    fn exec(
        &self,
        prog: &[Vec<u8>],
        pawn: &mut Pawn,
        entry: usize,
        turns: &mut usize,
        best: &mut u16,
        touched: &mut u16,
    ) -> StepOutcome {
        *touched |= 1 << entry;
        let mut stack = self.stack.borrow_mut();
        stack.clear(); // frames appelantes : (fonction, index du prochain caractère)
        let mut f = entry;
        let mut body: &[u8] = &prog[f];
        let mut i = 0usize;
        // Détection de cycle (Brent) sur les appels : si l'on rappelle la même
        // fonction dans le même état sans être jamais redescendu sous la
        // profondeur du point de repère, l'exécution se répète à l'infini.
        let mut ck_key = u64::MAX;
        let mut ck_depth = 0usize;
        let mut ck_min = 0usize;
        let mut power = 1u32;
        let mut count = 0u32;
        while *turns < MAX_TURNS {
            *turns += 1;
            if i >= body.len() {
                match stack.pop() {
                    None => return StepOutcome::Return,
                    Some((g, j)) => {
                        f = g as usize;
                        body = &prog[f];
                        i = j as usize;
                    }
                }
                ck_min = ck_min.min(stack.len());
                continue;
            }
            let c = body[i];
            i += 1;
            let off = match c {
                b'U' => -21,
                b'D' => 21,
                b'L' => -1,
                b'R' => 1,
                _ => 0,
            };
            if off != 0 {
                match self.step(pawn, off) {
                    StepOutcome::Return => {
                        let d = self.dist_of(pawn);
                        if d < *best {
                            *best = d;
                        }
                    }
                    o => return o,
                }
            } else if (b'1'..=b'9').contains(&c) {
                let idx = (c - b'0') as usize;
                if idx >= prog.len() {
                    return StepOutcome::Lose;
                }
                *touched |= 1 << idx;
                stack.push((f as u8, i as u16));
                f = idx;
                body = &prog[f];
                i = 0;
                let key = pawn.key() ^ ((idx as u64) << 59);
                if key == ck_key && ck_min >= ck_depth {
                    return StepOutcome::Timeout; // boucle infinie prouvée
                }
                count += 1;
                if count == power {
                    ck_key = key;
                    ck_depth = stack.len();
                    ck_min = ck_depth;
                    power *= 2;
                    count = 0;
                }
            }
        }
        StepOutcome::Timeout
    }

    /// Exécute un programme complet exactement comme `sim::wins`, en suivant la
    /// distance minimale à Fry atteinte (0 si gagné). Référence pour les tests ;
    /// la recherche utilise `run_from`.
    #[cfg(test)]
    pub fn run(&self, prog: &[Vec<u8>]) -> Eval {
        let mut pawn = self.initial_pawn();
        let mut best = self.dist_of(&pawn);
        let mut turns = 0;
        match self.exec(prog, &mut pawn, 0, &mut turns, &mut best, &mut 0) {
            StepOutcome::Win => Eval { win: true, best: 0 },
            _ => Eval { win: false, best },
        }
    }

    /// Fonctions de `prog` fixées, cherche par BFS sur les états du jeu le main
    /// le plus court (un token = un coup ou un appel de fonction). Retourne le
    /// programme obtenu s'il est plus court, None sinon (ou si `deadline` passe).
    pub fn best_main(&self, prog: &Prog, deadline: Instant) -> Option<Prog> {
        let nfun = prog.len();
        let mut tokens: Vec<u8> = b"UDLR".to_vec();
        tokens.extend((1..nfun).map(|g| b'0' + g as u8));
        let limit = prog[0].len(); // au-delà, inutile de chercher

        // noeud = (pion, parent, token, tours consommés, profondeur)
        let mut nodes: Vec<(Pawn, u32, u8, u16, u16)> =
            vec![(self.initial_pawn(), u32::MAX, 0, 0, 0)];
        let mut seen: HashSet<u64, BuildIdHasher> = HashSet::with_hasher(BuildIdHasher);
        seen.insert(nodes[0].0.key());
        let mut head = 0;
        let mut found: Option<(u32, u8)> = None;
        'bfs: while head < nodes.len() {
            if head % 1024 == 0 && Instant::now() >= deadline {
                return None;
            }
            let (pawn, _, _, turns, depth) = nodes[head];
            if depth as usize + 1 >= limit {
                break;
            }
            for &t in &tokens {
                let mut p = pawn;
                let mut tu = turns as usize + 1;
                let mut best = u16::MAX;
                let o = match t {
                    b'U' => self.step(&mut p, -21),
                    b'D' => self.step(&mut p, 21),
                    b'L' => self.step(&mut p, -1),
                    b'R' => self.step(&mut p, 1),
                    _ => self.exec(prog, &mut p, (t - b'0') as usize, &mut tu, &mut best, &mut 0),
                };
                match o {
                    StepOutcome::Win => {
                        found = Some((head as u32, t));
                        break 'bfs;
                    }
                    StepOutcome::Return if tu < MAX_TURNS => {
                        if seen.insert(p.key()) {
                            nodes.push((p, head as u32, t, tu as u16, depth + 1));
                        }
                    }
                    _ => {}
                }
            }
            head += 1;
        }
        let (mut at, last) = found?;
        let mut main = vec![last];
        while at != 0 {
            main.push(nodes[at as usize].2);
            at = nodes[at as usize].1;
        }
        main.reverse();
        let mut out = prog.clone();
        out[0] = main;
        cleanup(&mut out);
        if prog_cost(&out) < prog_cost(prog) {
            Some(out)
        } else {
            None
        }
    }
}

/// Exécution mémorisée d'un programme : état avant chaque token du main, et
/// pour chaque fonction le premier token du main dont l'exécution l'a appelée.
/// Permet de ne resimuler un programme muté qu'à partir du premier token dont
/// l'exécution diffère.
#[derive(Clone)]
pub struct Trace {
    snaps: Vec<(Pawn, u16, u16)>, // (pion, tours, meilleure distance) avant main[k]
    first_touch: [u16; 10],
    end: usize, // index du token du main où l'exécution s'est arrêtée
    pub eval: Eval,
}

impl Trace {
    pub fn new() -> Trace {
        Trace {
            snaps: Vec::new(),
            first_touch: [u16::MAX; 10],
            end: 0,
            eval: Eval { win: false, best: UNREACHABLE },
        }
    }
}

impl World {
    /// Premier token du main de `cand` dont l'exécution peut différer de celle
    /// de `base` (programme `old`) : tout ce qui précède est identique.
    fn restart_point(&self, old: &Prog, base: &Trace, cand: &Prog) -> usize {
        if base.snaps.is_empty() {
            return 0; // pas encore de trace
        }
        let mut k = old[0]
            .iter()
            .zip(cand[0].iter())
            .take_while(|(a, b)| a == b)
            .count();
        for f in 1..cand.len().max(old.len()) {
            if f >= old.len() || f >= cand.len() || old[f] != cand[f] {
                k = k.min(base.first_touch[f] as usize);
            }
        }
        k
    }

    /// Évalue `cand` en reprenant l'exécution de `base` (trace de `old`) au
    /// premier token qui diffère. Remplit `out` avec la trace de `cand`.
    pub fn run_from(&self, old: &Prog, base: &Trace, cand: &Prog, out: &mut Trace) -> Eval {
        let k0 = self.restart_point(old, base, cand);
        if k0 > base.end {
            // le programme muté s'exécute exactement comme l'ancien
            out.clone_from(base);
            return base.eval;
        }
        out.snaps.clear();
        out.snaps.extend_from_slice(&base.snaps[..k0]);
        for f in 0..10 {
            let t = base.first_touch[f];
            out.first_touch[f] = if (t as usize) < k0 { t } else { u16::MAX };
        }
        let (mut pawn, turns, best) = if k0 == 0 {
            let p = self.initial_pawn();
            let d = self.dist_of(&p);
            (p, 0u16, d)
        } else {
            base.snaps[k0]
        };
        let mut turns = turns as usize;
        let mut best = best;
        let main = &cand[0];
        let mut k = k0;
        let finish = |out: &mut Trace, k: usize, e: Eval| {
            out.end = k;
            out.eval = e;
            e
        };
        loop {
            out.snaps.push((pawn, turns as u16, best));
            if turns >= MAX_TURNS {
                return finish(out, k, Eval { win: false, best });
            }
            if k >= main.len() {
                return finish(out, k, Eval { win: false, best }); // plus de commandes
            }
            turns += 1;
            let c = main[k];
            let o = match c {
                b'U' => self.step(&mut pawn, -21),
                b'D' => self.step(&mut pawn, 21),
                b'L' => self.step(&mut pawn, -1),
                b'R' => self.step(&mut pawn, 1),
                _ => {
                    let g = (c - b'0') as usize;
                    if g >= cand.len() {
                        StepOutcome::Lose
                    } else {
                        let mut touched = 0u16;
                        let o = self.exec(cand, &mut pawn, g, &mut turns, &mut best, &mut touched);
                        for f in 1..10 {
                            if touched & (1 << f) != 0 && out.first_touch[f] == u16::MAX {
                                out.first_touch[f] = k as u16;
                            }
                        }
                        o
                    }
                }
            };
            match o {
                StepOutcome::Win => return finish(out, k, Eval { win: true, best: 0 }),
                StepOutcome::Return => {
                    if c.is_ascii_uppercase() {
                        let d = self.dist_of(&pawn);
                        if d < best {
                            best = d;
                        }
                    }
                }
                _ => return finish(out, k, Eval { win: false, best }),
            }
            k += 1;
        }
    }
}


/// État dynamique du jeu : position, champs actifs, balls.
#[derive(Clone, Copy)]
pub struct Pawn {
    pos: u16,
    mask: u16,
    balls: [u64; 7], // bitset des cases occupées par une ball
    bh: u64,         // hash Zobrist des positions des balls
}

/// Valeur Zobrist pseudo-aléatoire d'une case (mélange splitmix64).
fn zobrist(p: usize) -> u64 {
    let mut z = (p as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Pawn {
    #[inline]
    fn has_ball(&self, p: usize) -> bool {
        self.balls[p >> 6] & (1 << (p & 63)) != 0
    }

    /// Clé de déduplication (hash des balls : collisions négligeables, et le
    /// programme final est de toute façon revalidé par `sim::wins`).
    fn key(&self) -> u64 {
        (self.bh << 25) ^ ((self.mask as u64) << 9) ^ self.pos as u64
    }
}

#[derive(PartialEq)]
enum StepOutcome {
    Return,
    Win,
    Lose,
    Timeout,
}

/// Hasher pour des clés u64 (un seul multiplicatif, pas de SipHash).
#[derive(Default, Clone, Copy)]
struct BuildIdHasher;
struct IdHasher(u64);
impl std::hash::BuildHasher for BuildIdHasher {
    type Hasher = IdHasher;
    fn build_hasher(&self) -> IdHasher {
        IdHasher(0)
    }
}
impl std::hash::Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }
}


/// xorshift64* : suffisant ici, et sans dépendance externe.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn below(&mut self, n: u32) -> u32 {
        (((self.next() >> 32) * n as u64) >> 32) as u32
    }
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Paramètre réglable par variable d'environnement (essais hors ligne).
pub fn param(name: &str, default: f64) -> f64 {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

pub type Prog = Vec<Vec<u8>>;

pub fn parse(program: &str) -> Prog {
    program.split(';').map(|f| f.as_bytes().to_vec()).collect()
}

pub fn to_string(p: &Prog) -> String {
    p.iter()
        .map(|f| String::from_utf8_lossy(f).into_owned())
        .collect::<Vec<_>>()
        .join(";")
}

pub fn prog_cost(p: &Prog) -> usize {
    p.iter().map(|f| f.len()).sum::<usize>() + p.len() - 1
}

/// Supprime les fonctions vides ou jamais atteintes depuis le main, et
/// renumérote les appels. Ne change pas les coups joués (seulement des tours
/// à vide en moins).
fn cleanup(p: &mut Prog) {
    loop {
        let n = p.len();
        let mut reach = [false; 10];
        reach[0] = true;
        let mut todo = [0usize; 10];
        let mut nt = 1;
        while nt > 0 {
            nt -= 1;
            let f = todo[nt];
            for &c in &p[f] {
                if (b'1'..=b'9').contains(&c) {
                    let g = (c - b'0') as usize;
                    if g < n && !reach[g] {
                        reach[g] = true;
                        todo[nt] = g;
                        nt += 1;
                    }
                }
            }
        }
        let mut keep = [true; 10];
        for f in 1..n {
            keep[f] = reach[f] && !p[f].is_empty();
        }
        if keep[..n].iter().all(|&k| k) {
            return;
        }
        let mut new_id = vec![0u8; n];
        let mut k = 0u8;
        for f in 0..n {
            if keep[f] {
                new_id[f] = k;
                k += 1;
            }
        }
        let mut out: Prog = Vec::with_capacity(k as usize);
        for f in 0..n {
            if !keep[f] {
                continue;
            }
            let body: Vec<u8> = p[f]
                .iter()
                .filter_map(|&c| {
                    if (b'1'..=b'9').contains(&c) {
                        let g = (c - b'0') as usize;
                        if g >= n || !keep[g] {
                            None // appel d'une fonction vide/inexistante : coup à vide
                        } else {
                            Some(b'0' + new_id[g])
                        }
                    } else {
                        Some(c)
                    }
                })
                .collect();
            out.push(body);
        }
        *p = out;
    }
}

fn random_token(p: &Prog, rng: &mut Rng) -> u8 {
    let n = 4 + (p.len() - 1) as u32;
    match rng.below(n) {
        0 => b'U',
        1 => b'D',
        2 => b'L',
        3 => b'R',
        k => b'0' + (k - 3) as u8,
    }
}

/// Position aléatoire uniforme parmi tous les caractères (Some) ;
/// None si le programme est vide.
fn random_char(p: &Prog, rng: &mut Rng) -> Option<(usize, usize)> {
    let total: usize = p.iter().map(|f| f.len()).sum();
    if total == 0 {
        return None;
    }
    let mut r = rng.below(total as u32) as usize;
    for (f, body) in p.iter().enumerate() {
        if r < body.len() {
            return Some((f, r));
        }
        r -= body.len();
    }
    None
}

/// Applique une mutation aléatoire. Retourne false si rien n'a été fait.
/// `block` = poids (sur 100 pour les autres mutations) du remplacement de bloc.
fn mutate(p: &mut Prog, rng: &mut Rng, block: u32) -> bool {
    match rng.below(100 + block) {
        // remplacer un bloc de 1 à 4 caractères par 0 à 4 tokens aléatoires
        100.. => {
            let Some((f, i)) = random_char(p, rng) else { return false };
            let len = (1 + rng.below(4) as usize).min(p[f].len() - i);
            let n = rng.below(5) as usize;
            let new: Vec<u8> = (0..n).map(|_| random_token(p, rng)).collect();
            p[f].splice(i..i + len, new);
        }
        // remplacer un caractère
        0..=29 => {
            let Some((f, i)) = random_char(p, rng) else { return false };
            let t = random_token(p, rng);
            if t == p[f][i] {
                return false;
            }
            p[f][i] = t;
        }
        // supprimer un caractère
        30..=49 => {
            let Some((f, i)) = random_char(p, rng) else { return false };
            p[f].remove(i);
        }
        // insérer un caractère
        50..=64 => {
            let f = rng.below(p.len() as u32) as usize;
            let i = rng.below(p[f].len() as u32 + 1) as usize;
            let t = random_token(p, rng);
            p[f].insert(i, t);
        }
        // échanger deux voisins
        65..=74 => {
            let Some((f, i)) = random_char(p, rng) else { return false };
            if i + 1 >= p[f].len() || p[f][i] == p[f][i + 1] {
                return false;
            }
            p[f].swap(i, i + 1);
        }
        // extraire une sous-chaîne répétée dans une nouvelle fonction
        75..=84 => {
            if p.len() >= 10 {
                return false;
            }
            let Some((f, i)) = random_char(p, rng) else { return false };
            let len = 2 + rng.below(5) as usize;
            if i + len > p[f].len() {
                return false;
            }
            let pat = p[f][i..i + len].to_vec();
            let digit = b'0' + p.len() as u8;
            let mut count = 0;
            for body in p.iter_mut() {
                let mut out = Vec::with_capacity(body.len());
                let mut j = 0;
                while j < body.len() {
                    if body[j..].starts_with(&pat) {
                        out.push(digit);
                        j += len;
                        count += 1;
                    } else {
                        out.push(body[j]);
                        j += 1;
                    }
                }
                *body = out;
            }
            p.push(pat);
            if count < 2 {
                // rentable seulement si répété, mais on garde le cas unique
                // de temps en temps (il peut devenir utile après d'autres mutations)
                if rng.below(4) != 0 {
                    return false;
                }
            }
        }
        // remplacer un appel par le corps de la fonction
        85..=94 => {
            let calls: Vec<(usize, usize)> = p
                .iter()
                .enumerate()
                .flat_map(|(f, b)| {
                    b.iter()
                        .enumerate()
                        .filter(|&(_, &c)| (b'1'..=b'9').contains(&c))
                        .map(move |(i, _)| (f, i))
                })
                .collect();
            if calls.is_empty() {
                return false;
            }
            let (f, i) = calls[rng.below(calls.len() as u32) as usize];
            let g = (p[f][i] - b'0') as usize;
            let body = p[g].clone();
            p[f].splice(i..i + 1, body);
        }
        // dupliquer un caractère (allonge une ligne droite)
        _ => {
            let Some((f, i)) = random_char(p, rng) else { return false };
            let c = p[f][i];
            p[f].insert(i, c);
        }
    }
    cleanup(p);
    true
}

/// Recuit simulé sur le programme. Score = taille + LAMBDA * (distance minimale
/// à Fry atteinte), donc un programme gagnant a pour score sa taille. Retourne
/// le meilleur programme GAGNANT rencontré (au pire `init`).
pub fn anneal(
    world: &World,
    init: &Prog,
    (lambda, t0, t1): (f64, f64, f64),
    deadline: Instant,
    rng: &mut Rng,
) -> Prog {
    let block = param("BLOCK", 50.0) as u32;
    let score = |p: &Prog, e: Eval| prog_cost(p) as f64 + lambda * e.best as f64;

    let mut best = init.clone();
    let mut best_cost = prog_cost(&best);
    let mut cur = init.clone();
    let mut cand = init.clone();
    let mut cur_trace = Trace::new();
    let mut cand_trace = Trace::new();
    let e0 = world.run_from(&cur, &Trace::new(), &cur.clone(), &mut cur_trace);
    let mut cur_score = score(&cur, e0);

    let start = Instant::now();
    let total = deadline.saturating_duration_since(start).as_secs_f64().max(1e-6);
    let mut iter: u64 = 0;
    let mut temp = t0;
    let (mut acc, mut accwin, mut seen_win) = (0u64, 0u64, 0u64);
    loop {
        if iter % 64 == 0 {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let frac = (now - start).as_secs_f64() / total;
            temp = t0 * (t1 / t0).powf(frac);
        }
        iter += 1;

        cand.clone_from(&cur);
        if !mutate(&mut cand, rng, block) {
            continue;
        }
        let e = world.run_from(&cur, &cur_trace, &cand, &mut cand_trace);
        let s = score(&cand, e);
        // un programme gagnant plus court est gardé même si le recuit le refuse
        seen_win += e.win as u64;
        if e.win && prog_cost(&cand) < best_cost {
            best_cost = prog_cost(&cand);
            best = cand.clone();
        }
        let delta = s - cur_score;
        if delta <= 0.0 || rng.unit() < (-delta / temp).exp() {
            acc += 1;
            accwin += e.win as u64;
            std::mem::swap(&mut cur, &mut cand);
            cur_score = s;
            std::mem::swap(&mut cur_trace, &mut cand_trace);
        }
    }
    eprintln!(
        "anneal: {} iterations, {} -> {} (wins seen {}, accepted {}, accepted wins {})",
        iter, prog_cost(init), best_cost, seen_win, acc, accwin
    );
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_from;
    use crate::sim;

    fn random_prog(rng: &mut Rng) -> Prog {
        let nf = 1 + rng.below(5) as usize;
        let mut p: Prog = vec![];
        for _ in 0..nf {
            let len = 1 + rng.below(8) as usize;
            let body = (0..len)
                .map(|_| match rng.below(4 + nf as u32 - 1) {
                    0 => b'U',
                    1 => b'D',
                    2 => b'L',
                    3 => b'R',
                    k => b'0' + (k - 3) as u8,
                })
                .collect();
            p.push(body);
        }
        p
    }

    /// Le simulateur rapide (pile, détection de cycle, reprise sur préfixe)
    /// doit donner exactement le même verdict que `sim::wins`.
    #[test]
    fn fast_sim_matches_reference() {
        for map in [6, 10, 24] {
            let text = std::fs::read_to_string(format!("tests/{map}.txt")).unwrap();
            let (board, state) = load_from(&mut text.as_bytes());
            let world = World::new(&board, &state);
            let mut rng = Rng::new(map);
            let path = world.random_shortest_path(&mut rng, 200);
            let valid: Prog = vec![path.into_bytes()];
            let mut prev = valid.clone();
            let mut prev_trace = Trace::new();
            world.run_from(&prev.clone(), &Trace::new(), &prev, &mut prev_trace);
            let mut wins = 0;
            for it in 0..30000 {
                let mut p = prev.clone();
                if it % 500 == 0 {
                    p = if rng.below(2) == 0 { random_prog(&mut rng) } else { valid.clone() };
                } else {
                    mutate(&mut p, &mut rng, 50);
                }
                let s = to_string(&p);
                let reference = sim::wins(&board, &state, &s);
                let full = world.run(&p);
                assert_eq!(full.win, reference, "run: {s}");
                let mut t = Trace::new();
                let e = world.run_from(&prev, &prev_trace, &p, &mut t);
                assert_eq!(e.win, reference, "run_from: {} -> {s}", to_string(&prev));
                assert_eq!(e.best, full.best, "best: {} -> {s}", to_string(&prev));
                wins += reference as usize;
                // marche aléatoire qui reste surtout parmi les programmes gagnants
                if reference || rng.below(10) == 0 {
                    prev = p;
                    prev_trace = t;
                }
            }
            eprintln!("map {map}: {wins} wins");
        }
    }
}
