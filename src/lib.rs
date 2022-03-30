use std::fs;

pub mod bitboards;
pub mod board_files;
pub mod board_ranks;
pub mod castling;
pub mod chess_board_builder;
pub mod chess_boards;
pub mod colors;
pub mod errors;
pub mod peace_moves;
pub mod pieces;
pub mod square;

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
