use std::cmp::PartialEq;
use std::cmp::Eq;

use std::cmp::PartialOrd;
use std::cmp::Ord;

use std::cmp::Ordering;

use std::process;

#[derive(Clone)]
pub struct SudokuBoard {
    board: [u8; 81]
}
impl SudokuBoard {
    pub fn new(board_str: String) -> Self {
        let mut lines = board_str.lines();

        let mut board: [u8; 81] = [0; 81];

        for row in 0..9 {
            let mut chars = match lines.next() {
                Some(val) => val.trim().chars(),
                None => {
                    eprintln!("Error: input file does not contain enough input");
                    process::exit(1);
                }
            };

            for col in 0..9 {
                let cur_char = match chars.next() {
                    Some(val) => val,
                    None => {
                        eprintln!("Error: input file does not contain enough input");
                        process::exit(1);
                    }
                };

                board[row*9 + col] = match cur_char.to_digit(10) {
                    Some(val) => val as u8,
                    None => {
                        eprintln!("Error: input file has invalid data at character {}", row*10+col);
                        process::exit(1);
                    }
                };
            }
        }

        SudokuBoard {board}
    }
    pub fn as_string(&self) -> String {
        let mut out_str = String::new();
        
        for row in 0..9 {
            for col in 0..9 {
                let cur_val = self.board[row*9 + col];
                match cur_val {
                    0 => out_str.push(' '),
                    _ => out_str.push(char::from_digit(cur_val as u32, 10).unwrap())
                }

                match col {
                    2 => out_str.push('|'),
                    5 => out_str.push('|'),
                    _ => ()
                }
            }

            out_str.push('\n');

            match row {
                2 => out_str.push_str("-----------\n"),
                5 => out_str.push_str("-----------\n"),
                _ => ()
            }
        }

        out_str
    }
    pub fn get_tile_value(&self, row: usize, col: usize) -> u8 {
        self.board[row*9 + col]
    }
    pub fn set_tile_value(&mut self, row: usize, col: usize, value: u8) {
        self.board[row*9 + col] = value;
    }
    
    pub const fn get_solvables() -> [[Position; 9]; 27] {
        // I learned about the existence of constant functions. This completely solves my dilemma about hardcoding values in.
        let mut solvables: [[Position; 9]; 27] = [[Position {row:0, col:0}; 9]; 27];
        let mut i;

        // The first 9 arrays will have a row's set of values. This means only the column changes
        i = 0;
        while i < 9 {
            let mut col = 0;
            while col < 9 {
                solvables[i][col].row = i;
                solvables[i][col].col = col;
                col += 1;
            }
            i += 1;
        }

        // The next 9 arrays will have a col's set of values. This means only the row changes
        i = 0;
        while i < 9 {
            let mut row = 0;
            while row < 9 {
                solvables[i+9][row].row = row;
                solvables[i+9][row].col = i;
                row += 1;
            }
            i += 1;
        }

        // The final 9 arrays are a bit weirder. 
        i = 0;
        while i < 9 {
            let mut tile = 0;
            while tile < 9 {
                solvables[i+18][tile].row = (i / 3) * 3 + tile / 3;
                solvables[i+18][tile].col = (i % 3) * 3 + tile % 3;
                tile += 1;
            }
            i += 1;
        }

        solvables
    }
    pub fn get_solvables_for(row: usize, col: usize) -> [[Position; 9]; 3] {
        let all_solvables = SudokuBoard::get_solvables();
        let mut solvables = [[Position {row:0, col:0}; 9]; 3];

        // 0-8 are the solvables for each row
        solvables[0] = all_solvables[row];

        // 9-17 are the solvables for each col
        solvables[1] = all_solvables[col+9];

        // 18-26 are the solvables for each nonet
        solvables[2] = all_solvables[(col/3 + (row/3)*3)+18];

        solvables
    }
    
}

struct Branch {
    pub tile: TileSuperpos,
    cur_option: usize,
    pub board_state: SudokuBoard
}
impl Branch {
    pub fn new(tile: &TileSuperpos, board_state: &SudokuBoard) -> Self {
        Branch {tile: tile.clone(), cur_option: 0, board_state: board_state.clone()}
    }
}
impl Iterator for Branch {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_option >= self.tile.options.len() {
            return None;
        }

        let next_val = self.tile.options[self.cur_option];
        self.cur_option += 1;
        Some(next_val)
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize
}

// Turns out I can't use copy because of the vector. I can implement clone, though.
#[derive(Clone)]
struct TileSuperpos {
    pub row: usize,
    pub col: usize,
    pub options: Vec<u8>
}
impl PartialEq for TileSuperpos {
    fn eq(&self, other: &Self) -> bool {
        self.options.len() == other.options.len()
    }
}
impl Eq for TileSuperpos {}
impl PartialOrd for TileSuperpos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.options.len().cmp(&other.options.len()))
    }
}
impl Ord for TileSuperpos {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap() // It's literally impossible to get 'None' with how I wrote partial_cmp, so this is fine.
    }
}

pub fn solve(board: &mut SudokuBoard) -> Result<&'static str, &'static str> {
    // Loop starts
    let mut branches: Vec<Branch> = Vec::new();

    loop {
        if is_solved(&board) {
            break;
        }

        // Find the tile with the fewest possible values
        let mut tile_options: Vec<TileSuperpos> = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                if board.get_tile_value(row, col) == 0 {
                    tile_options.push(get_valid_states(&board, row, col));
                }
            }
        }

        tile_options.sort();

        let lowest = &tile_options[0];

        // If there's only one possible value, fill it in and 'continue;'
        if lowest.options.len() == 1 {
            board.set_tile_value(lowest.row, lowest.col, lowest.options[0]);
            continue;
        }

        // If there's more than one possible value, push a branch to the stack with the options available
        if lowest.options.len() > 1 {
            let mut new_branch = Branch::new(lowest, &board);

            // Fill in the tile with the first option in the list and 'continue;'
            board.set_tile_value(lowest.row, lowest.col, new_branch.next().unwrap()); // This should never panic. There's a major logical error if it does.

            branches.push(new_branch);
            continue;
        }

        // If there's no possible values, start backtracking
        if lowest.options.len() == 0 {
            // TODO: See if tracking and undoing changes to the board is faster than just cloning an old board state into the variable. Possible optimization, though likely small
            loop {
                let mut last_branch = match branches.pop() {
                    Some(val) => val,
                    None => { // No possible value, no branches made, then no solution exists
                        return Err("No solution found");
                    }
                };

                // Restore the board state from this branch
                *board = last_branch.board_state.clone();

                // Check if we've already done the last option of this branch
                let next_num = match last_branch.next() {
                    Some(val) => val,
                    None => continue
                };

                // If not, set the next value
                board.set_tile_value(last_branch.tile.row, last_branch.tile.col, next_num);
                branches.push(last_branch);
                break;
            }
        }
    }
    
    Ok("Solution found")
}

fn is_solved(board: &SudokuBoard) -> bool {
    let solvables = SudokuBoard::get_solvables();
    for solvable in solvables {
        let mut checklist = [false; 9];
        for pos in solvable {
            let cur_val = board.get_tile_value(pos.row, pos.col);
            if cur_val == 0 {
                return false;
            } else {
                checklist[(cur_val-1) as usize] = true;
            }
        }
        for val in checklist {
            if val == false {
                return false;
            }
        }
    }

    true
}

fn get_valid_states(board: &SudokuBoard, row: usize, col: usize) -> TileSuperpos {
    // Get all the solvables for this tile
    let solvables = SudokuBoard::get_solvables_for(row, col);
    let mut states = [true; 9];

    for solvable in solvables {
        for pos in solvable {
            let cur_val = board.get_tile_value(pos.row, pos.col);

            if cur_val != 0 {
                states[(cur_val-1) as usize] = false;
            }
        }
    }

    let mut valid_states: Vec<u8> = Vec::new(); 
    for (index, state) in states.into_iter().enumerate() { // Found out I could use 'enumerate()' from the Iterator trait
        if state {
            valid_states.push((index+1) as u8);
        }
    }
    
    TileSuperpos {row, col, options: valid_states}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonet1_check() {
        let solvables = SudokuBoard::get_solvables();

        let expected = [
            Position{row:0,col:0},Position{row:0,col:1},Position{row:0,col:2},
            Position{row:1,col:0},Position{row:1,col:1},Position{row:1,col:2},
            Position{row:2,col:0},Position{row:2,col:1},Position{row:2,col:2},
            ];

        for i in 0..9 {
            assert_eq!(solvables[18][i].row, expected[i].row);
            assert_eq!(solvables[18][i].col, expected[i].col);
        }
    }

    #[test]
    fn nonet9_check() {
        let solvables = SudokuBoard::get_solvables();

        let expected = [
            Position{row:6,col:6},Position{row:6,col:7},Position{row:6,col:8},
            Position{row:7,col:6},Position{row:7,col:7},Position{row:7,col:8},
            Position{row:8,col:6},Position{row:8,col:7},Position{row:8,col:8},
            ];

        for i in 0..9 {
            assert_eq!(solvables[26][i].row, expected[i].row);
            assert_eq!(solvables[26][i].col, expected[i].col);
        }
    }

    #[test]
    fn tile_4_1_check() {
        let all_solvables = SudokuBoard::get_solvables();
        let tile_solvables = SudokuBoard::get_solvables_for(4, 1);

        for i in 0..9 {
            assert_eq!(all_solvables[4][i].row, tile_solvables[0][i].row);
            assert_eq!(all_solvables[4][i].col, tile_solvables[0][i].col);
        }
        for i in 0..9 {
            assert_eq!(all_solvables[10][i].row, tile_solvables[1][i].row);
            assert_eq!(all_solvables[10][i].col, tile_solvables[1][i].col);
        }
        for i in 0..9 {
            assert_eq!(all_solvables[21][i].row, tile_solvables[2][i].row);
            assert_eq!(all_solvables[21][i].col, tile_solvables[2][i].col);
        }
    }

    #[test]
    fn tile_7_5_check() {
        let all_solvables = SudokuBoard::get_solvables();
        let tile_solvables = SudokuBoard::get_solvables_for(7, 5);

        for i in 0..9 {
            assert_eq!(all_solvables[7][i].row, tile_solvables[0][i].row);
            assert_eq!(all_solvables[7][i].col, tile_solvables[0][i].col);
        }
        for i in 0..9 {
            assert_eq!(all_solvables[14][i].row, tile_solvables[1][i].row);
            assert_eq!(all_solvables[14][i].col, tile_solvables[1][i].col);
        }
        for i in 0..9 {
            assert_eq!(all_solvables[25][i].row, tile_solvables[2][i].row);
            assert_eq!(all_solvables[25][i].col, tile_solvables[2][i].col);
        }
    }
}