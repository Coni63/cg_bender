from bfs import bfs
from loader import load_input
from board import Board


board, state = load_input()
board.show()

path = bfs(board.start, board.target, board, state)
print(path)