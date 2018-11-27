extern crate chess;
extern crate rand;
extern crate rayon;
extern crate num_cpus;
extern crate stopwatch;
use stopwatch::{Stopwatch};

use chess::{
	ALL_FILES,
	ALL_RANKS,
	Board,
	BoardStatus,
	ChessMove,
	Color,
	MoveGen,
	Piece,
	Square,
};

use std::io;
use std::env;
use rand::*;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;

const N:i32 = 50000;

#[derive(Debug)]
enum GameResult {
	BlackWin,
	WhiteWin,
	Draw
}

#[derive(Debug)]
struct SimResult {
	board: Board,
	result: GameResult
}

fn sim(b:Board) -> SimResult {
	let mut board = b.clone();
	let mut count = 0;
	let mut moves = [ChessMove::default(); 256];
	let mut rng = rand::thread_rng();

	while let BoardStatus::Ongoing = board.status() {
		let moves_count = board.enumerate_moves(&mut moves);
		let step = Range::new(0, moves_count);
		let choice = step.ind_sample(&mut rng);

		let chosen_move = moves[choice as usize];

		board = board.make_move_new(chosen_move);

		count += 1;

		if count >= 50 {
			break;
		}
	}

	SimResult {
		board: board,
		result: match board.status() {
			BoardStatus::Checkmate => {
				match board.side_to_move() {
					Color::Black => {
						GameResult::WhiteWin
					},
					_ => {
						GameResult::BlackWin
					}
				}
			},
			_ => {
				GameResult::Draw
			}
		}
	}
}

fn main() {
	let cpus = num_cpus::get();

	let mut finished_count = 0;
	let mut total_count = 0;
	let fen = env::args().nth(1).unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

	let mut board = Board::from_fen(fen.to_string()).unwrap();
	let mut rng = rand::thread_rng();

	let mut iter2 = MoveGen::new_legal(&board);
	let mut count = 0;

	let opening_moves = MoveGen::new_legal(&board);

	let sw = Stopwatch::start_new();

	let mut best = std::f64::MIN;
	let mut best_move = ChessMove::default();

	opening_moves.for_each(|m| {
		let after_opener = board.make_move_new(m);

		let results:i32 = (0..N)
			.into_par_iter()
			.map(|_| {
				let simmed = sim(after_opener);

				match simmed.result {
					GameResult::BlackWin => -1,
					GameResult::WhiteWin => 1,
					GameResult::Draw => 0
				}
			})
			.sum();

		if (results as f64) > (best) {
			best = results as f64 / N as f64;
			best_move = m
		}
	});

	println!("{}: {}", best_move, best);
	println!("Evaluated {} games in {}ms", cpus as i32 * N, sw.elapsed_ms());
	println!("({}/ms)", (cpus as f64 * N as f64)/(sw.elapsed_ms() as f64));
	println!("Result\n{}", board.make_move_new(best_move).to_string());
}
