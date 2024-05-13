
use std::fmt::Debug;

use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::entity::EntityLike;
use crate::world::_GlecsBaseWorld;

pub(crate) fn load_entity_script() -> Variant {
    load::<Script>("res://addons/glecs/gd/module.gd")
        .to_variant()
}

#[derive(GodotClass, Debug)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsBaseModule {
    pub(crate) base: Base<RefCounted>,
    /// The world this entity is from.
    pub(crate) world: Gd<_GlecsBaseWorld>,
    /// The ID of this entity.
    pub(crate) id: EntityId,
}
#[godot_api]
impl _GlecsBaseModule {
}

#[godot_api]
impl IRefCounted for _GlecsBaseModule {
    fn to_string(&self) -> GString {
        GString::from(format!(
            "{}:<{}#{}>",
            EntityLike::get_name(self),
            self.base().get_class(),
            self.base().instance_id(),
        ))
    }
}
impl EntityLike for _GlecsBaseModule {
    fn is_valid(&self) -> bool {
        if !self.world.is_instance_valid() {
            // World was deleted
            return false;
        }

        let flecs_id = self.get_flecs_id();
        if !unsafe { flecs::ecs_is_alive(
            self.world.bind().raw(),
            flecs_id,
        ) } {
            // Entity was deleted
            return false
        }

        return true;
    }

    fn get_world(&self) -> Gd<_GlecsBaseWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.id
    }
}
