use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::mpsc,
};

use nx_pkg4::{file::NxFile, node::NxNode, NxBitmap};

use crate::graphics::RendererEvent;

pub struct AssetManager {
    nx: HashMap<NxFileType, NxFile>,
    bitmaps: HashSet<String>,

    renderer_tx: mpsc::Sender<RendererEvent>,
}

impl AssetManager {
    pub fn new(renderer_tx: mpsc::Sender<RendererEvent>) -> Self {
        let mut nx = HashMap::new();

        nx.insert(
            NxFileType::Map001,
            NxFile::open(Path::new("nx/Map001.nx")).unwrap(),
        );
        nx.insert(NxFileType::Ui, NxFile::open(Path::new("nx/UI.nx")).unwrap());

        Self {
            nx,
            bitmaps: HashSet::new(),
            renderer_tx,
        }
    }

    pub fn nx(&self, file_type: &NxFileType) -> NxNode {
        self.nx.get(&file_type).unwrap().root()
    }

    pub fn get_bitmaps(&self) -> &HashSet<String> {
        &self.bitmaps
    }

    pub fn register_bitmap(&mut self, name: &str, bitmap: NxBitmap) {
        // TODO: should log/handle case where bitmap is already registered.

        self.bitmaps.insert(name.to_string());

        self.renderer_tx
            .send(RendererEvent::RegisterBitmap(name.to_string(), bitmap))
            .unwrap();
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum NxFileType {
    Map001,
    Ui,
}
