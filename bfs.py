from functools import cache
import sys
import time
from board import Board, States
from collections import deque
import heapq


@cache
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


def bfs_switches(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States) -> States | None:    
    total_time_bfs = 0
    total_count_bfs = 0

    queue = []
    heapq.heappush(queue, (initial_state.fitness(), (start, target, initial_state.clone())))

    visited = set()
    while queue:
        # print(len(queue), file=sys.stderr)

        s, t, state = heapq.heappop(queue)[1]

        # check if we can reach the target directly
        tic = time.time()
        next_state = bfs(s, t, board, state, avoid_switch=True)
        tac = time.time()
        total_count_bfs += 1
        total_time_bfs += tac - tic

        if next_state is not None:
            print("Total time spent in bfs:", total_time_bfs, file=sys.stderr)
            print("Total bfs calls:", total_count_bfs, file=sys.stderr)
            return next_state

        for i, switch in enumerate(board.switches):
            # skip the switch we are on
            if switch == start:
                continue

            # Avoid loops by checking if we already tried this move with the same initial state
            hash_state = (*s, *switch, hash(state))
            if hash_state in visited:
                continue
            visited.add(hash_state)

            # Try to reach the switch
            tic = time.time()
            inter_state = bfs(s, switch, board, state, avoid_switch=True)
            tac = time.time()
            total_count_bfs += 1
            total_time_bfs += tac - tic

            if inter_state is None:
                continue
            
            inter_state.steps.append(i)
            # From the switch, try to reach the target from the switch
            heapq.heappush(queue, (inter_state.fitness(), (switch, target, inter_state)))
    
    print("Total time spent in bfs:", total_time_bfs, file=sys.stderr)
    print("Total bfs calls:", total_count_bfs, file=sys.stderr)
    return None