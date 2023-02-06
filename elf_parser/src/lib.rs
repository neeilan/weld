use elf;

use std::vec::Vec;


pub fn parse(bytes : Vec<u8>) -> elf::File {
    println!("{}-byte ELF file", bytes.len());

    return elf::File{}
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
