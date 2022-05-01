use std::fs;

pub mod bitboards;
pub mod board_files;
pub mod board_ranks;
pub mod castling;
pub mod board_builders;
pub mod chess_boards;
#[macro_use]
pub mod chess_moves;
pub mod colors;
pub mod errors;
pub mod games;
pub mod move_masks;
pub mod pieces;
pub mod squares;

pub fn hello_world() {
    println!("Hello, world from the example!");
}

pub fn read_print_moves(path: String) {
    let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
    println!("{}", contents);

    use regex::Regex;
    let re = Regex::new(
        r"(?x)
        (\d+)\.  # move number
        \s+
        (\S+)  # white's move
        \s+
        (\S+)  # black's move
        \s+
    ",
    )
    .unwrap();

    println!();
    for cap in re.captures_iter(&contents) {
        println!("Move: {} White: {} Black: {};", &cap[1], &cap[2], &cap[3]);
    }
}
