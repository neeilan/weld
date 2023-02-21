use super::{RelocationWithAddend, Symbol};
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction};
use std::fmt;

// The goal of weld_core is to take one or more Relocatable and generate an Executable

#[derive(Default)]
pub struct Section {
    pub name: String,
    pub bytes: Vec<u8>,
    pub offset: u64,
    pub virtual_address: u64,
}

impl fmt::Debug for Section {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.name != ".text" {
            // return fmt.write_fmt(format_args!(
            //     " -- <Section [{}] size={} file_offset={} virtual_address={:X}> -- ",
            //     self.name,
            //     self.bytes.len(),
            //     self.offset,
            //     self.virtual_address
            // ))
            return fmt.write_fmt(format_args!(""));
        }

        fmt.write_fmt(format_args!(
            "\n<Section [{}] size={} file_offset={} virtual_address={:X}>",
            self.name,
            self.bytes.len(),
            self.offset,
            self.virtual_address
        ))
        .unwrap();
        if self.name == ".text" {
            fmt.write_str("[Disassembly: \n").unwrap();
            let mut decoder = Decoder::with_ip(64, &self.bytes, self.offset, DecoderOptions::NONE);
            let mut formatter = GasFormatter::new();
            let mut output = String::new();
            let mut instruction = Instruction::default();
            while decoder.can_decode() {
                decoder.decode_out(&mut instruction);
                output.clear();
                formatter.format(&instruction, &mut output);
                fmt.write_fmt(format_args!("{:4X} {}\n", instruction.ip(), output))
                    .unwrap();
            }
            fmt.write_str("\n]").unwrap();
        }
        fmt.write_str("\n")
    }
}

#[derive(Debug, Default)]
pub struct Relocatable {
    pub path: String,
    pub sections: Vec<Section>,
    pub relocations: Vec<X64Relocation>,
    pub symbols : Vec<SymbolInfo>
}

impl Relocatable {
    pub fn find_section(&self, name: &str) -> Option<usize> {
        self.sections.iter().position(|s| s.name == name)
    }
}

#[derive(Default, Clone)]
pub struct X64Relocation {
    pub offset: usize, // Location where relocation should be applied
    pub info: u64,
    pub addend: i64,
    pub symbol_name: String,
}

impl X64Relocation {
    pub fn from(r: &RelocationWithAddend, symbol_name: &str) -> X64Relocation {
        X64Relocation {
            offset: r.offset as usize,
            info: r.info,
            addend: r.addend,
            symbol_name: symbol_name.to_string(),
        }
    }
    // Processor-specific: https://docs.oracle.com/cd/E19120-01/open.solaris/819-0690/chapter7-2/index.html
    pub fn relo_type(&self) -> X64ReloType {
        match self.info & 0xffffffff {
            4 => X64ReloType::R_AMD64_PLT32,
            _ => X64ReloType::R_UNKNOWN
        }
    }
}

impl fmt::Debug for X64Relocation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!(
            "X64Reloc < symbol=[{}] offset={:#x} addend={} type={:?} >",
            self.symbol_name,
            self.offset,
            self.addend,
            self.relo_type()
        ))
    }
}

#[derive(Default, Debug, Clone)]
pub struct SymbolInfo {
    pub name : String,
    pub symbol : Symbol
}

impl SymbolInfo {
    pub fn from(s: &Symbol, name : &str) -> SymbolInfo {
        SymbolInfo {
            name: name.to_string(),
            symbol: s.clone()
        }
    }

    pub fn is_defined(&self) -> bool {
        self.symbol.relative_to_section != 0
    }
}

#[derive(Default, Debug)]
#[repr(u64)]
pub enum X64ReloType {
    #[default]
    R_AMD64_NONE = 0,
    R_AMD64_PLT32 = 4,
    R_UNKNOWN = 0xffffffff
}

#[derive(Debug, Default)]
pub struct Executable {
    pub entry_point: u64,
    sections: Vec<Section>,
    pub bytes: Vec<u8>,
}
