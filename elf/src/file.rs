// This module defines data structures that reflect the on-disk
// ELF representation.
//
// The structures and explanations mostly came directly from the
// ELF man page [1] and/or Oracle's Linker and Libraries Guide [2]
//
// These definitions are not intended to exhaustively model the
// ELF spec across all platforms. Rather, they are added on an
// as-needed basis to link progressivel-complex test programs on
// my 64-bit x86-64 Linux machine.
//
// [1] https://man7.org/linux/man-pages/man5/elf.5.html
// [2] https://docs.oracle.com/cd/E19683-01/816-1386/index.html

//  Commmon
// =========

use std::ops::BitOr;

pub type Address = u64;
type FileOffset = u64;
type Half = u16;
type Word = u32;
type XWord = u64;
type SignedXWord = i64;

#[derive(Debug, Default)]
#[repr(C)]
pub struct FileHeader {
    pub identification: Identification,
    pub object_file_type: Half,
    pub machine_type: Half,
    pub object_file_version: Word,
    pub entrypoint: Address,
    pub program_header_offset: FileOffset,
    pub section_header_offset: FileOffset,
    pub processor_specific_flags: Word,
    pub file_header_size: Half,
    pub program_headers_total_size: Half,
    pub program_header_entry_count: Half,
    pub section_headers_total_size: Half,
    pub section_header_entry_count: Half,
    pub sh_section_name_stringtab_entry_index: Half,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Identification {
    pub magic: [u8; 4],
    pub format_class: u8,
    pub endianness: u8,
    pub format_version: u8,
    pub os_abi: u8,
    pad: [u8; 8],
}

// 'static_assert' the header size - we only support 64-bit
//  for which the ELF header is 64 bytes (42 for 32-bit)
pub const FILE_HEADER_SIZE: usize = std::mem::size_of::<FileHeader>();
const _ASSERT_FILE_HDR_SIZE: [u8; 64] = [0; FILE_HEADER_SIZE];

//  Linkable/Section-based view
// ==============================

#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct SectionHeader {
    pub name: Word,
    pub section_type: SectionType,
    pub flags: u64,
    pub virtual_address: Address,
    pub offset: FileOffset,
    pub size: XWord,
    pub link_to_other_section: Word,
    pub misc_info: Word,
    pub address_allignment_boundary: XWord,
    pub entry_size: XWord,
}

#[derive(Debug)]
#[repr(u64)]
pub enum SectionFlags {
    None = 0x0,
    Write = 0x1,
    Alloc = 0x2,
    Executable = 0x4,
}

impl BitOr for SectionFlags {
    type Output = u64;
    fn bitor(self, rhs: SectionFlags) -> u64 {
        (self as u64) | (rhs as u64)
    }
}

pub const SECTION_HEADER_SIZE: usize = std::mem::size_of::<SectionHeader>();
const _ASSERT_SECTION_HDR_SIZE: [u8; 64] = [0; SECTION_HEADER_SIZE];

#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SectionType {
    #[default]
    None = 0x0,
    ProgramData = 0x1,
    SymbolTable = 0x2,
    StringTable = 0x3,
    RelocationWithAddend = 0x4,
    SymbolHashTable = 0x5,
    DynamicLinkingInfo = 0x6,
    Notes = 0x7,
    ProgramSpaceWithNoData = 0x8,
    RelocationWithoutAddend = 0x9,
    Reserved = 0x0A,
    DynamicLinkerSymbolTable = 0x0B,
    ArrayOfConstructors = 0x0E,
    ArrayOfDestructors = 0x0F,
    ArrayOfPreConstructors = 0x10,
    SectionGroup = 0x11,
    ExtendedSectionIndices = 0x12,
    NumberOfDefinedTypes = 0x13,
}

// Relocations

#[derive(Debug, Default)]
#[repr(C)]
pub struct RelocationWithAddend {
    pub offset: Address, // Location where relocation should be applied
    pub info: XWord,
    pub addend: SignedXWord,
}

impl RelocationWithAddend {
    pub fn symbol(&self) -> usize {
        (self.info >> 32) as usize
    }
    // Processor-specific: https://docs.oracle.com/cd/E19120-01/open.solaris/819-0690/chapter7-2/index.html
    pub fn relo_type(&self) -> XWord {
        self.info & 0xffffffff
    }
}

// Symbols

// https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-79797.html#chapter6-35166
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Symbol {
    pub name: Word,
    pub info: u8,
    pub other: u8,
    pub relative_to_section: Half, // If SHN_UNDEF, means undefined
    pub value: Address,            // can be an absolute value or an address
    pub size: XWord,
}

//  Executable/Segment-based view
// ===============================

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct ProgramHeader {
    pub segment_type: SegmentType,
    pub flags: Word,
    pub offset: FileOffset,
    pub virtual_address: Address,
    pub physical_address: Address,
    pub size_in_file: XWord,
    pub size_in_memory: XWord,
    pub required_alignment: Address,
}

pub const PROGRAM_HEADER_SIZE: usize = std::mem::size_of::<ProgramHeader>();
const _ASSERT_PROGRAM_HDR_SIZE: [u8; 56] = [0; PROGRAM_HEADER_SIZE];

#[derive(Debug, Default, Clone)]
#[repr(u32)]
pub enum SegmentType {
    #[default]
    None = 0x0,
    Loadable = 0x1,
    DynamicLinkInfo = 0x2,
    InterpreterInfo = 0x3,
    AuxiliaryInfo = 0x4,
    Reserved = 0x5,
    ProgramHeaderTableSegment = 0x6,
    ThreadLocalStorageTemplate = 0x7,
    GnuEHFrame = 1685382480,
    GnuStack = 1685382481,
    GnuRelRO = 1685382482,
    GnuProperty = 1685382483,
}


#[derive(Debug)]
#[repr(u32)]
pub enum SegmentFlags {
    None = 0x0,
    Execute = 0x1,
    Write = 0x2,
    Read = 0x4,
}

impl BitOr for SegmentFlags {
    type Output = u32;
    fn bitor(self, rhs: SegmentFlags) -> u32 {
        (self as u32) | (rhs as u32)
    }
}