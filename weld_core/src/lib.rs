/*  We perform the linking in passes:
    - Discard uninteresting sections (.comment, .note.gnu.property, .note.GNU-stack)
    - Merge sections into segments (determine layout of executable)
    - Walk through and assign an address to all defined symbols (and remember them
    - Walk through and replace the address of undefined symbols with the address of defined symbols
    - Issue an error if any undefined symbols remain      
    
    We want an abstract internediate representation of a set of ELF Files
    (Have a overrideable entrypoint symbol, which, if present, outputs an executable)
    (Do we want to support partial linking?)
    
    {


        sections {
            .name =
        }

        segments {
            .name = 
        }

        entrypoint {
            section_index
            offset_within_section
            symbol_index
        }

        symbols {
            .name =
        }DSG

        relocations {

        }
        


    }
*/

use std::{collections::HashMap, io::Write};
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction};

extern crate elf;

#[derive(Debug, Default)]
pub struct WeldError {}

// Function that converts to byte array. (found on stackoverflow)
unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

pub fn link(inputs : &Vec<elf::high_level_repr::Relocatable>) -> Result<elf::high_level_repr::Executable, Vec<WeldError>> {
    let mut res = elf::high_level_repr::Executable::default();


    let mut start_of_section = HashMap::<String, usize>::new();
    let mut symbols = HashMap::<String, usize>::new();

    for f in inputs {
        let text_idx = f.find_section(".text").expect("Cannot find .text");
        let section_start_in_exec = res.bytes.len();
        start_of_section.insert(f.path.clone(), section_start_in_exec);

        res.bytes.extend_from_slice(f.sections[text_idx].bytes.as_slice());
        
        for s in f.symbols.clone() {
            if s.is_defined() {
                // Assume defined relative to .text for now. st_shndx identifies which section we're *actually* relative to.
                symbols.insert(s.name, section_start_in_exec + s.symbol.value as usize);
            }
        }


    }

    println!("\nSymbols defined: {:?}", symbols );
    println!("\nsection starts: {:?}", start_of_section );


    for f in inputs {
        for r in f.relocations.clone() {
            if matches!(r.relo_type(), elf::high_level_repr::X64ReloType::R_AMD64_PLT32) {
                let symbol_addr = symbols.get(&r.symbol_name).expect("Couldn't find symbol").clone();
                let base_addr = start_of_section.get(&f.path).expect("Unknown start of section in exectuable");
                println!("Relocating symbol {:?}, defined_at:{:?} insert_at.base:{:?} insert_at.offset:{:?}", r.symbol_name, symbol_addr, base_addr, r.offset);
                let new_addr = ((symbol_addr as isize) - ((r.offset + base_addr) as isize) + (r.addend as isize)) as i32;
                res.bytes[base_addr + r.offset] = (new_addr & 0xff) as u8;                  
                res.bytes[base_addr + r.offset+1] = (new_addr >> 8  & 0xff) as u8;                     
                res.bytes[base_addr + r.offset+2] = (new_addr >> 16 & 0xff) as u8;                 
                res.bytes[base_addr + r.offset+3] = (new_addr >> 24 & 0xff) as u8;                 
            } else {
                println!("Unknown relo_type {:#x}", r.relo_type() as usize);
            }
        }
    }

    res.entry_point = symbols.get("_start").expect("Entrypoint symbol _start not found").clone() as u64;
    res.shstrtab = vec![0, 0x2e, 0x74, 0x65, 0x78, 0x74, 0, 0x2e, 0x73, 0x68, 0x73, 0x74, 0x72, 0x74, 0x61, 0x62];;


    let mut decoder = Decoder::with_ip(64, &res.bytes, 0, DecoderOptions::NONE);
    let mut formatter = GasFormatter::new();
    let mut output = String::new();
    let mut instruction = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        output.clear();
        formatter.format(&instruction, &mut output);
        println!("{:4X}  {}", instruction.ip(), output)
    }

    // Align the text section with the page size
    // >  Loadable process segments must have congruent values for p_vaddr and p_offset, modulo the page size.
    // WTF does this mean? https://refspecs.linuxfoundation.org/ELF/zSeries/lzsabi0_s390/c2090.html
    let page_size : usize = 4096;
    res.alignnent_padding = page_size - 64 - 56 - 56;

    // Build the rest of the file
    let hdr = build_header(&res).unwrap();
    let pht = build_pht(&res);

    // Write to a file
    let mut file = std::fs::OpenOptions::new()
        .create(true) // To create a new file
        .write(true)
    // either use the ? operator or unwrap since it returns a Result
        .open("./weld.out").unwrap();

    unsafe {   file.write_all(as_u8_slice(&hdr)); }
    for phdr in pht {
        unsafe { file.write_all(as_u8_slice(&phdr)); }
    }    
    let pad: Vec<u8> = vec![0; res.alignnent_padding];
    file.write_all(&pad);

    file.write_all(&res.bytes);

    // Write a null byte for the section header names strtab
    file.write_all(&res.shstrtab);

    let shdrs = build_sht(&res);

    for shdr in shdrs {
        unsafe { file.write_all(as_u8_slice(&shdr)); }
    }


    Ok(res)
}

pub fn build_header(executable : &elf::high_level_repr::Executable) -> Result<elf::FileHeader , Vec<WeldError>> {
    let mut hdr = elf::FileHeader::default();
    hdr.identification.magic = [0x7f, 0x45, 0x4c, 0x46];
    hdr.identification.format_class = 2; // 64-bit
    hdr.identification.endianness = 1; // little-endian
    hdr.identification.format_version = 1; // original ELF
    hdr.identification.os_abi = 0; // System V

    hdr.object_file_type = 0x02; // ET_EXEC
    hdr.machine_type = 0x3e; // AMD x86-64
    hdr.object_file_version = 1; // original ELF
    hdr.entrypoint= executable.entry_point + 0x401000; // e_entry - memory address where process starts executing
    hdr.program_header_offset = 64; // Immediately following ELF header
    hdr.section_header_offset = (64 + 2*56 + executable.alignnent_padding + executable.bytes.len() + executable.shstrtab.len()) as u64;
    hdr.processor_specific_flags = 0x00000102;
    hdr.file_header_size = std::mem::size_of::<elf::FileHeader>() as u16;

    hdr.program_headers_total_size = std::mem::size_of::<elf::ProgramHeader>() as u16;
    hdr.program_header_entry_count = 2;

    hdr.section_headers_total_size = std::mem::size_of::<elf::SectionHeader>() as u16;
    hdr.section_header_entry_count = 3;
    hdr.sh_section_name_stringtab_entry_index = 2;

    Ok(hdr)
}

pub fn build_pht(executable : &elf::high_level_repr::Executable) -> Vec<elf::ProgramHeader> {
    let mut phdr0 = elf::ProgramHeader::default();
    phdr0.segment_type = elf::SegmentType::Loadable;
    phdr0.offset = 0; // Read the goddamn elf header and PHT
    phdr0.virtual_address = 0; // Let's try this
    phdr0.physical_address = 0;
    phdr0.size_in_file = 64 + 56 * 2;
    phdr0.size_in_memory =  64 + 56 * 2;
    phdr0.required_alignment = 0; // Let's try this 4K
    phdr0.flags =0x1 | 0x2 | 0x4;

    let mut phdr = elf::ProgramHeader::default();
    phdr.segment_type = elf::SegmentType::Loadable;
    phdr.offset = (64 + 2 * 56 + executable.alignnent_padding) as u64;
    phdr.virtual_address = 0x401000; // Let's try this
    phdr.physical_address = 0x401000;
    phdr.size_in_file = executable.bytes.len() as u64;
    phdr.size_in_memory = executable.bytes.len() as u64;
    phdr.required_alignment = 0x1000; // Let's try this 4K
    phdr.flags = 0x1 | 0x2 | 0x4;

    vec![phdr0, phdr]

}

pub fn build_sht(executable : &elf::high_level_repr::Executable) -> Vec<elf::SectionHeader> {
    let sh_text = elf::SectionHeader {
        name: 1,
        section_type : elf::SectionType::ProgramData,
        flags :  0x1 | 0x2 | 0x4,
        virtual_address: 0x401000,
        offset : (64 + 2 * 56 + executable.alignnent_padding) as u64,
        size : executable.bytes.len() as u64,
        link_to_other_section : 0,
        misc_info : 0,
        address_allignment_boundary : 1,
        entries_total_size : 0
    };

    let sh_shstrtab = elf::SectionHeader {
        name: 7,
        section_type : elf::SectionType::StringTable,
        flags :  0x1 | 0x2 | 0x4,
        virtual_address: 0,
        offset : sh_text.offset + sh_text.size,
        size : executable.shstrtab.len() as u64,
        link_to_other_section : 0,
        misc_info : 0,
        address_allignment_boundary : 1,
        entries_total_size : 0
    };

    let mut sh0 = elf::SectionHeader::default();
    sh0.flags = 0x1 | 0x2 | 0x4;
    vec![sh0, sh_text, sh_shstrtab]
}