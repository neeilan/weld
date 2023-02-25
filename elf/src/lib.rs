pub mod high_level_repr;

// There are 3 modules in this crate
// elf::repr - models on-disk representation
// elf::logl - logical views of elf representations to perform operations on

// Types - http://www.staroceans.org/e-book/elf-64-hp.pdf
// The Elf64_ prefix has  been dropped
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
const _ASSERT_ELF_HDR_SIZE: [u8; 64] = [0; std::mem::size_of::<FileHeader>()];


#[derive(Debug, Default, PartialEq)]
#[repr(C)]
pub struct SectionHeader {
    // Offset, in bytes, to the section name, relative to the start of the
    // section name string table [elf-64-hp.pdf]
    pub name: Word,
    pub section_type: SectionType,
    pub flags: XWord,
    pub virtual_address: Address,
    pub offset: FileOffset,
    pub size: XWord,
    // Semantics of value depend on `section_type`
    pub link_to_other_section: Word,
    pub misc_info: Word,
    // Must be a power of 2
    pub address_allignment_boundary: XWord,
    // Only applicable for fixed-size entries
    pub entries_total_size: XWord,
}

#[derive(Debug, Default, PartialEq)]
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

#[derive(Debug, Default)]
#[repr(C)]
pub struct RelocationWithAddend {
   pub offset : Address, // Location where relocation should be applied
   pub info : XWord,
   pub addend : SignedXWord
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

// https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-79797.html#chapter6-35166
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Symbol {
    pub name : Word,
    pub info : u8,
    pub other : u8,
    pub relative_to_section : Half, // If SHN_UNDEF, means undefined
    pub value : Address,            // can be an absolute value or an address
    pub size : XWord,
}

const _ASSERT_SECTION_HDR_SIZE: [u8; 64] = [0; std::mem::size_of::<SectionHeader>()];



#[derive(Debug, Default)]
#[repr(C)]
pub struct ProgramHeader {
    pub segment_type: SegmentType,
    pub flags: Word,
    pub offset: FileOffset,
    pub virtual_address: Address,
    pub physical_address: Address,
    pub size_in_file: XWord,
    pub size_in_memory: XWord,
    pub required_alignment : Address
}


#[derive(Debug, Default)]
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
