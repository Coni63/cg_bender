import sys
from board import Board, States
from collections import deque

def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States) -> tuple[str | None, States | None]:
    queue = deque([(start, "", initial_state)])
    visited = set()
    while queue:
        position, actions, state = queue.pop()
        if position == target:
            return actions, state
        
        if position in visited:
            continue

        visited.add(position)

        row, col = position
        for row_offset, col_offset, direction in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
            new_row, new_col = row + row_offset, col + col_offset
            if board.is_blocked(state, new_row, new_col):
                continue

            new_state = state.clone()
            board.move_on(new_state, new_row, new_col)
            queue.append(((new_row, new_col), actions + direction, new_state))
        
    return None, None


def bfs_switches(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States, switched_done: list[int, int]) -> tuple[str | None, States | None]:
    # print("bfs_switches", start, target, file=sys.stderr)
    path, state = bfs(start, target, board, initial_state)
    if path is not None:
        return path, state
    
    for switch in board.switches:
        if switch in switched_done:
            continue

        path, state = bfs(start, switch, board, initial_state)
        new_switches = switched_done[:] + [switch]
        if path is not None:
            p, s = bfs_switches(switch, board.target, board, state, new_switches)
            return path + p, s
    return None, None