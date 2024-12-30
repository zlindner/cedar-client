use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    hash::Hasher,
};

use downcast_rs::{impl_downcast, Downcast};
use hecs::{DynamicBundle, Entity, Query, QueryBorrow, QueryMut, World};

use crate::resource::{AssetManager, WindowProxy};

pub struct State {
    world: World,
    resources: HashMap<ResourceTypeId, RefCell<Box<dyn Resource>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        self.world.spawn(components)
    }

    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.world.query_mut::<Q>()
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

    pub fn get_resource_mut<T: Resource>(&self) -> Option<RefMut<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.resources
            .get(type_id)
            .map(|x| RefMut::map(x.borrow_mut(), |inner| inner.downcast_mut::<T>().unwrap()))
    }

    pub fn assets(&self) -> Ref<AssetManager> {
        self.get_resource::<AssetManager>()
            .expect("AssetManager should exist")
    }

    pub fn window(&self) -> RefMut<WindowProxy> {
        self.get_resource_mut::<WindowProxy>()
            .expect("WindowProxy should exist")
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
