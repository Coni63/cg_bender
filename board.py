import sys

class States: 
    def __init__(self, magnetic_fields: list[bool], balls: tuple[int, int], steps: list[int], actions: str = ""):
        self.magnetic_fields = magnetic_fields
        self.balls = balls
        self.actions = actions
        self.steps = steps

    def clone(self):
        return States(self.magnetic_fields[:], self.balls[:], self.steps[:], self.actions)
    
    def __hash__(self) -> int:
        s = 0
        for state in self.magnetic_fields:
            s = s * 2 + state
        return s
    
    def fitness(self) -> int:
        return len(self.actions)
    
    def __lt__(self, other):
        return len(self.actions) < len(other.actions)

    def __le__(self, other):
        return len(self.actions) <= len(other.actions)
    
    def __eq__(self, other):
        return self.actions == other.actions
    
    def __ne__(self, other):
        return self.actions != other.actions
    
    def __gt__(self, other):
        return len(self.actions) > len(other.actions)
    
    def __ge__(self, other):
        return len(self.actions) >= len(other.actions)


class Board:
    def __init__(self, 
                 board: list[list[bool]], 
                 width: int, 
                 height: int, 
                 start: tuple[int, int], 
                 target: tuple[int, int], 
                 switches: list[tuple[int, int]],
                 magnetic_fields: list[tuple[int, int]]
                 ):
        self.board = board
        self.width = width
        self.height = height
        self.start = start
        self.target = target
        self.switches = switches
        self.magnetic_fields = magnetic_fields

    def show(self, state: States = None):
        print(*["".join(["#."[x] for x in row]) for row in self.board], sep="\n", file=sys.stderr)
        print("start:", self.start, file=sys.stderr)
        print("target:", self.target, file=sys.stderr)
        if state:
            print("balls:", state.balls, file=sys.stderr)
        print("switches:", self.switches, file=sys.stderr)
        if state:
            arr = [(r, c, s) for (r, c), s in zip(self.magnetic_fields, state.magnetic_fields)]
            print("magnetic_fields:", arr, file=sys.stderr)
        else:
            print("magnetic_fields:", self.magnetic_fields, file=sys.stderr)

    def is_blocked(self, state: States, row: int, col: int):
        is_free = self.board[row][col]
        if not is_free:
            return True
        
        # TODO: For now we consider that balls cannot be moved
        for ball in state.balls:
            if ball == (row, col):
                return True
        
        # magnetic fields are blocking if state is 1
        for (r, c), state in zip(self.magnetic_fields, state.magnetic_fields): 
            if (r, c, state) == (row, col, True):
                return True
        
        return False
    
    def move_on(self, state: States, row: int, col: int):
        for i, pos in enumerate(self.switches):
            if (row, col) == pos:
                # toggle the magnetic field
                state.magnetic_fields[i] = not state.magnetic_fields[i]

    def simplify_board(self):
        directions = [(-1, 0), (1, 0), (0, -1), (0, 1)]

        improved = True
        while improved:
            improved = False
            for row in range(1, self.height-1):
                for col in range(1, self.width-1):
                    p = (row, col)
                    if p in self.switches:
                        continue

                    if p in self.magnetic_fields:
                        continue

                    if p == self.start or p == self.target:
                        continue

                    if not self.board[row][col]:
                        continue
                    
                    wall = 0
                    for drow, dcol in directions:
                        wall += not self.board[row + drow][col + dcol]

                    if wall >= 3:
                        self.board[row][col] = False
                        improved = True