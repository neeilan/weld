use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(path) => match fs::read(path) {
            Ok(bytes) => {
                elf_parser::parse(bytes);
            }
            Err(err) => println!("{}", err),
        },
        None => println!("Usage: weld <files>"),
    }
}
