
use std::vec::Vec;


pub fn parse(bytes : Vec<u8>) -> elf::File {
    println!("{}-byte ELF file", bytes.len());

    let mut file = elf::File::default();

    let file_hdr_size = std::mem::size_of::<elf::FileHeader>();
    let hdr_bytes = &bytes[..file_hdr_size];
    file.file_header = unsafe { std::ptr::read(hdr_bytes.as_ptr() as *const _) };
    println!("{:?}", file.file_header);

    return file;
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
