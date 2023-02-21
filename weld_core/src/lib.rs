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

extern crate elf;

#[derive(Debug, Default)]
pub struct WeldError {}

struct Location {}

pub fn link(inputs : &Vec<elf::high_level_repr::Relocatable>) -> Result<elf::high_level_repr::Executable, Vec<WeldError>> {
    let mut res = elf::high_level_repr::Executable::default();

    let symbols = HashMap::<String, Location>::new();

    for rel in inputs {
        let text_idx = rel.find_section(".text").expect("Cannot find .text");
        res.bytes.extend_from_slice(rel.sections[text_idx].bytes.as_slice())
    }
    Ok(res)
}

pub fn build_header(executable : &elf::high_level_repr::Executable) -> Result<elf::FileHeader , Vec<WeldError>> {
    Ok(elf::FileHeader::default())
}