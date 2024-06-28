from collections import deque
from itertools import product
import sys
from board import Board, States

# def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, initial_state: States, avoid_switch: bool = False) -> States | None:
#     queue = deque([(start, initial_state.clone())])
#     visited = set()
#     while queue:
#         position, state = queue.pop()
#         if position == target:
#             return state
        
#         if position in visited:
#             continue

#         visited.add(position)

#         row, col = position
#         for row_offset, col_offset, direction in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
#             new_row, new_col = row + row_offset, col + col_offset
#             if board.is_blocked(state, new_row, new_col):
#                 continue

#             if avoid_switch and (new_row, new_col) != target and (new_row, new_col) in board.switches:
#                 continue

#             new_state = state.clone()
#             new_state.actions += direction
#             board.move_on(new_state, new_row, new_col)
#             queue.append(((new_row, new_col), new_state))
        
#     return None


def hashlist(arr: list[bool]) -> int:
    s = 0
    for state in arr:
        s = s * 2 + state
    return s


done = set()


def bfs(start: tuple[int, int], target: tuple[int, int], board: Board, magnetic_fields: list[bool], balls: list[tuple[int, int]]) -> States:
    global done

    foo = (*start, *target, hashlist(magnetic_fields))
    if foo in done:
        return None
    done.add(foo)

    state = States(
        magnetic_fields=magnetic_fields,
        balls=balls,
        steps=[]
    )

    # print(start, target, state.magnetic_fields, file=sys.stderr)

    queue = deque([(start, state.clone())])
    visited = set()
    while queue:
        position, state = queue.pop()
        if position == target:
            yield state
            break
        
        if position in visited:
            continue

        visited.add(position)

        row, col = position
        for row_offset, col_offset, direction in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
            new_row, new_col = row + row_offset, col + col_offset

            pole_index = board.get_magnetic_field(new_row, new_col)
            if pole_index >= 0 and not state.magnetic_fields[pole_index]:
                new_mag = state.magnetic_fields.copy()
                new_mag[pole_index] = True
                yield from bfs(start, target, board, new_mag, balls)
                # state.steps.append(pole_index)

            if board.is_blocked(state, new_row, new_col):
                continue
            
            # cannot walk on another switch
            if (new_row, new_col) != target and (new_row, new_col) in board.switches:
                continue

            new_state = state.clone()
            new_state.actions += direction
            board.move_on(new_state, new_row, new_col)
            queue.append(((new_row, new_col), new_state))


def graph(board: Board, start: tuple[int, int], target: tuple[int, int], magnetic_fields: list[bool], balls: list[tuple[int, int]]):
    ans = []
    for t in [target, *board.switches]:
        ans.extend(list(bfs(start, t, board, magnetic_fields, balls)))

    for s, t in product(board.switches, board.switches):
        if s == t:
            continue
        ans.extend(list(bfs(s, t, board, magnetic_fields, balls)))
    
    for s in board.switches:
        ans.extend(list(bfs(s, target, board, magnetic_fields, balls)))

    print(*ans, sep="\n", file=sys.stderr)

