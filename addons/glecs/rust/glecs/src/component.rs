
use std::ptr::NonNull;

use flecs::EntityId;
use godot::prelude::*;

use crate::entity::EntityLike;
use crate::gd_bindings::_GlecsBindings;
use crate::gd_bindings::_GlecsComponents;
use crate::show_error;
use crate::world::_GlecsBaseWorld;

/// An ECS component.
#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsBaseComponent {
    pub(crate) base: Base<RefCounted>,
    pub(crate) world: Gd<_GlecsBaseWorld>,
    /// The ID that this component is attatached to.
    pub(crate) entity_id: EntityId,
    pub(crate) component_id: EntityId,
}
#[godot_api]
impl _GlecsBaseComponent {
    /// Copies the data from the given component to this one.
    #[func]
    fn _copy_from_component(&mut self, from_component:Gd<_GlecsBaseComponent>) {
        EntityLike::validate(self);
        if self.get_flecs_id() != from_component.bind().get_flecs_id() {
            show_error!(
                "Failed to copy component",
                "Destination component is of type {}, but source component is of type {}",
                self.base().get_script(),
                from_component.bind().base().get_script(),
            )
        }

        let gd_component_data = _GlecsComponents::_get_gd_component_data(
            self.world.bind().raw(),
            self.component_id,
        ).expect(&format!("Unable to copy to component {}", self));

        unsafe {
            std::slice::from_raw_parts_mut(
                self.get_data().as_mut(),
                gd_component_data.size() as usize,
            ).copy_from_slice(
                std::slice::from_raw_parts(
                    from_component.bind().get_data().as_ptr(),
                    gd_component_data.size() as usize,
                ),
            );
        }
    }

    /// Returns the name of the the type of this component.
    #[func]
    fn _get_type_name(&self) -> StringName {
        EntityLike::validate(self);

        _GlecsBindings::get_name(
            self.world.clone(),
            self.component_id,
        ).into()
    }

    /// Returns a property from the component data.
    #[func]
    fn _getc(&self, property: StringName) -> Variant {
        EntityLike::validate(self);

        _GlecsComponents::get(
            self.world.clone(),
            self.entity_id,
            self.component_id,
            property,
        )
    }

    /// Sets a property in the component data.
    #[func]
    fn _setc(&mut self, property: StringName, value:Variant) {
        EntityLike::validate(self);

        _GlecsComponents::set(
            self.world.clone(),
            self.entity_id,
            self.component_id,
            property,
            value,
        );
    }

    #[func]
    fn _delete(&self) {
        EntityLike::delete(self)
    }

    /// Override default 'free' behavior (This only works if the
    /// variable is staticly typed in GdScript.)
    #[func]
    fn free(&self) {
        EntityLike::delete(self)
    }

    #[func]
    fn _is_valid(&self) -> bool {
        EntityLike::is_valid(self)
    }

    fn get_data(&self) -> NonNull<u8> {
        unsafe { NonNull::new_unchecked(flecs::ecs_get_mut_id(
            self.world.bind().raw(),
            self.entity_id,
            self.get_flecs_id(),
        ).cast::<u8>()) }
    }

    /// Returns the Flecs ID of this component's type.
    pub(crate) fn get_flecs_id(&self) -> EntityId {
        self.component_id
    }
}
impl std::fmt::Display for _GlecsBaseComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "<{}#{}>",
            self._get_type_name(),
            self.component_id,
        )
    }
}


impl EntityLike for _GlecsBaseComponent {
    fn get_world(&self) -> Gd<_GlecsBaseWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.component_id
    }

    fn delete(&self) {
        unsafe { flecs::ecs_remove_id(
            self.world.bind().raw(),
            self.entity_id,
            self.get_flecs_id(),
        ) };
    }

    fn is_valid(&self) -> bool{
        // Check world
        let Some(_) = self.world.is_instance_valid()
            .then_some(())
            else { return false };

        // Check master entity
        let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), self.entity_id)
            .then_some(())
            else { return false };

        // Check component type is alive
        match self.get_flecs_id() {
            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_first(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the first part is a component
                let id = _GlecsBindings::pair_first(c);
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    else { return false };
            },

            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_second(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the second part is a component
                let id = _GlecsBindings::pair_second(c);
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    else { return false };
            },

            c => {
                // ID is a normal component
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), c)
                    .then_some(())
                    else { return false };
            },

        }

        // Check that the entity has this component attached
        let ett_id = self.entity_id;
        let comp_id = self.get_flecs_id();
        let Some(_) = _GlecsBindings::has_id(self.world.clone(), ett_id, comp_id)
            .then_some(())
            else { return false };

        return true;
    }

    fn validate(&self) {
        // Check world
        self.world.is_instance_valid()
            .then_some(())
            .expect("Component's world was deleted");

        // Check master entity
        _GlecsBindings::id_is_alive(self.world.clone(), self.entity_id)
            .then_some(())
            .expect("The entity this component was attached to was delted.");

        // Check component type is alive
        match self.get_flecs_id() {
            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_first(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the first part is a component
                let id = _GlecsBindings::pair_first(c);
                _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_second(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the second part is a component
                let id = _GlecsBindings::pair_second(c);
                _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

            c => {
                // ID is a normal component
                _GlecsBindings::id_is_alive(self.world.clone(), c)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

        }

        // Check that the entity has this component attached
        let ett_id = self.entity_id;
        let comp_id = self.get_flecs_id();
        _GlecsBindings::has_id(self.world.clone(), ett_id, comp_id)
            .then_some(())
            .expect(&format!(
                "Component was removed from its entity. Component ID: {}, Entity ID: {}",
                comp_id,
                ett_id,
            ));
    }
}
impl std::fmt::Debug for _GlecsBaseComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("_GlecsComponent")
            .field("entity", &self.entity_id)
            .field("component", &self.component_id)
            .field("world", &self.world)
            .finish()
    }
}
