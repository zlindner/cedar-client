use std::{
    any::TypeId,
    cell::{Ref, RefCell},
    collections::HashMap,
    hash::Hasher,
};

use downcast_rs::{impl_downcast, Downcast};

use crate::engine::resource::NxManager;

pub struct World {
    resources: HashMap<ResourceTypeId, RefCell<Box<dyn Resource>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources
            .insert(ResourceTypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    pub fn get_resource<T: Resource>(&self) -> Option<Ref<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.resources
            .get(type_id)
            .map(|x| Ref::map(x.borrow(), |inner| inner.downcast_ref::<T>().unwrap()))
    }

    pub fn nx(&self) -> Ref<NxManager> {
        self.get_resource::<NxManager>()
            .expect("NxManager should exist")
    }
}

pub trait Resource: 'static + Downcast {}

impl<T> Resource for T where T: 'static {}
impl_downcast!(Resource);

#[derive(Copy, Clone, Debug, Eq, PartialOrd, Ord)]
pub struct ResourceTypeId {
    type_id: TypeId,
    name: &'static str,
}

impl ResourceTypeId {
    /// Returns the resource type ID of the given resource type.
    pub fn of<T: Resource>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }
}

impl std::hash::Hash for ResourceTypeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl PartialEq for ResourceTypeId {
    fn eq(&self, other: &Self) -> bool {
        self.type_id.eq(&other.type_id)
    }
}
