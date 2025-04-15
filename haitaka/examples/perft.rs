use std::env::args;
use std::time::Instant;

// Copied from `cozy-chess` with only trivial modifications.
// Note that bulk counting on leave nodes significantly speeds up the run.

use haitaka::*;

fn perft<const DROPS: bool>(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut nodes = 0;
        board.generate_board_moves(|moves| {
            for mv in moves {
                let mut board = board.clone();
                board.play_unchecked(mv);
                nodes += perft::<DROPS>(&board, depth - 1);
            }
            false
        });
        if DROPS {
            board.generate_drops(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    nodes += perft::<DROPS>(&board, depth - 1);
                }
                false
            });
        }
        nodes
    }
}

fn perft_bulk<const DROPS: bool>(board: &Board, depth: u8) -> u64 {
    let mut nodes = 0;
    match depth {
        0 => nodes += 1,
        1 => {
            board.generate_board_moves(|moves| {
                nodes += moves.into_iter().len() as u64;
                false
            });
            if DROPS {
                board.generate_drops(|moves| {
                    nodes += moves.into_iter().len() as u64;
                    false
                });
            }
        }
        _ => {
            board.generate_board_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    let child_nodes = perft_bulk::<DROPS>(&board, depth - 1);
                    nodes += child_nodes;
                }
                false
            });
            if DROPS {
                board.generate_drops(|moves| {
                    for mv in moves {
                        let mut board = board.clone();
                        board.play_unchecked(mv);
                        let child_nodes = perft_bulk::<DROPS>(&board, depth - 1);
                        nodes += child_nodes;
                    }
                    false
                });
            }
        }
    }
    nodes
}

fn format_with_underscores(num: u64) -> String {
    let num_str = num.to_string();
    let mut formatted = String::new();
    let mut count = 0;

    for c in num_str.chars().rev() {
        if count == 3 {
            formatted.push('_');
            count = 0;
        }
        formatted.push(c);
        count += 1;
    }

    formatted.chars().rev().collect()
}

fn help_message() {
    eprintln!("USAGE: perft <depth> [<SFEN>] [--no-bulk] [--help]");
    eprintln!("  Defaults to the start position if no SFEN is specified.");
    eprintln!("  OPTIONS:");
    eprintln!("    --no-drops: Do not count drops.");
    eprintln!("    --no-bulk:  Disable bulk counting on leaf node parents.");
    eprintln!("    --help:     Print this message.");
}

fn main() {
    let mut depth = None;
    let mut board = None;
    let mut bulk = true;
    let mut drops = true;
    let mut help = false;
    for arg in args().skip(1) {
        if arg == "--no-bulk" {
            bulk = false;
            continue;
        }
        if arg == "--no-drops" {
            drops = false;
            continue;
        }
        if arg == "--help" {
            help = true;
            continue;
        }
        if depth.is_none() {
            if let Ok(arg) = arg.parse() {
                depth = Some(arg);
                continue;
            }
            eprintln!("ERROR: Invalid depth '{}'.", arg);
            help_message();
            return;
        }
        if board.is_none() {
            if let Ok(arg) = Board::from_sfen(&arg) {
                board = Some(arg);
                continue;
            }
            eprintln!("ERROR: Invalid SFEN '{}'.", arg);
            help_message();
            return;
        }
        eprintln!("ERROR: Unexpected argument '{}'.", arg);
        help_message();
        return;
    }

    if help {
        help_message();
        return;
    }

    let depth = if let Some(depth) = depth {
        depth
    } else {
        eprintln!("ERROR: Missing required argument 'depth'.");
        help_message();
        return;
    };
    let board = if board.is_some() {
        board.unwrap()
    } else {
        Board::startpos()
    };

    let start = Instant::now();
    let nodes = if bulk {
        if drops {
            perft_bulk::<true>(&board, depth)
        } else {
            perft_bulk::<false>(&board, depth)
        }
    } else if drops {
        perft::<true>(&board, depth)
    } else {
        perft::<false>(&board, depth)
    };
    let elapsed = start.elapsed();
    let nps = nodes as f64 / elapsed.as_secs_f64();
    println!(
        "perft {}: {} nodes in {:.2?} ({} nps)",
        depth,
        format_with_underscores(nodes),
        elapsed,
        format_with_underscores(nps as u64)
    );
}
