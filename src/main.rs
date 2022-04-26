use sudoku_solver::*;

use std::fs;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: invalid arguments");
        eprintln!("Usage: sudoku_solver [filename]");
        eprintln!("The input file should contain 9 characters '0-9', 0 being an empty tile, followed by any single character. Repeat this 9 times for each row.");
        eprintln!("On Windows, this means the file can not end with CRLF. If you don't know the difference between LF and CRLF, Google it.");
        process::exit(1);
    }

    // Read a puzzle in from a file
    let board_str: String = match fs::read_to_string(&args[1]) {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error: file {} not found or inaccessible", &args[1]);
            process::exit(1);
        }
    };

    // Build a structure with the contents of the file
    let mut board = sudoku_solver::SudokuBoard::new(board_str);

    // Run the solving algorithm and print an appropriate response
    match solve(&mut board) {
        Ok(_) => {
            println!("Solved:");
            println!("{}", board.as_string());
        },
        Err(_) => {
            println!("No solution found.");
        }
    }
}
