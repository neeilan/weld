use elf::logical::Executable;
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction};
use std::collections::HashMap;

extern crate elf;

#[derive(Debug, Default)]
pub struct WeldError;

pub fn link(
    inputs: &[elf::logical::Relocatable],
) -> Result<elf::logical::Executable, Vec<WeldError>> {
    let mut exec = elf::logical::Executable::default();

    let mut start_of_section = HashMap::<String, usize>::new();
    let mut symbols = HashMap::<String, usize>::new();

    for f in inputs {
        let text_section_offset = f.find_section(".text").expect("Cannot find .text");
        let section_start_in_exec = exec.text_section.len();
        start_of_section.insert(f.path.clone(), section_start_in_exec);

        exec.text_section
            .extend_from_slice(f.sections[text_section_offset].bytes.as_slice());

        for s in &f.symbols {
            if s.is_defined() {
                // Assume defined relative to .text for now. st_shndx identifies which section we're *actually* relative to.
                symbols.insert(s.name.clone(), section_start_in_exec + s.symbol.value as usize);
            }
        }
    }

    println!("\nSymbols defined: {symbols:?}");
    println!("\nsection starts: {start_of_section:?}");

    for f in inputs {
        for r in &f.relocations {
            if matches!(r.relo_type(), elf::logical::RelocationType::Plt32) {
                let symbol_addr = *symbols
                    .get(&r.symbol.name)
                    .expect("Couldn't find symbol");
                let base_addr = start_of_section
                    .get(&f.path)
                    .expect("Unknown start of section in exectuable");
                println!("Relocating symbol {:?}, defined_at:{:?} insert_at.base:{:?} insert_at.offset:{:?}", r.symbol.name, symbol_addr, base_addr, r.offset);
                let new_addr = ((symbol_addr as isize) - ((r.offset + base_addr) as isize)
                    + (r.addend as isize)) as i32;
                exec.text_section[base_addr + r.offset] = (new_addr & 0xff) as u8;
                exec.text_section[base_addr + r.offset + 1] = (new_addr >> 8 & 0xff) as u8;
                exec.text_section[base_addr + r.offset + 2] = (new_addr >> 16 & 0xff) as u8;
                exec.text_section[base_addr + r.offset + 3] = (new_addr >> 24 & 0xff) as u8;
            } else {
                println!(
                    "Unhandled relo_type {:#x}  in {} ; full relo: [{:?}]",
                    r.relo_type() as usize,
                    f.path,
                    r
                );
            }
        }
    }

    let entry_point = *symbols
        .get("_start")
        .expect("Entrypoint symbol _start not found") as u64;

    let mut decoder = Decoder::with_ip(64, &exec.text_section, 0, DecoderOptions::NONE);
    let mut formatter = GasFormatter::new();
    let mut output = String::new();
    let mut instruction = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        output.clear();
        formatter.format(&instruction, &mut output);
        println!("{:4X}  {}", instruction.ip(), output)
    }

    // We know exactly how many segments and sections we have.
    // This is useful for calculating padding, building headers
    // in a single pass, etc.
    // At the moment, we have these segments:
    //    1) NULL header to load PHT
    //    2) Program data
    // and these sections:
    //    1) NULL section
    //    2) .text
    //    3) .shstrtab
    // If a segment or section is unneeded, we can keep the header
    // and give it a 'null' value, keeping our layout logic simpler.
    const NUM_PROGRAM_HEADERS: u16 = 2;
    exec.pre_text_pad = get_pre_text_pad(NUM_PROGRAM_HEADERS as usize);

    const NUM_SECTION_HEADERS: u16 = 3;

    // Build the executable
    exec.file_header = build_header(&exec, NUM_PROGRAM_HEADERS, NUM_SECTION_HEADERS, entry_point);
    exec.program_headers = build_pht(&exec);
    exec.section_headers = build_sht(&mut exec);

    Ok(exec)
}

fn get_pre_text_pad(num_program_headers: usize) -> usize {
    // Align the text section with the page size
    // The man page says 'loadable process segments must have congruent values
    // for p_vaddr and p_offset, modulo the page size.' Not quite sure what this
    // means - but aligning right at a page boundary makes a lot of sense for mmap-ing.
    // I played with other alignments - some of which led to segfaults on execve ¯\_(ツ)_/¯
    let page_size: usize = 4096;
    let unpadded_text_offset =
        elf::file::FILE_HEADER_SIZE + num_program_headers * elf::file::PROGRAM_HEADER_SIZE;
    if unpadded_text_offset < page_size {
        page_size - unpadded_text_offset
    } else {
        unpadded_text_offset % page_size
    }
}

// Precondition: The following fields in `e` must be correctly populated:
//    - text_section
//    - pre_text_pad
pub fn build_header(
    e: &Executable,
    program_header_entry_count: u16,
    section_header_entry_count: u16,
    entrypoint_offset_from_text_start: u64,
) -> elf::file::FileHeader {
    let mut hdr = elf::file::FileHeader::default();
    hdr.identification.magic = [0x7f, 0x45, 0x4c, 0x46];
    hdr.identification.format_class = 2; // 64-bit
    hdr.identification.endianness = 1; // little-endian
    hdr.identification.format_version = 1; // original ELF
    hdr.identification.os_abi = 0; // System V
    hdr.object_file_type = 0x02; // ET_EXEC
    hdr.machine_type = 0x3e; // AMD x86-64
    hdr.object_file_version = 1; // original ELF
    hdr.processor_specific_flags = 0x00000102;
    hdr.file_header_size = elf::file::FILE_HEADER_SIZE as u16;
    hdr.program_header_offset = elf::file::FILE_HEADER_SIZE as u64;
    hdr.program_headers_total_size = elf::file::PROGRAM_HEADER_SIZE as u16;
    hdr.section_headers_total_size = elf::file::SECTION_HEADER_SIZE as u16;
    hdr.program_header_entry_count = program_header_entry_count;
    hdr.section_header_entry_count = section_header_entry_count;
    hdr.sh_section_name_stringtab_entry_index = section_header_entry_count - 1; // Always last
    hdr.entrypoint = entrypoint_offset_from_text_start + 0x401000;
    hdr.section_header_offset = (elf::file::FILE_HEADER_SIZE
        + (program_header_entry_count as usize) * elf::file::PROGRAM_HEADER_SIZE
        + e.pre_text_pad
        + e.text_section.len()
        + e.shstrtab.len()) as u64;
    hdr
}

// Precondition - executable's text_section and pre_text_pad must be populated
pub fn build_pht(e: &elf::logical::Executable) -> Vec<elf::file::ProgramHeader> {
    assert!(!e.text_section.is_empty());
    assert!(e.file_header.program_header_entry_count == 2);

    // The ELF header and program headers comprise a segment
    let mut phdr0 = elf::file::ProgramHeader::default();
    phdr0.segment_type = elf::file::SegmentType::Loadable;
    phdr0.offset = 0;
    phdr0.virtual_address = 0x400000; // Let's try this
    phdr0.physical_address = 0;
    phdr0.size_in_file = (elf::file::FILE_HEADER_SIZE + elf::file::PROGRAM_HEADER_SIZE * 2) as u64;
    phdr0.size_in_memory = phdr0.size_in_file;
    phdr0.required_alignment = 0x1000;
    phdr0.flags = elf::file::SegmentFlags::Read as u32;

    let mut phdr = elf::file::ProgramHeader::default();
    phdr.segment_type = elf::file::SegmentType::Loadable;
    phdr.offset = (elf::file::FILE_HEADER_SIZE
        + elf::file::PROGRAM_HEADER_SIZE * (e.file_header.program_header_entry_count as usize)
        + e.pre_text_pad) as u64;
    phdr.virtual_address = 0x401000;
    phdr.physical_address = phdr.virtual_address;
    phdr.size_in_file = e.text_section.len() as u64;
    phdr.size_in_memory = phdr.size_in_file;
    phdr.required_alignment = 0x1000;
    phdr.flags = elf::file::SegmentFlags::Read | elf::file::SegmentFlags::Execute;

    vec![phdr0, phdr]
}

pub fn build_sht(e: &mut elf::logical::Executable) -> Vec<elf::file::SectionHeader> {
    assert!(e.file_header.section_header_entry_count == 3);

    let sh_text = elf::file::SectionHeader {
        name: e.shstrtab.insert(".text") as u32,
        section_type: elf::file::SectionType::ProgramData,
        flags: elf::file::SectionFlags::Alloc | elf::file::SectionFlags::Executable,
        virtual_address: 0x400100,
        offset: (elf::file::FILE_HEADER_SIZE
            + (e.file_header.program_header_entry_count as usize) * elf::file::PROGRAM_HEADER_SIZE
            + e.pre_text_pad) as u64,
        size: e.text_section.len() as u64,
        link_to_other_section: 0,
        misc_info: 0,
        address_allignment_boundary: 1,
        entry_size: 0,
    };

    let sh_shstrtab = elf::file::SectionHeader {
        name: e.shstrtab.insert(".shstrtab") as u32,
        section_type: elf::file::SectionType::StringTable,
        flags: elf::file::SectionFlags::Alloc | elf::file::SectionFlags::Executable,
        virtual_address: 0,
        offset: sh_text.offset + sh_text.size,
        size: e.shstrtab.len() as u64,
        link_to_other_section: 0,
        misc_info: 0,
        address_allignment_boundary: 1,
        entry_size: 0,
    };

    let sh0 = elf::file::SectionHeader {
    flags : elf::file::SectionFlags::Alloc | elf::file::SectionFlags::Executable,
    ..Default::default()
    };
    vec![sh0, sh_text, sh_shstrtab]
}
