# Bender4 (CodinGame) — Notes de travail

## Sujet

Bender doit s'échapper d'un labyrinthe pour rejoindre Fry. Contraintes du jeu :

- Grille avec murs, cases vides, **switches** et **champs magnétiques**.
- Chaque switch contrôle un champ magnétique (le switch et le champ peuvent
  être n'importe où sur la carte, pas forcément adjacents). Entrer sur la
  case du switch **toggle** l'état du champ associé (ON <-> OFF).
- Un champ magnétique actif est **létal** s'il est traversé.
- Il y a des **garbage balls** (murs mobiles) : Bender les pousse en marchant
  dedans, si rien ne bloque derrière. Une ball poussée sur un switch le
  toggle aussi. Une ball poussée sur Fry le tue (perte).
- **Chaque map est garantie solvable sans bouger aucune ball** — donc les
  balls peuvent être traitées comme des murs pour une solution correcte
  (pas forcément optimale).
- Mouvement dans les 4 directions, un mur bloque (Bender reste sur place,
  pas d'erreur).
- **Le vrai objectif n'est PAS de minimiser le nombre de mouvements**, mais
  la **taille du programme compressé** envoyé au juge. L'énoncé précise que
  la solution optimale *sans fonctions* dépasse déjà 150 caractères — le
  score se joue donc essentiellement sur la qualité de la compression.

### Langage de sortie (fonctions)

- Programme = une chaîne d'actions (`U`/`D`/`L`/`R`) + définitions de
  fonctions séparées par `;`.
- Une fonction peut appeler une autre fonction (y compris récursivement,
  y compris elle-même) via un chiffre.
- **Les macros/fonctions sont numérotées `1` à `9`** (confirmé dans le
  moteur du jeu : `if (execute >= '1' && execute <= '9')`) → **8 slots
  maximum en pratique** si on réserve un slot pour une fonction récursive
  spéciale (voir plus bas), ou 9 sinon.
- Une fonction s'exécute caractère par caractère ; quand elle est épuisée,
  l'exécution reprend dans l'appelant juste après l'appel.
- **Point clé exploité** : l'exécution s'arrête dès que Fry est atteint,
  peu importe où on en est dans la pile d'appels. Donc une fonction
  récursive du style `F = "RF"` peut couvrir une distance arbitrairement
  longue pour un coût fixe de 2-3 caractères, tant qu'aucun champ actif
  n'est traversé en cas de "dépassement" après la cible (les murs ne
  posent pas de problème, Bender reste juste sur place).

Source du moteur : https://github.com/eulerscheZahl/Bender4

## Architecture actuelle du solveur

Idée centrale : le juge **exécute** le programme et regarde si Bender atteint
Fry ; il n'exige pas de reproduire un chemin. On cherche donc directement dans
l'espace des programmes, avec un simulateur exact comme juge.

1. **`search.rs`** (le cœur) :
   - `World::new` : carte brute (non simplifiée) + **table de distance exacte**
     `dist[masque * 441 + pos]` vers Fry, par BFS arrière sur les états
     (position, champs actifs), balls traitées comme des murs. ~3 ms en release.
   - `random_shortest_path` : tire des plus courts chemins au hasard en suivant
     la table (biais « garder la direction » réglable) → chemins de départ variés.
   - `exec` / `run_from` : simulateur rapide, vérifié contre `sim::wins` par
     le test `fast_sim_matches_reference`. Optimisations : bitset + hash Zobrist
     des balls, pile réutilisée, **détection de cycle de Brent** sur les appels
     (même état + même fonction appelée sans redescendre dans la pile = boucle
     infinie prouvée → perdu tout de suite), **reprise sur préfixe** (`Trace` :
     état sauvegardé avant chaque token du main ; un programme muté n'est
     resimulé qu'à partir du premier token dont l'exécution touche la partie
     modifiée).
   - `anneal` : recuit simulé sur le programme. Score = taille +
     λ × (distance minimale à Fry atteinte pendant l'exécution). Mutations :
     remplacer/supprimer/insérer/échanger/dupliquer un caractère, extraire une
     sous-chaîne en fonction, inliner un appel, et surtout **remplacer un bloc
     de 1-4 caractères par 0-4 tokens aléatoires** (poids BLOCK=50, gros gain).
     Tout programme gagnant plus court croisé est gardé, même s'il est refusé.
   - `best_main` : fonctions fixées, BFS sur les états du jeu (un token = un
     coup ou un appel) → main optimal. Gain rare, gardé en polish final.
2. **`encoder.rs`** : compression gloutonne / hybride d'un chemin en
   programme (sert seulement à fabriquer les programmes de départ du recuit).
   `split_runs` / `widen_variants` allongent les lignes droites qui finissent
   contre un mur (coups gratuits).
3. **`sim.rs`** : simulateur de référence, fidèle au moteur (vérifié ligne à
   ligne contre Interpreter/Robot/Box/Referee du dépôt Bender4). `main`
   revalide toujours le programme final avec `sim::wins`.
4. **`main.rs`** : table de distance → chemins aléatoires + compression
   (~8 % du budget) → recuit **exploration** (2 chaînes, λ=3, T 3→0.5, 70 % du
   reste) → recuit **affinage** depuis le meilleur gagnant (λ=20 : reste parmi
   les gagnants, T 1→0.1) → `best_main`.

Tous les réglages passent par `search::param` (variables d'environnement :
`BENDER_MS`, `SEED`, `E_L`, `E_T0`, `E_T1`, `E_RUNS`, `E_FRAC`, `R_L`, `R_T0`,
`R_T1`, `BLOCK`, `F_PATHS`, `F_QUICK`, `F_DEEP`, `TOP_K`) pour les essais hors
ligne ; les valeurs par défaut sont les meilleures mesurées.

### Pourquoi l'exploration marche
Avec une pénalité faible, la chaîne quitte vite les programmes gagnants et
rétrécit ; les gagnants courts sont croisés surtout pendant cette descente. Les
meilleurs résultats sont souvent des « vagabonds » récursifs quasi aléatoires
qui profitent des coups contre les murs, ex. carte 6 :
`U1;3D322231;RD44L44R;LDDRDRR;UULU` (33 chars, contre 79 avant). Le résultat
reste très dépendant de la graine (variance par carte de ±10 chars).

### Garbage balls
Elles sont traitées comme des murs pour la table de distance et les chemins de
départ (carte garantie solvable sans les bouger), mais le simulateur les gère
exactement : si une mutation pousse une ball et que ça gagne, c'est accepté.

## Historique des scores (cumul des chars, release)

| Étape | Tests (30) | Validateurs (30) |
|---|---|---|
| Baseline (beam=2, filtre `is_alphabetic`) | 2750 | |
| Split phase 1 / phase 2 | 2442 | |
| Coups perdus en fin de ligne (`widen_variants`, `tune_lengths`) | 1877-1885 | 2321 |
| Table de distance + chemins aléatoires + recuit sur le programme | 1488 | 1817 |
| Exploration + affinage, reprise sur préfixe, détection de cycle | ~1270 | ~1620 |
| Mutation par bloc (BLOCK=50) | **~1200** | **~1470-1510** |

Best du top CodinGame : ~1200 (validateurs). Avec `BENDER_MS=8000` on obtient
~1265 sur les validateurs : le temps de calcul reste un levier (×2 ≈ −50 à −100).

## Contraintes connues de la plateforme

- **Les validateurs (cartes réellement notées) ne sont PAS les 30 tests
  publics.** Leurs entrées sont publiques dans le dépôt du moteur
  (`config/test31.json` … `test60.json`), copiées dans `tests/validators/`.
  Toujours mesurer sur les deux pour éviter le sur-réglage.
- CodinGame semble compiler Rust **avec optimisations** : l'ancienne version
  fait 2321 sur les validateurs en release (≈ 2340 observé sur CG) contre 2514
  avec timeouts en debug. Le code reste sûr en debug (pas de dépassement
  arithmétique, au moins 5 chemins générés même si la table est lente).
- Limite : 1 s au premier tour. Budget interne 850 ms (~0.88 s mur, démarrage
  du process compris).

## Pistes suivantes

- **Calcul hors ligne** : les entrées des validateurs sont publiques ; lancer
  la recherche longtemps (minutes, plusieurs graines) par carte et coder en dur
  les meilleurs programmes (avec repli sur le solveur si la carte est
  inconnue) est sans doute ce que fait le haut du classement.
- Vitesse : ~3-5 µs par itération ; la mutation clone encore des `Vec<Vec<u8>>`
  (représentation à plat possible), l'exploration simule souvent ~1000 tours.
- Recuit parallèle (parallel tempering) ou redémarrages depuis le meilleur
  gagnant, voisinages plus structurés.

## Benchmarking

```
cargo build --release
python full_bench.py rust_release              # 30 tests publics
python full_bench.py rust_release validators   # 30 validateurs CG
cargo test --release                           # dont la vérification du simulateur rapide
```

Le résultat varie d'un lancement à l'autre (dépend du temps machine) : pour
comparer deux réglages, moyenner plusieurs graines (`SEED=1..4`) sur les deux
jeux de cartes ; l'écart-type de la somme est d'environ ±40 avec 4 graines.
