
use std::fmt::Debug;

use flecs::EntityId;
use godot::prelude::*;

use crate::world::_GlecsWorld;


#[derive(GodotClass, Debug)]
#[class(base=Object, no_init)]
pub struct _GlecsEvent {
    pub(crate) base: Base<Object>,
    /// The world this entity is from.
    pub(crate) _world: Gd<_GlecsWorld>,
    /// The ID of this entity.
    pub(crate) _id: EntityId,
}
#[godot_api]
impl _GlecsEvent {
	#[func]
    fn override_id() -> EntityId {
        0
    }
}