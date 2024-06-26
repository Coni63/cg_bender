import sys
from board import Board, States
from collections import deque

def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States) -> States | None:
    queue = deque([(start, initial_state)])
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

            new_state = state.clone()
            new_state.actions += direction
            board.move_on(new_state, new_row, new_col)
            queue.append(((new_row, new_col), new_state))
        
    return None


def bfs_switches(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States, switched_done: list[int, int]) -> States | None:
    # print("bfs_switches", start, target, file=sys.stderr)
    state = bfs(start, target, board, initial_state)
    if state is not None:
        return state
        
    for switch in board.switches:
        if switch in switched_done:
            continue

        inter_state = bfs(start, switch, board, initial_state)
        if inter_state is None:
            continue
    
        new_switches = switched_done[:] + [switch]
        return bfs_switches(switch, board.target, board, inter_state, new_switches)
        
    return None