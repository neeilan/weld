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
        }

        relocations {

        }
        


    }
*/

use std::collections::HashMap;
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction};


extern crate elf;

#[derive(Debug, Default)]
pub struct WeldError {}

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

    Ok(res)
}

pub fn build_header(executable : &elf::high_level_repr::Executable) -> Result<elf::FileHeader , Vec<WeldError>> {
    Ok(elf::FileHeader::default())
}