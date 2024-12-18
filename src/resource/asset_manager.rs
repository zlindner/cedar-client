use std::{collections::HashMap, path::Path};

use nx_pkg4::{file::NxFile, node::NxNode};

pub struct AssetManager {
    nx: HashMap<NxFileType, NxFile>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut nx = HashMap::new();

        nx.insert(
            NxFileType::Map001,
            NxFile::open(Path::new("nx/Map001.nx")).unwrap(),
        );
        nx.insert(NxFileType::Ui, NxFile::open(Path::new("nx/UI.nx")).unwrap());

        Self { nx }
    }

    pub fn nx(&self, file_type: NxFileType) -> NxNode {
        self.nx.get(&file_type).unwrap().root()
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum NxFileType {
    Map001,
    Ui,
}
