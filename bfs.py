import sys
from board import Board, States
from collections import deque

def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States) -> str | None:
    queue = deque([(start, "", initial_state)])
    visited = set()
    while queue:
        position, actions, state = queue.pop()
        if position == target:
            return actions
        
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
        
    return None