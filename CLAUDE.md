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

1. **`board.rs`** : représentation de la grille, simplifications :
   - `simplify_deadend` : supprime récursivement les culs-de-sac (cases
     vides à 3 murs adjacents, hors start/target) → réduit la taille du
     graphe de recherche.
   - `simplify_balls` : une ball coincée dans un coin (2 murs adjacents en
     diagonale) est convertie en mur → simplifie `bfs`/`find_path` en
     supprimant un état inutile à tracker.

2. **`bfs.rs`** :
   - `bfs(start, target, ...)` : BFS classique point-à-point sur la grille
     simplifiée (une case = un nœud, ignore l'état des switches en transit).
   - `prepare(board, state)` : précalcule, pour chaque paire (départ,
     cible) parmi {start, target, tous les switches, tous les champs
     magnétiques}, le chemin BFS entre les deux → **graphe compressé** où
     les nœuds sont uniquement les points d'intérêt, les arêtes sont des
     séquences de mouvements précalculées.
   - `find_path(graph, board, state)` : Dijkstra (`BinaryHeap` avec
     `State::Ord` inversé = min-heap sur la longueur du chemin) sur ce
     graphe compressé. État = `(position, bitmask des champs actifs)`,
     dédupliqué par hash. Garde jusqu'à **100 chemins candidats**
     atteignant la cible (ou coupe à 900ms), pas juste le premier trouvé.

3. **`encode.rs`** (le plus retravaillé) :
   - `find_substrings` : énumère les sous-chaînes candidates (fenêtre de
     taille max configurable, actuellement 30) pour devenir des macros.
     **Autorise les chiffres** dans les motifs (permet la composition
     hiérarchique de macros — une macro peut être construite à partir
     d'une autre déjà définie).
   - `sort_substring` : trie les candidats par gain réel
     `gain = len × (1 - occurrences) + 2` (négatif = rentable).
   - `compress_greedy_from` : passe **gloutonne sans branchement** —
     prend systématiquement le meilleur candidat, répète jusqu'à
     `max_macros` ou plus rien à gagner. Quasi gratuit en temps de calcul.
   - `compress_hybrid` : **branche sur les 2 premiers niveaux** avec un
     beam de 5, complète chaque branche en glouton pur, garde le meilleur
     résultat. Deadline absolue passée en paramètre (jamais recréée par
     appel, pour éviter la multiplication par le nombre de candidats).
   - `extract_recursive_tail` : détecte un motif périodique en toute fin
     de chemin et le transforme en fonction récursive auto-appelante
     (`motif + appel de soi-même`), en évaluant si le gain net est positif.
   - **Pipeline en 2 phases dans `main()`** :
     - Phase 1 (`quick_compress` = glouton pur) sur les 100 candidats
       BFS, tri par taille compressée obtenue.
     - Phase 2 (`deep_compress` = hybride avec branchement) seulement sur
       le **top-K** (actuellement 5) des candidats les plus prometteurs
       identifiés en phase 1, avec toute la deadline restante dédiée à eux.

## Historique des scores (cas de test internes, cumul des chars)

| Étape | Score |
|---|---|
| Baseline (beam=2, filtre `is_alphabetic`, sans deadline globale) | 2750 |
| Fix filtre (autoriser chiffres dans les motifs) + beam=2 | 2463 |
| Compression hybride avec cache + deadline globale (beam=4, branch total) | 2450 (mais timeout intermittent) |
| Passage à `compress_greedy` pur (rapide mais sans branchement) | 2540 (régression) |
| `compress_hybrid` (branch_depth=2, beam=5) avec deadline globale correcte | 2457 |
| Split phase 1 (glouton sur tous) + phase 2 (hybride sur top-5) | **2442 (actuel)** |

Best du top 10 CodinGame (Rust) : ~1596-1675. Marge encore importante.

## Contraintes connues de la plateforme

- Rust n'est **pas compilé avec optimisations** sur CodinGame → perf
  notablement moins bonne qu'en C++/Java/C# à algorithme équivalent.
  Le budget temps réel disponible est **~0.8s** (au lieu du 1s habituel,
  marge de sécurité à garder).
- Les meilleurs scores sont en C++/Java/C#, mais du Rust apparaît quand
  même dans le top 10 → la marge de progression vient de l'algorithme,
  pas uniquement du langage.

## Pistes déjà explorées

- ✅ Fix du filtre `is_alphabetic` → composition hiérarchique de macros.
- ✅ Deadline globale (absolue, calculée une fois dans `main()`) au lieu
  d'une deadline relative recréée à chaque appel — c'était la cause de
  deux régressions timeout successives.
- ✅ Split glouton rapide (filtre) / hybride profond (top-K seulement) —
  meilleur usage du budget temps que brancher un peu partout.
- ⚠️ Queue récursive (`extract_recursive_tail`) : implémentée mais
  **impact réel non mesuré isolément**. À vérifier : combien de cas de
  test en bénéficient vraiment, et de combien.

## Pistes à tester ensuite (priorité suggérée)

Avec les ~250ms libérés par le split phase 1/phase 2, la marge de temps
n'est plus le facteur limitant — la priorité doit basculer vers la
**qualité du chemin généré**, pas uniquement sa compression.

### 1. Tuning des paramètres actuels (rapide, à faire en premier)
Avant tout nouveau chantier, vérifier combien de marge de temps reste
réellement disponible et pousser les curseurs existants un par un :
- `TOP_K` (5 → 10/15) dans `main()`
- `BEAM_WIDTH` dans `deep_compress` (5 → 8)
- `BRANCH_DEPTH` (2 → 3)
- `WINDOW` dans `find_substrings` (30 → plus grand, si le temps le permet)

Mesurer l'impact de chaque changement isolément avant de les cumuler.

### 2. Mesurer l'impact réel de `extract_recursive_tail`
Ajouter un `eprintln!` pour savoir : sur combien de cas de test un motif
récursif est trouvé, et quel gain ça apporte. Si le taux de déclenchement
est faible, c'est un signal que les chemins générés ne se terminent pas
naturellement par une ligne droite/motif cyclique → lien direct avec le
point 3 ci-dessous (il faudrait *forcer* cette propriété plutôt que
compter sur le hasard).

### 3. Optimiser la génération du chemin lui-même (le plus gros chantier, probablement le plus payant)

Le chemin vient actuellement d'un Dijkstra qui minimise la **longueur
brute**. Mais l'objectif réel est la taille **compressée**. Deux chemins
de même longueur peuvent compresser très différemment. Pistes concrètes :

- **Chemins alternatifs de même coût entre deux nœuds du graphe** :
  `bfs()` ne retourne qu'un seul chemin par paire de nœuds. Sur une
  grille, plusieurs chemins de longueur minimale existent souvent (ex:
  "droite puis haut" vs "haut puis droite"). Modifier `bfs` pour
  retourner plusieurs variantes optimales, puis choisir lors de
  l'assemblage celle qui réutilise un motif déjà présent ailleurs dans
  le trajet global plutôt que la première trouvée arbitrairement.

- **Ordre de visite des switches** : si plusieurs ordres donnent un coût
  total égal ou proche (façon TSP), certains génèrent des trajets plus
  symétriques/répétitifs. Énumérer quelques ordres alternatifs dans
  `find_path` plutôt que de garder l'unique ordre exploré par le
  Dijkstra actuel.

- **Forcer une fin de trajet compressible** : plutôt que d'espérer que
  `extract_recursive_tail` trouve un motif par chance, orienter la
  recherche de chemin pour que le dernier segment vers la cible soit une
  ligne droite ou un motif cyclique simple quand c'est géométriquement
  possible.

- **Élargir le pool de candidats gardés par `find_path`** : actuellement
  les 100 chemins de plus courte longueur brute. Envisager d'accepter
  aussi des chemins légèrement plus longs (+X%) qui seraient plus
  répétitifs — la phase 1 (glouton rapide) sert déjà de filtre pour
  ne pas payer cher l'exploration de candidats supplémentaires.

C'est un chantier qui touche `bfs.rs` (pas seulement `encode.rs`), donc
à traiter comme une session à part plutôt qu'un ajustement rapide.

### 4. Vérifier la validité stricte des chemins (filet de sécurité, non urgent)
`bfs()` (point-à-point, utilisé dans `prepare`) ne bloque que sur
`Cell::Wall` — un switch/champ intermédiaire *en transit* (ni départ ni
cible du segment) est traversé sans déclencher son effet dans le calcul
du graphe, alors qu'en jeu réel marcher dessus togglerait toujours l'état.
Risque théorique de divergence entre l'état supposé par le solveur et
l'état réel du jeu. Pas de cas confirmé à ce jour, mais une simulation de
validation complète du chemin final avant compression serait un bon
filet de sécurité si des bugs difficiles à expliquer apparaissent.

## Fichiers concernés

- `src/board.rs` : représentation + simplifications de la grille.
- `src/bfs.rs` : recherche de chemin (BFS point-à-point + Dijkstra sur graphe
  compressé). **Cible principale du point 3.**
- `src/encode.rs` : compression du chemin en programme avec fonctions.
- `src/loader.rs` : parsing de l'input CodinGame.
- `src/main.rs` : orchestration, deadlines, pipeline 2 phases.

### 5. Benchmarking

Utilise `full_bench.py` to test the code agains the 30 public tests cases. There is 3 options:

```
python full_bench.py python
python full_bench.py rust
python full_bench.py rust_release
```

But for rust, it requires a build from cargo 
```
cargo build
cargo build --release
```