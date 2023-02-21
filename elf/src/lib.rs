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
type SignedWord = i32;
type XWord = u64;
type SignedXWord = i64;

#[derive(Debug, Default)]
pub struct File {
    pub file_header: FileHeader,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct FileHeader {
    identification: Identification,
    object_file_type: Half,
    machine_type: Half,
    object_file_version: Word,
    entrypoint: Address,
    pub program_header_offset: FileOffset,
    pub section_header_offset: FileOffset,
    processor_specific_flags: Word,
    file_header_size: Half,
    program_headers_total_size: Half,
    pub program_header_entry_count: Half,
    section_headers_total_size: Half,
    pub section_header_entry_count: Half,
    pub sh_section_name_stringtab_entry_index: Half,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Identification {
    magic: [u8; 4],
    format_class: u8,
    endianness: u8,
    format_version: u8,
    abi_version: u8,
    pad: [u8; 8],
}

// 'static_assert' the header size - we only support 64-bit
//  for which the ELF header is 64 bytes (42 for 32-bit)
const _ASSERT_ELF_HDR_SIZE: [u8; 64] = [0; std::mem::size_of::<FileHeader>()];


#[derive(Debug, Default)]
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
    misc_info: Word,
    // Must be a power of 2
    address_allignment_boundary: XWord,
    // Only applicable for fixed-size entries
    entries_total_size: XWord,
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
#[repr(C)]
pub struct Symbol {
    pub name : Word,
    info : u8,
    other : u8,
    relative_to_section : Half,
    value : Address,
    size : XWord,
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
    size_in_file: XWord,
    size_in_memory: XWord,
    required_alignment : Address
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
