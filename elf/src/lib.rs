// Types - http://www.staroceans.org/e-book/elf-64-hp.pdf
// The Elf64_ prefix has  been dropped
type Address = u64;
type Offset = u64;
type Half = u16;
type Word = u32;
type SignedWord = i32;
type XWord = u64;
type SignedXWord = i64;

#[derive(Debug, Default)]
pub struct File {
    pub file_header : FileHeader,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct FileHeader {
    identification: Identification,
    object_file_type: Half,
    machine_type: Half,
    object_file_version: Word,
    entrypoint : Address,
    program_header_offset : Offset,
    section_header_offset : Offset,
    processor_specific_flags : Word,
    file_header_size : Half,
    program_headers_total_size : Half,
    program_header_entry_count : Half,
    section_headers_total_size : Half,
    section_header_entry_count: Half,
    sh_section_name_stringtab_entry_index : Half
}


#[derive(Debug, Default)]
#[repr(C)]
pub struct Identification {
    magic: [u8; 4],
    format_class: u8,
    endianness: u8,
    format_version: u8,
    abi_version : u8,
    pad: [u8; 8],
}

// 'static_assert' the header size - we only support 64-bit
//  for which the ELF header is 64 bytes (42 for 32-bit)
const ASSERT_HDR_SIZE : [u8; 64] = [0; std::mem::size_of::<FileHeader>()];

pub struct ProgramHeader {}

pub struct SectionHeader {}