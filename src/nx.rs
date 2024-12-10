use core::str;
use std::{cmp::Ordering, fs::File, path::Path};

use memmap2::Mmap;
use thiserror::Error;

const NX_NODE_OFFSET: u64 = 20;

pub struct NxFile {
    data: Mmap,
    header: NxHeader,
    root: NxNode,
}

impl NxFile {
    pub fn open(path: &Path) -> Result<Self, NxError> {
        let file = File::open(path)?;

        // Safety: TODO
        let data = unsafe { Mmap::map(&file)? };

        let header = NxHeader::new(&data)?;
        println!("{:?}", header);

        let root = data.try_get_node(header.node_offset)?;
        println!("{:?}", root);

        Ok(Self { data, header, root })
    }

    // TODO we should allow x/y/z querying
    fn get_node(&self, root: &NxNode, name: &str) -> Option<NxNode> {
        let mut index = self.header.node_offset + root.children as u64 * NX_NODE_OFFSET;
        let mut count = root.count as u64;

        while count > 0 {
            let middle = count / 2;

            let current = match self.data.try_get_node(index + (middle * NX_NODE_OFFSET)) {
                Ok(node) => node,
                Err(_) => return None,
            };

            let current_name = match self.get_str(current.name as u64) {
                Ok(name) => name,
                Err(_) => return None,
            };

            println!("current: {:?}", current);
            println!("{}", current_name);

            match current_name.cmp(name) {
                Ordering::Less => {
                    index += middle * (NX_NODE_OFFSET * 2);
                    count -= middle + 1;
                }
                Ordering::Equal => {
                    return Some(current);
                }
                Ordering::Greater => count = middle,
            }
        }

        None
    }

    fn get_str(&self, index: u64) -> Result<&str, NxError> {
        let offset = self
            .data
            .try_get_u64(self.header.string_offset + (index * size_of::<u64>() as u64))?;

        let len = self.data.try_get_u16(offset)?;
        Ok(self.data.try_get_str(offset + 2, len)?)
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
        // Validate that the first 4 bytes equals "PKG4".
        if data.try_get_u32(0)? != 0x34474B50 {
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

#[derive(Debug)]
pub struct NxNode {
    name: u32,
    children: u32,
    count: u16,
    data_type: NodeType,
    data: u64,
}

#[derive(Error, Debug)]
pub enum NxError {
    #[error("failed to load nx file")]
    Io(#[from] std::io::Error),

    #[error("the header's magic bytes are invalid")]
    InvalidMagicBytes,

    #[error("{0} is out of bounds")]
    OutOfBoundsIndex(usize),

    #[error("{0}..{1} is out of bounds")]
    OutOfBoundsRange(usize, usize),

    #[error("invalid cast")]
    InvalidCast(#[from] core::array::TryFromSliceError),

    #[error("invalid string")]
    InvalidString(#[from] core::str::Utf8Error),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NodeType {
    Empty = 0,
    Integer = 1,
    Float = 2,
    String = 3,
    Vector = 4,
    Bitmap = 5,
    Audio = 6,
}

impl From<u16> for NodeType {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Empty,
            1 => Self::Integer,
            2 => Self::Float,
            3 => Self::String,
            4 => Self::Vector,
            5 => Self::Bitmap,
            6 => Self::Audio,
            _ => Self::Empty,
        }
    }
}

trait NxTryGet {
    fn try_get_node(&self, index: u64) -> Result<NxNode, NxError>;

    fn try_get_u16(&self, index: u64) -> Result<u16, NxError>;

    fn try_get_u32(&self, index: u64) -> Result<u32, NxError>;

    fn try_get_u64(&self, index: u64) -> Result<u64, NxError>;

    fn try_get_str(&self, index: u64, len: u16) -> Result<&str, NxError>;
}

impl NxTryGet for [u8] {
    fn try_get_node(&self, index: u64) -> Result<NxNode, NxError> {
        let usize_index = index as usize;
        let node_table = self
            .get(usize_index..)
            .ok_or(NxError::OutOfBoundsIndex(usize_index))?;

        // TODO: should we try to get the actual string here?
        let name = node_table.try_get_u32(0)?;
        let children = node_table.try_get_u32(4)?;
        let count = node_table.try_get_u16(8)?;
        let data_type = node_table.try_get_u16(10)?.into();
        let data = node_table.try_get_u64(12)?;

        Ok(NxNode {
            name,
            children,
            count,
            data_type,
            data,
        })
    }

    fn try_get_u16(&self, index: u64) -> Result<u16, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u16>();

        let bytes = self
            .get(usize_index..usize_index + offset)
            .ok_or(NxError::OutOfBoundsRange(usize_index, usize_index + offset))?;

        Ok(u16::from_le_bytes(bytes.try_into()?))
    }

    fn try_get_u32(&self, index: u64) -> Result<u32, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u32>();

        let bytes = self
            .get(usize_index..usize_index + offset)
            .ok_or(NxError::OutOfBoundsRange(usize_index, usize_index + offset))?;

        Ok(u32::from_le_bytes(bytes.try_into()?))
    }

    fn try_get_u64(&self, index: u64) -> Result<u64, NxError> {
        let usize_index = index as usize;
        let offset = size_of::<u64>();

        let bytes = self
            .get(usize_index..usize_index + offset)
            .ok_or(NxError::OutOfBoundsRange(usize_index, usize_index + offset))?;

        Ok(u64::from_le_bytes(bytes.try_into()?))
    }

    fn try_get_str(&self, index: u64, len: u16) -> Result<&str, NxError> {
        let usize_index = index as usize;
        let usize_len = len as usize;

        let bytes =
            self.get(usize_index..usize_index + usize_len)
                .ok_or(NxError::OutOfBoundsRange(
                    usize_index,
                    usize_index + usize_len,
                ))?;

        Ok(str::from_utf8(bytes)?)
    }
}
