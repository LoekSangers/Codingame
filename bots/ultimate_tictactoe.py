import datetime
from random import choice
from math import log, sqrt
import sys

class Board(object):
    # 1 for opponent
    # -1 for me
    def start(self):
        return tuple([tuple([0 for _ in range(9)]) for _ in range(10)])

    def current_player(self, state):
        total = 0
        for row in state[0:9]:
            total += sum(row)
        
        if total == 0:
            return -1
        else:
            return -total


    def next_state(self, state, play):
        new_state = []
        for row in state:
            new_state.append(list(row))
        
        new_state[play[0]][play[1]] = self.current_player(state)

        # check whether inner board won, if so update outer board state
        start_r = play[0] - play[0]%3
        start_c = play[1] - play[1]%3

        winner = self.small_winner(new_state[start_r][start_c:start_c+3] +
            new_state[start_r + 1][start_c:start_c+3] + 
            new_state[start_r + 2][start_c:start_c+3])

        if winner != 0:
            new_state[9][start_r + start_c // 3] = winner

        return tuple(tuple(el) for el in new_state)

    def legal_plays(self, state_history):

        if len(state_history) == 1:
            return [(r, c) for r in range(9) for c in range(9)]

        curr = state_history[-1]
        last = state_history[-2]

        play = (-1,-1)
        moves = []

        for row in range(9):
            if sum(curr[row]) != sum(last[row]):
                for col in range(9):
                    if curr[row][col] != last[row][col]:
                        play = (row, col)
                        break
                break

        inner_r = row % 3
        inner_c = col % 3

        plays = [(r, c) for r in range(inner_r*3, inner_r*3+3) for c in range(inner_c*3, inner_c*3 + 3) if curr[r][c] == 0]

        # is target tictactoe not won yet
        if curr[9][inner_r * 3+ inner_c] == 0 and len(plays) > 0:
            return plays
        else:
            return [(r, c) for r in range(9) for c in range(9) if curr[r][c] == 0]


    def winner(self, state_history):

        return self.small_winner(state_history[-1][9])


    def small_winner(self, b):
        for i in range(3):
            if b[i*3] > 0 and b[i*3] == b[i*3 + 1] and b[i*3] == b[i*3 + 2]:
                return b[i*3]

            if b[i] > 0 and b[i] == b[3+i] and b[i] == b[6+i]:
                return b[i]

        if b[0] > 0 and b[0] == b[4] and b[0] == b[8]:
            return b[0]
        if b[2] > 0 and b[2] == b[4] and b[2] == b[6]:
            return b[2]

        else:
            if any(x == 0 for x in b):
                return 0 # not done yet
            return 2 # Tie and no more possible moves

class MonteCarlo(object):
    def __init__(self, board, **kwargs):
        self.board = board
        self.states = []

        self.C = kwargs.get('C', 1.4)
        seconds = kwargs.get('time', 1)
        self.calculation_time = datetime.timedelta(seconds=seconds)
        self.max_moves = kwargs.get('max_moves', 100)

        self.wins = {}
        self.plays = {}

    def update(self, state):
        self.states.append(state)

    def update_calculation_time(self, seconds):
        self.calculation_time = datetime.timedelta(seconds=seconds)

    def get_play(self):
        self.max_depth = 0
        state = self.states[-1]
        player = self.board.current_player(state)
        legal = self.board.legal_plays(self.states[:])

        # Bail out early if there is no real choice to be made.
        if not legal:
            return
        if len(legal) == 1:
            return legal[0]

        games = 0
        begin = datetime.datetime.utcnow()
        while datetime.datetime.utcnow() - begin < self.calculation_time:
            self.run_simulation()
            games += 1

        moves_states = [(p, self.board.next_state(state, p)) for p in legal]

        # Pick the move with the highest percentage of wins.
        percent_wins, move = max(
            (self.wins.get((player, S), 0) /
             self.plays.get((player, S), 1),
             p)
            for p, S in moves_states
        )

        return move

    def run_simulation(self):
        # A bit of an optimization here, so we have a local
        # variable lookup instead of an attribute access each loop.
        plays, wins = self.plays, self.wins

        visited_states = set()
        states_copy = self.states[:]
        state = states_copy[-1]
        player = self.board.current_player(state)

        expand = True
        for t in range(1, self.max_moves + 1):
            legal = self.board.legal_plays(states_copy)
            moves_states = [(p, self.board.next_state(state, p)) for p in legal]

            if all(plays.get((player, S)) for p, S in moves_states):
                # If we have stats on all of the legal moves here, use them.
                log_total = log(
                    sum(plays[(player, S)] for p, S in moves_states))
                value, move, state = max(
                    ((wins[(player, S)] / plays[(player, S)]) +
                     self.C * sqrt(log_total / plays[(player, S)]), p, S)
                    for p, S in moves_states
                )
            else:
                # Otherwise, just make an arbitrary decision.
                move, state = choice(moves_states)

            states_copy.append(state)

            # `player` here and below refers to the player
            # who moved into that particular state.
            if expand and (player, state) not in plays:
                expand = False
                plays[(player, state)] = 0
                wins[(player, state)] = 0
                if t > self.max_depth:
                    self.max_depth = t

            visited_states.add((player, state))

            player = self.board.current_player(state)
            winner = self.board.winner(states_copy)
            if winner:
                break

        for player, state in visited_states:
            if (player, state) not in plays:
                continue
            plays[(player, state)] += 1
            if player == winner:
                wins[(player, state)] += 1

"""
opponent_row, opponent_col = [int(i) for i in input().split()]

valid_action_count = int(input())
valid_moves = []
for i in range(valid_action_count):
    row, col = [int(j) for j in input().split()]
    valid_moves.append((row,col))
"""

board = Board()
mcts = MonteCarlo(board, time=.9, C=1.4, max_moves=100)
mcts.update(board.start())

"""
if opponent_row != -1:
    mcts.update(board.next_state(mcts.states[-1], (opponent_row, opponent_col)))

row, col = mcts.get_play()
print("{} {}".format(row, col))

mcts.update_calculation_time(.08)
# game loop
while True:
    opponent_row, opponent_col = [int(i) for i in input().split()]

    valid_action_count = int(input())
    valid_moves = []
    for i in range(valid_action_count):
        row, col = [int(j) for j in input().split()]
        valid_moves.append((row,col))

    row, col = mcts.get_play()    
    mcts.update(board.next_state(mcts.states[-1], (row, col)))
    print("{} {}".format(row, col))
"""
print(board.legal_plays(mcts.states))
mcts.update(board.next_state(mcts.states[-1], mcts.get_play()))
mcts.update(board.next_state(mcts.states[-1], (7,7)))
print(board.legal_plays(mcts.states))
mcts.update(board.next_state(mcts.states[-1], mcts.get_play()))
mcts.update(board.next_state(mcts.states[-1], (7,1)))
print(board.legal_plays(mcts.states))
#print(mcts.plays)