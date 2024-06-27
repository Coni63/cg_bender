import sys
import time
from bfs import bfs_switches
from loader import load_input

board, state = load_input()

start = time.time()
board.simplify_board()
board.show(state)

state = bfs_switches(board.start, board.target, board, state)
if state:
    print(state.actions)
else:
    print("")
print("Time:", time.time() - start, file=sys.stderr)