use crate::string_table;

// This module defines high-level 'logical' representations of ELF
// structures. While the on-disk layout (defined in elf::file) is
/// effective, manipulating fixed-size structures is often restrictive.
//
// Therefore, these data structures, rather than the `file::` structs,
// are used in the public API for the weld library. For example, in
// the simplest terms, the weld library takes a collection of `Relocatable`s
// and returns a single `Executable`.
use super::file;
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction};
use std::fmt;

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
    pub relocations: Vec<Relocation>,
    pub symbols: Vec<SymbolInfo>,
}

impl Relocatable {
    pub fn find_section(&self, name: &str) -> Option<usize> {
        self.sections.iter().position(|s| s.name == name)
    }
}

#[derive(Default, Debug)]
#[repr(u64)]
pub enum RelocationType {
    #[default]
    None = 0,
    Abs64 = 1,
    Abs32 = 2,
    Plt32 = 4,
    Copy = 5,
    GlobalData = 6,
    JumpSlot = 7,
    RelativeToReloc = 8,
    ThreadPtrOffset = 10, // Used with TLS - see https://akkadia.org/drepper/tls.pdf
    Unknown = 0xffffffff,
}

#[derive(Default, Clone)]
pub struct Relocation {
    pub offset: usize, // Location where relocation should be applied
    pub info: u64,
    pub addend: i64,
    pub symbol: SymbolInfo,
}

impl Relocation {
    pub fn from(r: &file::RelocationWithAddend, symbol: &SymbolInfo) -> Relocation {
        Relocation {
            offset: r.offset as usize,
            info: r.info,
            addend: r.addend,
            symbol: symbol.clone(),
        }
    }
    // Processor-specific: https://docs.oracle.com/cd/E19120-01/open.solaris/819-0690/chapter7-2/index.html
    pub fn relo_type(&self) -> RelocationType {
        match self.raw_relo_type() {
            4 => RelocationType::Plt32,
            _ => RelocationType::Unknown,
        }
    }

    fn raw_relo_type(&self) -> u64 {
        self.info & 0xffffffff
    }
}

impl fmt::Debug for Relocation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!(
            "Relocation < symbol=[{:?} name={}] offset={:#x} addend={} raw_type={:#x} type={:?} >",
            self.symbol.symbol,
            self.symbol.name,
            self.offset,
            self.addend,
            self.raw_relo_type(),
            self.relo_type()
        ))
    }
}

#[derive(Default, Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub symbol: file::Symbol,
}

impl SymbolInfo {
    pub fn from(s: &file::Symbol, name: &str) -> SymbolInfo {
        SymbolInfo {
            name: name.to_string(),
            symbol: s.clone(),
        }
    }

    pub fn is_defined(&self) -> bool {
        self.symbol.relative_to_section != 0
    }
}

// An executable has a very specific layout
//   [ 64 bytes             ] File Header
//   [ 56*(# phrs) bytes    ] Program Header Table
//   [ `pre_text_pad` bytes ] Padding
//   [ sh.size bytes        ] Text Section
//   [ sh.size bytes        ] Section Header String Table
//   [ 64*(# shrs) bytes    ] Section Header Table

#[derive(Debug, Default)]
pub struct Executable {
    // Fields match final on-disk layout order
    pub file_header: file::FileHeader,
    pub program_headers: Vec<file::ProgramHeader>,
    pub pre_text_pad: usize,
    pub text_section: Vec<u8>,
    pub shstrtab: string_table::StrTab,
    pub section_headers: Vec<file::SectionHeader>, // Always last
}

impl Executable {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(as_u8_slice(&self.file_header));
        for phdr in self.program_headers.clone() {
            bytes.extend_from_slice(as_u8_slice(&phdr));
        }
        bytes.extend(vec![0; self.pre_text_pad]);
        bytes.extend_from_slice(&self.text_section);
        bytes.extend_from_slice(&self.shstrtab.bytes);
        for shdr in self.section_headers.clone() {
            bytes.extend_from_slice(as_u8_slice(&shdr));
        }
        bytes
    }
}

// Function that converts to byte array.
// Slightly modified from https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>()) }
}
