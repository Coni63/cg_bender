import sys
from board import Board, States
from collections import deque

r = sys.getrecursionlimit()
sys.setrecursionlimit(r * 2)

def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States, avoid_switch: bool = False) -> States | None:
    queue = deque([(start, initial_state.clone())])
    visited = set()
    while queue:
        position, state = queue.pop()
        if position == target:
            return state
        
        if position in visited:
            continue

        visited.add(position)

        row, col = position
        for row_offset, col_offset, direction in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
            new_row, new_col = row + row_offset, col + col_offset
            if board.is_blocked(state, new_row, new_col):
                continue

            if avoid_switch and (new_row, new_col) != target and (new_row, new_col) in board.switches:
                continue

            new_state = state.clone()
            new_state.actions += direction
            board.move_on(new_state, new_row, new_col)
            queue.append(((new_row, new_col), new_state))
        
    return None


def bfs_switches(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States, switched_done: set[int, int, int], depth=0) -> States | None:
    # check if we can reach the target directly
    state = bfs(start, target, board, initial_state, avoid_switch=False)
    if state is not None:
        return state
    
    if depth > 50:
        return None
    
    # there is no direct path to the target, we need to test all the switches
    for i, switch in enumerate(board.switches):

        #skip the switch we are on
        if switch == start:
            continue

        # Avoid loops by checking if we already tried this move with the same initial state
        # hash_state = (*start, *switch, hash(initial_state))
        hash_state = (*start, *switch, hash(initial_state))
        if hash_state in switched_done:
            continue
        
        new_switches = switched_done.copy()
        new_switches.add(hash_state)

        # Try to reach the switch
        inter_state = bfs(start, switch, board, initial_state, avoid_switch=True)
        if inter_state is None:
            continue
        
        # From the switch, try to reach the target from the switch
        ans = bfs_switches(switch, board.target, board, inter_state, new_switches, depth=depth+1)
        if ans is not None:
            return ans
        
    return None
