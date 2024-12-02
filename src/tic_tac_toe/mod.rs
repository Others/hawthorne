use crate::cfr::game_model::{
    GamestateSampler, OracleGamestate, PlayerNumber, Probability, UtilityForAllPlayers, VisibleInfo,
};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TicTacToeBoard {
    squares: [TicTacToeSquare; 9],
    turn: Player,
}

static THREE_IN_ROW: LazyLock<Vec<[usize; 3]>> = LazyLock::new(|| {
    vec![
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        [0, 3, 6],
        [1, 4, 7],
        [2, 5, 8],
        [0, 4, 8],
        [2, 4, 6],
    ]
});

impl TicTacToeBoard {
    fn winner(&self) -> Option<Player> {
        for [a, b, c] in &*THREE_IN_ROW {
            let a_square = self.squares[*a];
            let b_square = self.squares[*b];
            let c_square = self.squares[*c];

            if a_square == b_square && b_square == c_square {
                return match a_square {
                    TicTacToeSquare::X => Some(Player::X),
                    TicTacToeSquare::O => Some(Player::O),
                    TicTacToeSquare::Empty => continue,
                };
            }
        }

        None
    }

    fn moves(&self) -> Vec<TicTacToeMove> {
        if self.winner().is_some() {
            return Vec::new();
        }

        let square = match self.turn {
            Player::X => TicTacToeSquare::X,
            Player::O => TicTacToeSquare::O,
        };

        let moves: Vec<TicTacToeMove> = self
            .squares
            .iter()
            .enumerate()
            .flat_map(|(idx, v)| {
                if *v == TicTacToeSquare::Empty {
                    Some(TicTacToeMove {
                        square: idx,
                        state: square,
                    })
                } else {
                    None
                }
            })
            .collect();

        moves
    }
}

impl Display for TicTacToeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("-----\n")?;
        f.write_str(&format!(
            "{} {} {}\n",
            self.squares[0], self.squares[1], self.squares[2]
        ))?;
        f.write_str(&format!(
            "{} {} {}\n",
            self.squares[3], self.squares[4], self.squares[5]
        ))?;
        f.write_str(&format!(
            "{} {} {}\n",
            self.squares[6], self.squares[7], self.squares[8]
        ))?;
        f.write_str("-----")?;
        Ok(())
    }
}

impl OracleGamestate<TicTacToeBoard> for TicTacToeBoard {
    fn info_for_turn_player(&self) -> TicTacToeBoard {
        self.clone()
    }

    fn players_playing(&self) -> PlayerNumber {
        2
    }

    fn turn(&self) -> PlayerNumber {
        match self.turn {
            Player::X => 0,
            Player::O => 1,
        }
    }

    fn advance(&self, m: &TicTacToeMove) -> Self {
        assert_eq!(self.squares[m.square], TicTacToeSquare::Empty);

        let mut new_squares = self.squares;
        new_squares[m.square] = m.state;
        Self {
            turn: match self.turn {
                Player::X => Player::O,
                Player::O => Player::X,
            },
            squares: new_squares,
        }
    }
}

impl VisibleInfo for TicTacToeBoard {
    type Move = TicTacToeMove;
    type Gamestate = TicTacToeBoard;

    fn players_playing(&self) -> PlayerNumber {
        2
    }

    fn turn(&self) -> PlayerNumber {
        OracleGamestate::turn(self)
    }

    fn run_for_moves(&self, mut f: impl FnMut(Self::Move)) -> Option<UtilityForAllPlayers> {
        match self.winner() {
            Some(Player::X) => {
                return Some(UtilityForAllPlayers {
                    util: [1.0, 0.0, 0.0, 0.0],
                })
            }
            Some(Player::O) => {
                return Some(UtilityForAllPlayers {
                    util: [0.0, 1.0, 0.0, 0.0],
                })
            }
            None => {}
        }

        let square = match self.turn {
            Player::X => TicTacToeSquare::X,
            Player::O => TicTacToeSquare::O,
        };

        let moves: Vec<TicTacToeMove> = self
            .squares
            .iter()
            .enumerate()
            .flat_map(|(idx, v)| {
                if *v == TicTacToeSquare::Empty {
                    Some(TicTacToeMove {
                        square: idx,
                        state: square,
                    })
                } else {
                    None
                }
            })
            .collect();

        if moves.is_empty() {
            // Stalemate
            return Some(UtilityForAllPlayers {
                util: [0.5, 0.5, 0.0, 0.0],
            });
        }

        moves.iter().for_each(|x| f(*x));

        None
    }

    // fn gamestate_sampler(&self) -> impl GamestateSampler<Info = Self> {
    //     TicTacToeSampler {
    //         board: self.clone(),
    //     }
    // }
}

impl Default for TicTacToeBoard {
    fn default() -> Self {
        Self {
            squares: [TicTacToeSquare::default(); 9],
            turn: Player::X,
        }
    }
}

#[derive(Debug, Clone)]
struct TicTacToeSampler {
    board: TicTacToeBoard,
}

impl GamestateSampler for TicTacToeSampler {
    type Info = TicTacToeBoard;

    fn sample(&mut self) -> (<Self::Info as VisibleInfo>::Gamestate, Probability) {
        (self.board.clone(), 1.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum TicTacToeSquare {
    X,
    O,
    #[default]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    X,
    O,
}

impl Display for TicTacToeSquare {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TicTacToeSquare::X => "X",
            TicTacToeSquare::O => "O",
            TicTacToeSquare::Empty => " ",
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TicTacToeMove {
    state: TicTacToeSquare,
    square: usize,
}

#[cfg(test)]
mod test {
    use crate::cfr::game_model::OracleGamestate;
    use crate::cfr::strategy_generation::strategy_generator::StrategyGenerator;
    use crate::tic_tac_toe::{TicTacToeBoard, TicTacToeSampler};
    use bumpalo_herd::Herd;

    #[test]
    fn play_a_game() {
        let herd = Herd::new();
        let strategy_generator = StrategyGenerator::new(&herd);
        strategy_generator.refine_strategy(
            TicTacToeSampler {
                board: TicTacToeBoard::default(),
            },
            1000,
        );

        // for (k, v) in &strategy.probability {
        //     println!("{}", k);
        //     println!("{:?}", v);
        // }

        let mut board = TicTacToeBoard::default();
        println!("{}", board);
        println!(
            "{:?}",
            strategy_generator
                .strategy_for_info(board.clone())
                .move_probabilities(),
        );

        while let Some(m) = strategy_generator
            .strategy_for_info(board.clone())
            .pick_move()
        {
            board = board.advance(&m);
            println!("{}", board);
            println!(
                "{:?}",
                strategy_generator
                    .strategy_for_info(board.clone())
                    .move_probabilities()
            );
        }
    }
}
