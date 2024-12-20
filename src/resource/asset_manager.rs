use std::{collections::HashMap, path::Path, sync::mpsc};

use nx_pkg4::{NxBitmap, NxFile, NxNode};

use crate::graphics::RendererEvent;

pub struct AssetManager {
    nx: HashMap<NxFileType, NxFile>,
    bitmaps: Vec<String>,

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
            bitmaps: Vec::new(),
            renderer_tx,
        }
    }

    pub fn nx(&self, file_type: &NxFileType) -> NxNode {
        self.nx.get(&file_type).unwrap().root()
    }

    pub fn get_bitmaps(&self) -> &Vec<String> {
        &self.bitmaps
    }

    pub fn register_bitmap(&mut self, name: &str, bitmap: NxBitmap) {
        // TODO: should log/handle case where bitmap is already registered.
        let width = bitmap.width;
        let height = bitmap.height;

        self.bitmaps.push(name.to_string());

        if let Err(e) = self
            .renderer_tx
            .send(RendererEvent::RegisterBitmap(name.to_string(), bitmap))
        {
            log::error!("Error sending RegisterBitmap event: {}", e);
        } else {
            log::info!(
                "Successfully registered bitmap {} width: {} height: {}",
                name,
                width,
                height
            );
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum NxFileType {
    Map001,
    Ui,
}
