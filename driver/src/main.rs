use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    // Parse the file

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

    let executable = weld_core::link(&relocatables);
    match executable {
        Ok(exec) =>  {println!("{:?}", exec)}
        Err(errs) => {println!("{:?}", errs)}
    }

    // Link inputs into executable

    // Build header for the executable

    // Write header and executable to disk
}
