import sys

nrOfBoost = 1
nrOfRound = 0
prev_x = 0
prev_y = 0

while True:
    x, y, next_checkpoint_x, next_checkpoint_y, next_checkpoint_dist, next_checkpoint_angle = [int(i) for i in input().split()]
    opponent_x, opponent_y = [int(i) for i in input().split()]

    if nrOfRound == 0:
        prev_x = x
        prev_y = y

    momentum_x = x - prev_x
    momentum_y = y - prev_y
    print(momentum_x, file=sys.stderr, flush=True)
    print(momentum_y, file=sys.stderr, flush=True)


    thrust = 100
    boost = False

    if next_checkpoint_dist > 5000 and nrOfBoost > 0:
        nrOfBoost-=1
        boost = True

    if (next_checkpoint_angle > 90 or next_checkpoint_angle < -90) and abs(momentum_x) + abs(momentum_y) > 300:
        thrust = 5
        if boost:
            boost = False
            nrOfBoost+=1



    target_x = next_checkpoint_x - momentum_x
    target_y = next_checkpoint_y - momentum_y
    target_move = "BOOST" if boost else str(thrust)


    print(str(target_x) + " " + str(target_y) + " " + str(target_move))

    prev_x = x
    prev_y = y
    nrOfRound+=1
