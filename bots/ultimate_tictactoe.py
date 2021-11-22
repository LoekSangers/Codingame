import sys
import math

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

board = [[0,0,0],[0,0,0],[0,0,0]]

def check_win(b):
    for i in range(3):
        if b[i][0] > 0 and b[i][0] == b[i][1] and b[i][0] == b[i][2]:
            return b[i][0]

        if b[0][i] > 0 and b[0][i] == b[1][i] and b[0][i] == b[2][i]:
            return b[0][i]

    if b[0][0] > 0 and b[0][0] == b[1][1] and b[0][0] == b[2][2]:
        return b[2][0]
    if b[2][0] > 0 and b[2][0] == b[1][1] and b[2][0] == b[0][2]:
        return b[2][0]

def calc_valid_moves(b):
    valid = []
    for x in range(3):
        for y in range(3):
            if (b[x][y] == 0):
                valid.append((x,y))
    return valid

def move(b, x, y, pl, p = False):
    b[x][y] = pl
    if p:
        print(str(x) + " " + str(y))

moves = [(0,0), (2,0), (0,2), (2,2)]

# game loop
while True:
    opponent_row, opponent_col = [int(i) for i in input().split()]


    valid_action_count = int(input())
    valid_moves = []
    for i in range(valid_action_count):
        row, col = [int(j) for j in input().split()]
        valid_moves.append((row,col))

    
    if opponent_col != -1:
        move(board, opponent_row, opponent_col, 1)

    print(str(board), file=sys.stderr, flush=True)

    if len(valid_moves) == 9:
        move(board, 0,0, 2, True)
    else:
        for m in valid_moves:
            b = [row[:] for row in board]
            move(b, m[0], m[1], 2)
            if check_win(b) == 2:
                move(board, m[0], m[1], 2, True)
                continue

        for m in valid_moves:
            b = [row[:] for row in board]
            move(b, m[0], m[1], 1)
            if check_win(b) == 1:
                move(board, m[0], m[1], 2, True)
                continue

    # Write an action using print
    # To debug: print("Debug messages...", file=sys.stderr, flush=True)
        for m in moves:
            if m in valid_moves:
                move(board, m[0], m[1], 2, True)
                continue
