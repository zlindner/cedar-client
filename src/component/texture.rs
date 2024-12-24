use std::fmt;

use nx_pkg4::{Node, NxError, NxNode};

pub struct Texture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data: Vec<u8>,

    /// The value of the `origin` child node.
    pub(crate) origin: Option<(i32, i32)>,

    /// The value of the `z` child node.
    layer: Option<i64>,
}

impl Texture {
    pub fn load(node: NxNode) -> Result<Option<Self>, NxError> {
        let origin = match node.get("origin") {
            Some(child) => child.vector()?,
            None => None,
        };

        let layer = match node.get("z") {
            Some(child) => child.integer()?,
            None => None,
        };

        let bitmap = match node.bitmap()? {
            Some(bitmap) => bitmap,
            None => {
                return Ok(None);
            }
        };

        Ok(Some(Self {
            width: bitmap.width.into(),
            height: bitmap.height.into(),
            data: bitmap.data,
            origin,
            layer,
        }))
    }
}

/// Manually implementing Debug for Texture, replacing data with an empty slice since it can
/// contain hundreds of elements and isn't useful to log.
impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[derive(Debug)]
        #[allow(unused)]
        struct Texture<'a> {
            width: &'a u32,
            height: &'a u32,
            data: [u8; 0],
            origin: &'a Option<(i32, i32)>,
            layer: &'a Option<i64>,
        }

        let Self {
            width,
            height,
            data: _,
            origin,
            layer,
        } = self;

        fmt::Debug::fmt(
            &Texture {
                width,
                height,
                data: [],
                origin,
                layer,
            },
            f,
        )
    }
}
