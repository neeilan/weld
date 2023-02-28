use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: weld <files>");
        return;
    }

    let mut relocatables = Vec::new();
    for i in 1..args.len() {
        let path = args.get(i).unwrap();
        match fs::read(path) {
            Ok(bytes) => {
                println!("\n=============================================================");
                let reloc = elf_parser::parse(path, &bytes);
                println!("{reloc:?}");
                relocatables.push(reloc);
            }
            Err(err) => {
                println!("{path} : {err}");
                return;
            }
        }
    }

    println!("\n======================== WELD ===========================");
    match weld_core::link(&relocatables) {
        Ok(exec) => {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open("./weld.out")
                .unwrap();
            file.write_all(&exec.encode())
                .expect("Write to file failed");
        }
        Err(errs) => {
            println!("{errs:?}")
        }
    }
}
