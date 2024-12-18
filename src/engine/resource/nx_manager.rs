use std::{collections::HashMap, path::Path};

use nx_pkg4::{file::NxFile, node::NxNode};

pub struct NxManager {
    data: HashMap<NxFileType, NxFile>,
}

impl NxManager {
    pub fn new() -> Self {
        let mut data = HashMap::new();

        data.insert(
            NxFileType::Map001,
            NxFile::open(Path::new("nx/Map001.nx")).unwrap(),
        );
        data.insert(NxFileType::Ui, NxFile::open(Path::new("nx/UI.nx")).unwrap());

        Self { data }
    }

    pub fn get(&self, file_type: NxFileType) -> NxNode {
        self.data.get(&file_type).unwrap().root()
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum NxFileType {
    Map001,
    Ui,
}
