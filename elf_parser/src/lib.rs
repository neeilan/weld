use std::vec::Vec;

pub fn parse(path: &str, bytes: &Vec<u8>) -> elf::logical::Relocatable {
    let header = parse_header(&bytes);
    let section_headers = parse_section_headers(&bytes, &header);
    let section_names = parse_section_name_string_table(&bytes, &section_headers, &header);
    let symbols = parse_symbol_table(&bytes, &section_headers, &header);
    let relocations = parse_relocations(&bytes, &section_headers, &symbols);

    let mut result = elf::logical::Relocatable::default();
    result.path = path.to_string();
    result.symbols.extend_from_slice(&symbols);
    result.relocations.extend(relocations);

    for shdr in section_headers {
        let mut section = elf::logical::Section::default();

        section.name = section_names.at(shdr.name as usize).unwrap().clone();
        section.bytes =
            bytes[(shdr.offset as usize)..((shdr.offset as usize) + (shdr.size as usize))].to_vec();
        section.offset = shdr.offset;
        section.virtual_address = shdr.virtual_address;
        result.sections.push(section);
    }

    result
}

fn parse_header(bytes: &Vec<u8>) -> elf::file::FileHeader {
    let hdr_bytes = &bytes[..elf::file::FILE_HEADER_SIZE];
    let header: elf::file::FileHeader = unsafe { std::ptr::read(hdr_bytes.as_ptr() as *const _) };
    return header;
}

fn parse_section_headers(
    bytes: &Vec<u8>,
    file_header: &elf::file::FileHeader,
) -> Vec<elf::file::SectionHeader> {
    let mut section_headers = Vec::new();

    for i in 0..file_header.section_header_entry_count {
        let base: usize = (file_header.section_header_offset as usize)
            + (i as usize) * elf::file::SECTION_HEADER_SIZE;
        let shdr: elf::file::SectionHeader =
            unsafe { std::ptr::read(bytes[base..].as_ptr() as *const _) };
        section_headers.push(shdr);
    }
    section_headers
}

fn parse_symbol_string_table(
    bytes: &Vec<u8>,
    section_headers: &Vec<elf::file::SectionHeader>,
    file_header: &elf::file::FileHeader,
) -> elf::string_table::StrTab {
    let shstrtab_header =
        &section_headers[file_header.sh_section_name_stringtab_entry_index as usize];

    let header = section_headers
        .iter()
        .find(|&hdr| {
            matches!(hdr.section_type, elf::file::SectionType::StringTable)
                && hdr != shstrtab_header
        })
        .unwrap();

    parse_string_table(&bytes, &header)
}

fn parse_string_table(bytes: &Vec<u8>, header: &elf::file::SectionHeader) -> elf::string_table::StrTab {
    elf::string_table::StrTab::new(
        &bytes[(header.offset as usize)..((header.offset + header.size) as usize)],
    )
}

fn parse_section_name_string_table(
    bytes: &Vec<u8>,
    section_headers: &Vec<elf::file::SectionHeader>,
    file_header: &elf::file::FileHeader,
) -> elf::string_table::StrTab {
    let section_header =
        &section_headers[file_header.sh_section_name_stringtab_entry_index as usize];
    parse_string_table(&bytes, &section_header)
}

fn parse_symbol_table(
    bytes: &Vec<u8>,
    section_headers: &Vec<elf::file::SectionHeader>,
    file_header: &elf::file::FileHeader,
) -> Vec<elf::logical::SymbolInfo> {
    let header = section_headers
        .iter()
        .find(|&hdr| matches!(hdr.section_type, elf::file::SectionType::SymbolTable))
        .unwrap();

    let num_symbols = (header.size as usize) / std::mem::size_of::<elf::file::Symbol>();
    let symbol_names = parse_symbol_string_table(&bytes, &section_headers, &file_header);
    let mut symbols = Vec::new();

    for i in 0..num_symbols {
        let sym_bytes =
            &bytes[(header.offset as usize) + i * std::mem::size_of::<elf::file::Symbol>()..];
        let symbol: elf::file::Symbol = unsafe { std::ptr::read(sym_bytes.as_ptr() as *const _) };
        let symbol_info = elf::logical::SymbolInfo {
            name: symbol_names.at(symbol.name as usize).unwrap().clone(),
            symbol: symbol,
        };
        symbols.push(symbol_info.clone());
    }
    symbols
}

fn parse_relocations(
    bytes: &Vec<u8>,
    section_headers: &Vec<elf::file::SectionHeader>,
    symbol_table: &Vec<elf::logical::SymbolInfo>,
) -> Vec<elf::logical::Relocation> {
    let mut relocations = Vec::new();

    let header_or = section_headers.iter().find(|&hdr| {
        matches!(
            hdr.section_type,
            elf::file::SectionType::RelocationWithAddend
        )
    });

    if header_or.is_none() {
        return relocations;
    }

    let header = header_or.unwrap();
    let num_relocs = header.size as usize / std::mem::size_of::<elf::file::RelocationWithAddend>();
    for i in 0..num_relocs {
        let base =
            (header.offset as usize) + i * std::mem::size_of::<elf::file::RelocationWithAddend>();
        let r: elf::file::RelocationWithAddend =
            unsafe { std::ptr::read(bytes[base..].as_ptr() as *const _) };
        relocations.push(elf::logical::Relocation::from(
            &r,
            &symbol_table[r.symbol()],
        ));
    }

    relocations
}
