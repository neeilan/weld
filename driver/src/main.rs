use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut relocatables = Vec::new();
    for i in 1..args.len() {
        match args.get(i) {
            Some(path) => {
                match fs::read(path) {
                    Ok(bytes) => {
                        println!("\n=======================================================================");
                        let reloc = elf_parser::parse(path, bytes);
                        println!("{:?}", reloc);
                        relocatables.push(reloc);
                    }
                    Err(err) => println!("{}", err),
                }
            }
            None => println!("Usage: weld <files>"),
        };
    }

    let exec_or = weld_core::link(&relocatables);
    match exec_or {
        Ok(exec) => {
            let mut file = std::fs::OpenOptions::new()
                .create(true) // To create a new file
                .write(true)
                // either use the ? operator or unwrap since it returns a Result
                .open("./weld.out")
                .unwrap();
            file.write_all(&exec.encode())
                .expect("Write to file failed");
        }
        Err(errs) => {
            println!("{:?}", errs)
        }
    }
}
