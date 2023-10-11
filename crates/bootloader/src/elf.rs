use elf::ElfBytes;
use elf::endian::AnyEndian;
use crate::error::Error;

pub fn parse_file(bytes: &[u8]) -> Result<(), Error> {
    let elf_file = ElfBytes::<AnyEndian>::minimal_parse(bytes)?;
    Ok(())
}