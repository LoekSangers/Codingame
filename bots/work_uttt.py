"""My implementation for Codingame Ultimate TicTacToe."""
from __future__ import annotations
from abc import ABC, abstractmethod
import datetime
from math import log
from random import choice, shuffle

class MonteCarloNode(ABC):
    """Base class for a Monte Carlo Tree Search state"""

    C = 1.4

    def __init__(self, parent:MonteCarloNode = None) -> None:
        self.visited_count = 0
        self._wins_score = 0
        self._loses_score = 0
        self.parent = parent
        self.unvisited_actions = []
        self.children = {}

    @property
    def score(self) -> int:
        """Return the score of this Node."""
        return self._wins_score - self._loses_score

    @property
    def utc(self) -> float:
        """Return the UTC value of this node.
        Based on: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search
        Section Exploration and exploitation.

        """
        return (self.score/self.visited_count) \
            + self.C * (log(self.parent.visited_count)/self.visited_count)**.5

    @property
    def best_child(self) -> MonteCarloNode:
        """Select the node with currently the highest utc value."""
        return self.children[max(self.children, key=lambda key: self.children[key].utc)]

    @property
    @abstractmethod
    def is_terminal(self):
        """Returns whether this node is terminal."""

    def select_leaf(self) -> MonteCarloNode:
        """Select a Leaf Node for the next iterations.
        Note: if fully expanded is recursive!

        """
        if not self.children:
            self.expand()
            shuffle(self.unvisited_actions)

        if self.unvisited_actions:
            return self.children[self.unvisited_actions.pop()]
        return self.best_child.select_leaf()

    @abstractmethod
    def expand(self):
        """Create all Child Nodes."""

    @abstractmethod
    def simulate(self) -> bool:
        """Complete one playout.
        Returns True if the starting player has won the playout.

        """

    def backpropagate(self, has_won:bool) -> None:
        """Use the result of the simulation to update information
        in the Nodes on the from the Child Node to the root Node.

        """
        self.visited_count += 1
        if has_won:
            self._wins_score += 1
        else:
            self._loses_score += 1
        if self.parent:
            self.parent.backpropagate(not has_won)

    def run(self, run_time: float) -> None:
        """Run simulations until we run out of run_time"""
        #iter = 0
        begin = datetime.datetime.now()
        while (datetime.datetime.now() - begin).total_seconds() < run_time:
            self.select_leaf().simulate()
        #    iter += 1
        #print(str(iter))

class Action:
    """Class for TicTacToe actions."""
    def __init__(self, x_coord: int, y_coord: int, player: int) -> None:
        self.x_coord = x_coord
        self.y_coord = y_coord
        self.player = player

    def __eq__(self, obj: Action):
        return self.player == obj.player \
            and self.x_coord == obj.x_coord \
            and self.y_coord == obj.y_coord

    def tuple(self):
        """Return action as tuple."""
        return (self.player, self.x_coord, self.y_coord)



class NormalBoard:
    """Class for normal TicTacToe board."""
    def __init__(self, normal_board: NormalBoard = None) -> None:
        if normal_board is None:
            self._grid = [0 for _ in range(9)]
            self._moves_count = 0
            self.local_winner = 0
        else:
            self._grid = normal_board._grid[:]
            self._moves_count = normal_board._moves_count
            self.local_winner = normal_board.local_winner


    def get_valid_actions(self, current_player:int, base_x:int = 0, base_y:int = 0) -> list[Action]:
        """List of valid moves on this board."""
        if self.local_winner != 0:
            return []

        valid_actions = []
        for y_coord in range(3):
            for x_coord in range(3):
                if self._grid[y_coord*3 + x_coord] == 0:
                    valid_actions.append(Action(base_x + x_coord, base_y + y_coord, current_player))
        return valid_actions

    def play(self, move: Action, *, localize:bool = False) -> int:
        """Play a given move on this board."""
        x_coord = move.x_coord if not localize else move.x_coord % 3
        y_coord = move.y_coord if not localize else move.y_coord % 3
        if x_coord < 0 or x_coord >= 3 \
                or y_coord < 0 or y_coord >= 3 \
                or self._grid[y_coord*3 + x_coord] != 0:
            raise InvalidMoveError(x_coord, y_coord, self._grid)

        self._grid[y_coord*3 + x_coord] = move.player
        self._moves_count += 1

        return self._update_winner()

    def _update_winner(self) -> int:
        """Return the id of the player that has won.
        Return -1 if the board has tied
        Return 0 otherwise
        """
        for index in range(3):
            #rows
            if self._grid[index*3 + 0] > 0 \
                    and self._grid[index*3 + 0] == self._grid[index*3 + 1] \
                    and self._grid[index*3 + 0] == self._grid[index*3 + 2]:
                self.local_winner = self._grid[index*3 + 0]
                return self.local_winner

            #cols
            if self._grid[0 + index] > 0 \
                    and self._grid[0 + index] == self._grid[3 + index] \
                    and self._grid[0 + index] == self._grid[6 + index]:
                self.local_winner = self._grid[0 + index]
                return self.local_winner

        #diagonals
        if self._grid[0] > 0 \
                and self._grid[0] == self._grid[4] \
                and self._grid[0] == self._grid[8]:
            self.local_winner = self._grid[0]
            return self.local_winner
        if self._grid[2] > 0 \
                and self._grid[2] == self._grid[4] \
                and self._grid[2] == self._grid[6]:
            self.local_winner = self._grid[2]
            return self.local_winner

        if self._moves_count > 8:
            self.local_winner = -1
            return self.local_winner
        return 0

class UltimateBoard(MonteCarloNode):
    """Class for Ultimate TicTacToe board."""
    def __init__(self, *, last_move: Action = None, parent: UltimateBoard = None) -> None:
        super().__init__(parent)
        if parent is None:
            self._grid = [NormalBoard() for _ in range(9)]
            self._done_board_count = 0
        else:
            self._grid = [NormalBoard(normal_board) for normal_board in parent._grid]
            self._done_board_count = parent._done_board_count

        self.last_move = last_move
        self.local_winner = 0

    def get_valid_actions(self) -> list[Action]:
        """List of valid moves on this board."""
        valid_actions = []
        if self.last_move is not None:
            x_grid = self.last_move.x_coord % 3
            y_grid = self.last_move.y_coord % 3
            board = self._grid[y_grid*3 + x_grid]
            if board.local_winner == 0:
                valid_actions = board.get_valid_actions(
                    1 if self.last_move.player != 1 else 2,
                    x_grid*3,
                    y_grid*3
                )

        if not valid_actions:
            position: int
            board: NormalBoard
            for position, board in enumerate(self._grid):
                if board.local_winner == 0:
                    valid_actions.extend(
                        board.get_valid_actions(
                            1 if self.last_move.player != 1 else 2,
                            (position%3)*3,
                            (position//3)*3
                        )
                    )

        return valid_actions

    def play(self, move: Action) -> None:
        """Play a given move on this board."""
        x_grid = move.x_coord // 3
        y_grid = move.y_coord // 3

        winner = self._grid[y_grid*3 + x_grid].play(move, localize = True)

        if winner != 0:
            #someone won last board, so check whether game done
            self._done_board_count += 1
            self._calc_winner()

        self.last_move = move

    def _calc_winner(self) -> int:
        """Return the id of the player that has won.
        Return -1 if the board has tied
        Return 0 otherwise
        """
        for index in range(3):
            #rows
            if self._grid[index*3 + 0].local_winner > 0 \
            and self._grid[index*3 + 0].local_winner == self._grid[index*3 + 1].local_winner \
            and self._grid[index*3 + 0].local_winner == self._grid[index*3 + 2].local_winner:
                self.local_winner = self._grid[index*3 + 0].local_winner
                return self.local_winner

            #cols
            if self._grid[0 + index].local_winner > 0 \
            and self._grid[0 + index].local_winner == self._grid[3 + index].local_winner \
            and self._grid[0 + index].local_winner == self._grid[6 + index].local_winner:
                self.local_winner = self._grid[0 + index].local_winner
                return self.local_winner

        #diagonals
        if self._grid[0].local_winner > 0 \
                and self._grid[0].local_winner == self._grid[4].local_winner \
                and self._grid[0].local_winner == self._grid[8].local_winner:
            self.local_winner = self._grid[0].local_winner
            return self.local_winner
        if self._grid[6 + 0].local_winner > 0 \
                and self._grid[6 + 0].local_winner == self._grid[4].local_winner \
                and self._grid[6+ 0].local_winner == self._grid[2].local_winner:
            self.local_winner = self._grid[6].local_winner
            return self.local_winner

        if self._done_board_count > 8:
            self.local_winner = -1
            return -1
        return 0

    @property
    def is_terminal(self):
        return self.local_winner != 0

    def expand(self):
        actions = self.get_valid_actions()

        for action in actions:
            child_node = UltimateBoard(parent=self)
            child_node.play(action)
            tup = action.tuple()
            self.unvisited_actions.append(tup)
            self.children[tup] = child_node

    def simulate(self):
        simulation_board = UltimateBoard(parent=self)
        simulation_board.last_move = self.last_move
        while not simulation_board.is_terminal:
            try:
                simulation_board.play(choice(simulation_board.get_valid_actions()))
            except:
                print(simulation_board.get_valid_actions())
                for nb in self._grid:
                    print(nb._grid)

        self.backpropagate(simulation_board.local_winner not in [-1, self.last_move.player])

    def print_move(self):
        """Print the best move to console.
        Note: They expect first the y coord then the x coord.
        """
        
        print(str(self.last_move.y_coord) + " " + str(self.last_move.x_coord))


# define Python user-defined exceptions
class InvalidMoveError(Exception):
    """Exception raised for Invalid moves."""

    def __init__(self, x_coord: int, y_coord: int, board: NormalBoard) -> None:
        self.message = f"Cannot move to {x_coord}, {y_coord} on board {board}"
        super().__init__(self.message)

def main():
    """Main"""
    #first turn
    opponent_row, opponent_col = [int(i) for i in input().split()]
    valid_action_count = int(input())
    for _ in range(valid_action_count):
        _, valid_action_count = [int(j) for j in input().split()]

    root = UltimateBoard()

    if opponent_row == -1:# I start in the middle for no apparent reason
        root.play(Action(4, 4, 1))
        root.run(.90)
        print("4 4")
    else:
        act = Action(opponent_col, opponent_row, 2)
        root.play(act)
        root.run(.90)
        root = root.best_child
        root.parent = None
        root.print_move()


    #game loop
    while True:
        opponent_row, opponent_col = [int(i) for i in input().split()]
        valid_action_count = int(input())
        for _ in range(valid_action_count):
            _, _ = [int(j) for j in input().split()]

        if not root.children:
            root.expand()

        root = root.children[(2, opponent_col, opponent_row)]
        root.run(.06)
        root = root.best_child
        root.parent = None
        root.print_move()

main()