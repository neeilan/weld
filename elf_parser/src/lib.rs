use std::ffi::CStr;
use std::vec::Vec;

pub fn parse(bytes: Vec<u8>) -> elf::File {
    println!("{}-byte ELF file", bytes.len());

    let mut file = elf::File::default();

    let file_hdr_size = std::mem::size_of::<elf::FileHeader>();
    let hdr_bytes = &bytes[..file_hdr_size];
    file.file_header = unsafe { std::ptr::read(hdr_bytes.as_ptr() as *const _) };

    // Parse the section header table
    let mut section_headers = Vec::new();
    for i in 0..file.file_header.section_header_entry_count {
        let shdr_base: usize = (file.file_header.section_header_offset as usize)
            + (i as usize) * std::mem::size_of::<elf::SectionHeader>();
        let shdr_end: usize = shdr_base + std::mem::size_of::<elf::SectionHeader>();
        let shdr_bytes = &bytes[shdr_base..shdr_end];

        let shdr: elf::SectionHeader = unsafe { std::ptr::read(shdr_bytes.as_ptr() as *const _) };
        section_headers.push(shdr);
    }

    let section_name_string_table_offset = section_headers
        [file.file_header.sh_section_name_stringtab_entry_index as usize]
        .offset as usize;

    for i in 0..file.file_header.section_header_entry_count {
        let shdr = &section_headers[i as usize];
        unsafe {
            let section_name_start_offset = section_name_string_table_offset + (shdr.name as usize);
            let str_ref = &bytes[section_name_start_offset..];

            println!(
                "    > Read section header [{:?}] : {:?}",
                shdr.section_type,
                CStr::from_ptr(str_ref.as_ptr() as *const _)
                    .to_str()
                    .unwrap()
            );
        }
    }

    return file;
}
