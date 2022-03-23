use std::fs;

pub mod board_file;
pub mod board_rank;
pub mod square;
pub mod errors;


pub fn hello_world() {
    println!("Hello, world from the example!");
}


pub fn read_print_moves(path: String) {
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    println!("{}", contents);

    use regex::Regex;
    let re = Regex::new(r"(?x)
        (\d+)\.  # move number
        \s+
        (\S+)  # white's move
        \s+
        (\S+)  # black's move
        \s+
    ").unwrap();

    println!();
    for cap in re.captures_iter(&contents) {
        println!("Move: {} White: {} Black: {};", &cap[1], &cap[2], &cap[3]);
    }
}
