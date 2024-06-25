from bfs import bfs, bfs_switches
from loader import load_input
from board import Board


board, state = load_input()
board.show(state)

# simple check
# path, _ = bfs(board.start, board.target, board, state)

# complex solving
path, _ = bfs_switches(board.start, board.target, board, state, [])
    
print(path)