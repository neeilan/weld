use std::ffi::CStr;
use std::vec::Vec;

fn bitcast<T>(bytes : &Vec<u8>, start : usize) -> T {
    let t_bytes = &bytes[start..];
    return unsafe { std::ptr::read(t_bytes.as_ptr() as *const _)}
}

fn read_cstr(bytes : &Vec<u8>, table_offset : usize, str_offset : usize) -> &str {
    let str_ref = &bytes[table_offset+str_offset..];

    unsafe {
    return  CStr::from_ptr(str_ref.as_ptr() as *const _)
            .to_str()
            .unwrap();
    }
}

pub fn parse(bytes: Vec<u8>) -> elf::File {
    println!("{}-byte ELF file", bytes.len());

    let mut file = elf::File::default();

    let file_hdr_size = std::mem::size_of::<elf::FileHeader>();
    let hdr_bytes = &bytes[..file_hdr_size];
    file.file_header = unsafe { std::ptr::read(hdr_bytes.as_ptr() as *const _) };

    // Parse the section header table
    let mut section_headers = Vec::new();
    let mut relocation_indices: Vec<usize> = Vec::new();
    let mut symtab_index : usize = 0;
    let mut strtab_index : usize = 0;
    for i in 0..file.file_header.section_header_entry_count {
        let shdr_base: usize = (file.file_header.section_header_offset as usize)
            + (i as usize) * std::mem::size_of::<elf::SectionHeader>();
        let shdr_end: usize = shdr_base + std::mem::size_of::<elf::SectionHeader>();
        let shdr_bytes = &bytes[shdr_base..shdr_end];

        let shdr: elf::SectionHeader = unsafe { std::ptr::read(shdr_bytes.as_ptr() as *const _) };
        if matches!(shdr.section_type, elf::SectionType::RelocationWithAddend) {
            relocation_indices.push(i as usize)
        } else if matches!(shdr.section_type, elf::SectionType::SymbolTable) {
            symtab_index = i as usize;
        } else if matches!(shdr.section_type, elf::SectionType::StringTable) {
            if (strtab_index == 0) { // HACK: How do we check that this is the program symbol table (not section one)?
                strtab_index = i as usize;
            }
        }
        section_headers.push(shdr);

    }

    let section_name_string_table_offset = section_headers
        [file.file_header.sh_section_name_stringtab_entry_index as usize]
        .offset as usize;

    for i in 0..file.file_header.section_header_entry_count {
        let shdr = &section_headers[i as usize];
            println!(
                "    > Read section header [{:?}] : {:?}",
                shdr.section_type,
                read_cstr(&bytes, section_name_string_table_offset, shdr.name as usize)
            );
    }


    println!("Symbol table : {:?}", section_headers[symtab_index]);

    println!("Relocations");
    for i in relocation_indices {
        println!("{:?}", section_headers[i]);

        let reloc_base: usize = (section_headers[i].offset as usize);
        let reloc_end: usize = reloc_base + std::mem::size_of::<elf::RelocationWithAddend>();
        let reloc_bytes = &bytes[reloc_base..reloc_end];

        let reloc: elf::RelocationWithAddend = unsafe { std::ptr::read(reloc_bytes.as_ptr() as *const _) };

        let t_bytes = &bytes[(section_headers[symtab_index].offset as usize) + (std::mem::size_of::<elf::Symbol>() * reloc.symbol()) ..];
        let symbol : elf::Symbol = unsafe { std::ptr::read(t_bytes.as_ptr() as *const _) };

        println!("    > Read relocation {:?} [{:?}] - Symbol {:?}=[{:?}] symname=[{:?}]",
            reloc,
            read_cstr(&bytes, section_name_string_table_offset, section_headers[i].name as usize), reloc.symbol(), symbol,
            read_cstr(&bytes, section_headers[strtab_index].offset as usize, symbol.name as usize),
        )
    };
//     Relocation section '.rela.text' at offset 0x118 contains 1 entry:
//     Offset             Info             Type               Symbol's Value  Symbol's Name + Addend
// 0000000000000007  0000000300000004 R_X86_64_PLT32         0000000000000000 multiply - 4
    return file;
}
