
from board import Board, States


def load_input():
    width, height = [int(i) for i in input().split()]
    balls = []
    board = []
    for i in range(height):
        row = []
        for j, char in enumerate(input()):
            if char == '+':
                balls.append((i, j))
                row.append(True)
            elif char == '#':
                row.append(False) # wall
            else:
                row.append(True)  # free space 
        board.append(row)

    start_col, start_row = [int(i) for i in input().split()]
    start = (start_row, start_col)
    target_col, target_row = [int(i) for i in input().split()]
    target = (target_row, target_col)

    switches, magnetic_fields, magnetic_fields_states = [], [], []
    switch_count = int(input())
    for i in range(switch_count):
        # initial_state: 1 if blocking, 0 otherwise
        switch_x, switch_y, block_x, block_y, initial_state = [int(j) for j in input().split()]
        switches.append((switch_y, switch_x))
        magnetic_fields.append((block_y, block_x))
        magnetic_fields_states.append(initial_state == 1)

    board = Board(board, width, height, start, target, switches, magnetic_fields)
    states = States(magnetic_fields_states, balls, "")

    return board, states

