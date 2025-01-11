use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    hash::Hasher,
};

use downcast_rs::{impl_downcast, Downcast};

use crate::{
    graphics::{
        ui::{Button, Text, TextInput},
        Sprite,
    },
    resource::{Cursor, WindowProxy},
};

// TODO: maybe we can have a "UI" field that contains buttons, images, text fields, etc.
pub struct State {
    resources: HashMap<ResourceTypeId, RefCell<Box<dyn Resource>>>,
    pub sprites: Vec<Sprite>,
    pub buttons: Vec<Button>,
    pub text_inputs: Vec<TextInput>,
    pub text: Vec<Text>,
}

impl State {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            sprites: Vec::new(),
            buttons: Vec::new(),
            text_inputs: Vec::new(),
            text: Vec::new(),
        }
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.resources
            .insert(ResourceTypeId::of::<T>(), RefCell::new(Box::new(resource)));

        self
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

    pub fn cursor(&self) -> RefMut<Cursor> {
        self.get_resource_mut::<Cursor>()
            .expect("Cursor should exist")
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
