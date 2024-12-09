use std::{fs::File, io, path::Path};

use memmap2::Mmap;
use thiserror::Error;

const MAGIC_BYTES: [u8; 4] = [0x50, 0x4b, 0x47, 0x34];

#[derive(Error, Debug)]
pub enum NxError {
    #[error("failed to load nx file")]
    Io(#[from] io::Error),

    #[error("the file header is invalid")]
    InvalidHeader,

    #[error("the header's magic bytes are invalid")]
    InvalidMagicBytes,
}

pub struct NxFile {
    mmap: Mmap,
    header: NxFileHeader,
}

impl NxFile {
    pub fn open(path: &Path) -> Result<Self, NxError> {
        let file = File::open(path)?;

        // Safety: TODO
        let mmap = unsafe { Mmap::map(&file)? };

        let header = NxFileHeader::new(&mmap)?;
        println!("{:?}", header);

        let node_table = mmap.get(header.node_offset as usize..).unwrap();
        // let string_table = mmap.get(header.string_offset as usize..8).unwrap();

        let name = u32::from_le_bytes(node_table.get(0..4).unwrap().try_into().unwrap());
        let children = u32::from_le_bytes(node_table.get(4..8).unwrap().try_into().unwrap());
        let count = u16::from_le_bytes(node_table.get(8..10).unwrap().try_into().unwrap());
        let data_type = u16::from_le_bytes(node_table.get(10..12).unwrap().try_into().unwrap());
        let data = u64::from_le_bytes(node_table.get(12..20).unwrap().try_into().unwrap());

        println!("data type: {}", data_type);

        Ok(Self { mmap, header })
    }

    pub fn get(&self, path: &str) {}
}

#[derive(Debug)]
pub struct NxFileHeader {
    node_count: u32,
    node_offset: u64,
    string_count: u32,
    string_offset: u64,
    bitmap_count: u32,
    bitmap_offset: u64,
    audio_count: u32,
    audio_offset: u64,
}

#[derive(Debug)]
#[repr(packed)]
pub struct Header {
    pub magic: u32,
    pub nodecount: u32,
    pub nodeoffset: u64,
    pub stringcount: u32,
    pub stringoffset: u64,
    pub bitmapcount: u32,
    pub bitmapoffset: u64,
    pub audiocount: u32,
    pub audiooffset: u64,
}

impl NxFileHeader {
    pub fn new(data: &[u8]) -> Result<Self, NxError> {
        // Validate the header's "magic" bytes.
        let magic_bytes: &[u8] = data.get(0..4).ok_or(NxError::InvalidHeader)?;

        if magic_bytes != MAGIC_BYTES {
            return Err(NxError::InvalidMagicBytes);
        }

        let mut index = 4;

        Ok(Self {
            node_count: Self::get_u32(data, &mut index)?,
            node_offset: Self::get_u64(data, &mut index)?,
            string_count: Self::get_u32(data, &mut index)?,
            string_offset: Self::get_u64(data, &mut index)?,
            bitmap_count: Self::get_u32(data, &mut index)?,
            bitmap_offset: Self::get_u64(data, &mut index)?,
            audio_count: Self::get_u32(data, &mut index)?,
            audio_offset: Self::get_u64(data, &mut index)?,
        })
    }

    fn get_u32(data: &[u8], index: &mut usize) -> Result<u32, NxError> {
        let bytes = data
            .get(*index..*index + size_of::<u32>())
            .ok_or(NxError::InvalidHeader)?;

        let num = u32::from_le_bytes(bytes.try_into().map_err(|_| NxError::InvalidHeader)?);

        *index += size_of::<u32>();
        Ok(num)
    }

    fn get_u64(data: &[u8], index: &mut usize) -> Result<u64, NxError> {
        let bytes = data
            .get(*index..*index + size_of::<u64>())
            .ok_or(NxError::InvalidHeader)?;

        let num = u64::from_le_bytes(bytes.try_into().map_err(|_| NxError::InvalidHeader)?);

        *index += size_of::<u64>();
        Ok(num)
    }
}

pub struct NxNode {
    name: u32,
    children: u32,
    count: u16,
    data_type: u16,
    data: u64,
}
