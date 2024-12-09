use std::{fs::File, io, path::Path};

use memmap2::Mmap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NxError {
    #[error("failed to load nx file")]
    Io(#[from] io::Error),

    #[error("the header's magic bytes are invalid")]
    InvalidMagicBytes,

    #[error("{0}..{1} is out of bounds")]
    OutOfBoundsAccess(usize, usize),

    #[error("invalid cast")]
    InvalidCast(#[from] core::array::TryFromSliceError),
}

trait NxTryGet {
    fn try_get_u16(&self, index: u64) -> Result<u16, NxError>;

    fn try_get_u32(&self, index: u64) -> Result<u32, NxError>;

    fn try_get_u64(&self, index: u64) -> Result<u64, NxError>;
}

impl NxTryGet for Mmap {
    fn try_get_u16(&self, index: u64) -> Result<u16, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u16>();

        let bytes =
            self.get(usize_index..usize_index + offset)
                .ok_or(NxError::OutOfBoundsAccess(
                    usize_index,
                    usize_index + offset,
                ))?;

        Ok(u16::from_le_bytes(bytes.try_into()?))
    }

    fn try_get_u32(&self, index: u64) -> Result<u32, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u32>();

        let bytes =
            self.get(usize_index..usize_index + offset)
                .ok_or(NxError::OutOfBoundsAccess(
                    usize_index,
                    usize_index + offset,
                ))?;

        Ok(u32::from_le_bytes(bytes.try_into()?))
    }

    fn try_get_u64(&self, index: u64) -> Result<u64, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u64>();

        let bytes =
            self.get(usize_index..usize_index + offset)
                .ok_or(NxError::OutOfBoundsAccess(
                    usize_index,
                    usize_index + offset,
                ))?;

        Ok(u64::from_le_bytes(bytes.try_into()?))
    }
}

pub struct NxFile {
    data: Mmap,
    header: NxHeader,
    string_table: u64,
    audio_table: u64,
    bitmap_table: u64,
}

impl NxFile {
    pub fn open(path: &Path) -> Result<Self, NxError> {
        let file = File::open(path)?;

        // Safety: TODO
        let data = unsafe { Mmap::map(&file)? };

        let header = NxHeader::new(&data)?;
        println!("{:?}", header);

        // TODO try_get_node
        let node_table = data.get(header.node_offset as usize..).unwrap();

        // TODO: *_table_offset?
        let string_table = data.try_get_u64(header.string_offset)?;
        let audio_table = data.try_get_u64(header.audio_offset)?;
        let bitmap_table = data.try_get_u64(header.bitmap_offset)?;

        Ok(Self {
            data,
            header,
            string_table,
            audio_table,
            bitmap_table,
        })
    }
}

#[derive(Debug)]
pub struct NxHeader {
    node_count: u32,
    node_offset: u64,
    string_count: u32,
    string_offset: u64,
    bitmap_count: u32,
    bitmap_offset: u64,
    audio_count: u32,
    audio_offset: u64,
}

impl NxHeader {
    pub fn new(data: &Mmap) -> Result<Self, NxError> {
        // Validate the header's "magic" bytes.
        let magic_bytes = data.try_get_u32(0)?;

        if magic_bytes != 0x34474B50 {
            return Err(NxError::InvalidMagicBytes);
        }

        Ok(Self {
            node_count: data.try_get_u32(4)?,
            node_offset: data.try_get_u64(8)?,
            string_count: data.try_get_u32(16)?,
            string_offset: data.try_get_u64(20)?,
            bitmap_count: data.try_get_u32(28)?,
            bitmap_offset: data.try_get_u64(32)?,
            audio_count: data.try_get_u32(40)?,
            audio_offset: data.try_get_u64(44)?,
        })
    }
}

pub struct NxNode {
    name: u32,
    children: u32,
    count: u16,
    data_type: u16,
    data: u64,
}
