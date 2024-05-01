
use std::ffi::c_void;
use std::fmt::Debug;

use flecs::EntityId;
use godot::engine::Engine;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsBaseComponent;
use crate::Int;
use crate::show_error;
use crate::world::_GlecsBaseWorld;

pub(crate) fn load_entity_script() -> Variant {
    load::<Script>("res://addons/glecs/gd/glecs.gd")
        .get_script_constant_map()
        .get("Entity")
        .unwrap()
        .to_variant()
}

#[derive(GodotClass, Debug)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsBaseEntity {
    pub(crate) base: Base<RefCounted>,
    /// The world this entity is from.
    pub(crate) world: Gd<_GlecsBaseWorld>,
    /// The ID of this entity.
    pub(crate) id: EntityId,
}
#[godot_api]
impl _GlecsBaseEntity {
    #[func]
    pub(crate) fn _spawn(
        world: Option<Gd<_GlecsBaseWorld>>,
    ) -> Gd<Self> {
        // Use a default world if world is none
        let world = match world {
            Some(w) => w,
            None => {
                Engine::singleton().get_singleton("Glecs".into())
                    .unwrap()
                    .cast::<_GlecsBaseWorld>()
            },
        };

        // Create new entity
        let entity_id = unsafe { flecs::ecs_new_id(world.bind().raw()) };

        let mut entity = Gd::from_init_fn(|base| {
            Self { base, id: entity_id, world }
        });
        entity.set_script(load_entity_script());
        entity
    }

    #[func]
    pub(crate) fn _from(
        entity: Variant,
        world: Option<Gd<_GlecsBaseWorld>>,
    ) -> Gd<Self> {
        // Use a default world if world is none
        let world = match world {
            Some(w) => w,
            None => {
                Engine::singleton().get_singleton("Glecs".into())
                    .unwrap()
                    .cast::<_GlecsBaseWorld>()
            },
        };

        // Convert variant to entity ID
        let entity_id = _GlecsBaseWorld::_id_from_variant(
            world.clone(),
            entity,
        );

        let mut entity = Gd::from_init_fn(|base| {
            Self { base, id: entity_id, world }
        });
        entity.set_script(load_entity_script());
        entity
    }

    #[func]
    fn _add_component(
        &mut self,
        component: Variant,
        data:Variant,
    ) -> Option<Gd<_GlecsBaseComponent>> {
        EntityLike::add_component(self, component, data)
    }

    /// Returns a componently previously attached to this entity.
    #[func]
    fn _get_component(&mut self, component: Variant) -> Option<Gd<_GlecsBaseComponent>> {
        EntityLike::get_component(self, component)
    }

    /// Removes the given component from this entity.
    #[func]
    fn _remove_component(&mut self, component: Variant) {
        EntityLike::remove_component(self, component);
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
    fn _add_entity(&mut self, entity: Variant) {
        EntityLike::add_entity(self, entity);
    }

    #[func]
    fn _has_entity(&mut self, entity: Variant) -> bool {
        EntityLike::has_entity(self, entity)
    }

    #[func]
    fn _remove_entity(&mut self, entity: Variant) {
        EntityLike::remove_entity(self, entity);
    }

    #[func]
    fn _get_id(&self) -> Int {
        self.id as Int
    }

    /// Returns the entity's name.
    #[func]
    fn _get_name(&self) -> String {
        EntityLike::get_name(self)
    }

    /// Sets the entity's name.
    #[func]
    fn _set_name(&self, value:String) {
        EntityLike::set_name(self, value)
    }

    /// Adds a relationship from this entity to another.
    #[func]
    fn _add_relation(
        &mut self,
        relation: Variant,
        target: Variant,
    ) {
        EntityLike::add_relation(self, relation, target)
    }

    /// Removes a previously initiated relationship.
    #[func]
    fn _remove_relation(
        &mut self,
        relation: Variant,
        target: Variant,
    ) {
        EntityLike::remove_relation(self, relation, target)
    }

    #[func]
    fn _is_valid(&self) -> bool {
        EntityLike::is_valid(self)
    }

    #[func]
    fn _get_world(&self) -> Gd<_GlecsBaseWorld> {
        EntityLike::get_world(self)
    }

    #[func]
    fn _set_world(&mut self, world: Gd<_GlecsBaseWorld>) {
        self.world = world;
    }
}

#[godot_api]
impl IRefCounted for _GlecsBaseEntity {
    fn to_string(&self) -> GString {
        GString::from(format!(
            "{}:<{}#{}>",
            EntityLike::get_name(self),
            self.base().get_class(),
            self.base().instance_id(),
        ))
    }
}
impl EntityLike for _GlecsBaseEntity {
    fn get_world(&self) -> Gd<_GlecsBaseWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.id
    }
}
 impl EntityLike for Gd<_GlecsBaseEntity> {
    fn get_world(&self) -> Gd<_GlecsBaseWorld> {
        let world = self.bind()._get_world();
        world
    }

    fn get_flecs_id(&self) -> EntityId {
        let id = self.bind().get_flecs_id();
        id
    }
}

pub(crate) trait EntityLike: Debug {
    fn get_world(&self) -> Gd<_GlecsBaseWorld>;
    fn get_flecs_id(&self) -> EntityId;

    fn is_valid(&self) -> bool {
        let world_gd = self.get_world();
        if !world_gd.is_instance_valid() {
            // World was deleted
            return false;
        }

        let flecs_id = self.get_flecs_id();
        if !unsafe { flecs::ecs_is_alive(
            world_gd.bind().raw(),
            flecs_id,
        ) } {
            // Entity was deleted
            return false
        }

        return true;
    }

    fn add_component(
        &mut self,
        component: Variant,
        with_data: Variant,
    ) -> Option<Gd<_GlecsBaseComponent>> {
        self.validate();

        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_id = _GlecsBaseWorld::_id_from_variant(
            world_gd.clone(),
            component.clone(),
        );
        Self::add_component_raw(
            world_gd.clone(),
            flecs_id,
            component_id,
            with_data,
        );
        
        // Create Godot wrapper
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _GlecsBaseComponent {
                base,
                entity_id: flecs_id,
                world: world_gd.clone(),
                component_definition: world_gd.bind()
                    .get_component_description(component_id)
                    .unwrap(),
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn add_component_raw(
        world_gd: Gd<_GlecsBaseWorld>,
        raw_entity: EntityId,
        component: EntityId,
        with_data: Variant,
    ) {
        let world_raw = world_gd.bind().world.raw();
        if with_data == Variant::nil() {
            // Add component to entity
            unsafe { flecs::ecs_add_id(
                world_raw,
                raw_entity,
                component,
            ) };
        } else {
            let initial_data = _GlecsBaseComponent
                ::create_initial_data(
                    &world_gd.bind()
                        .get_component_description(component)
                        .unwrap(),
                    with_data,
                );
    
            // Add component to entity
            // TODO: Fix zero sized components
            unsafe { flecs::ecs_set_id(
                world_raw,
                raw_entity,
                component,
                initial_data.len(),
                initial_data.as_ptr().cast::<c_void>(),
            ) };
        }
    }

    fn get_component(&mut self, component: Variant) -> Option<Gd<_GlecsBaseComponent>> {
        self.validate();

        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();
        
        // Get component ID
        let c_id = _GlecsBaseWorld::_id_from_variant(
            world_gd.clone(),
            component.clone(),
        );
        
        let world = world_gd.bind();

        // Get component description
        let Some(component_definition) = world
            .get_component_description(c_id)
            else {
                show_error!(
                    "Failed to get component from entity",
                    "Component {} has not been added to entity {:?}.",
                    component,
                    self,
                );
                return None;
            };

        // Get flecs entity
        let Some(entt) = world.world.find_entity(flecs_id)
            else {
                show_error!(
                    "Failed to get component from entity",
                    "Entity {:?} was freed.",
                    self,
                );
                unreachable!();
            };
        
        // Get component data
        if !entt.has_id(component_definition.flecs_id) {
            show_error!(
                "Failed to get component from entity",
                "Component {} has not been added to entity {:?}.",
                    component,
                    self,
            );
            return None;
        }


        let world_gd_clone = world_gd.clone();
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _GlecsBaseComponent {
                base,
                world: world_gd_clone,
                entity_id: flecs_id,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn remove_component(&mut self, component: Variant) {
        self.validate();

        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_id = _GlecsBaseWorld::_id_from_variant(
            world_gd.clone(),
            component,
        );

        let world = world_gd.bind();

        unsafe { flecs::ecs_remove_id(
            world.world.raw(),
            flecs_id,
            component_id,
        ) };
    }

    fn delete(&self) {
        let world = self.get_world();
        let id = self.get_flecs_id();
        unsafe { flecs::ecs_delete(world.bind().raw(), id) };
    }

    fn add_entity(&mut self, entity: Variant) {
        self.validate();

        let world = self.get_world();
        let id = self.get_flecs_id();

        let adding_id = _GlecsBaseWorld::_id_from_variant(
            world.clone(),
            entity,
        );

        unsafe { flecs::ecs_add_id(
            world.bind().raw(),
            id,
            adding_id,
        ) };
    }

    fn has_entity(&mut self, entity: Variant) -> bool {
        self.validate();

        let world = self.get_world();
        let id = self.get_flecs_id();

        let adding_id = _GlecsBaseWorld::_id_from_variant(
            world.clone(),
            entity,
        );

        let world_ptr = world.bind().raw();
        unsafe { flecs::ecs_has_id(
            world_ptr,
            id,
            adding_id,
        ) }
    }
    fn remove_entity(&mut self, entity: Variant) {
        self.validate();

        let world = self.get_world();
        let id = self.get_flecs_id();

        let adding_id = _GlecsBaseWorld::_id_from_variant(
            world.clone(),
            entity,
        );

        let world_ptr = world.bind().raw();
        unsafe { flecs::ecs_remove_id(
            world_ptr,
            id,
            adding_id,
        ) }
    }

    fn get_name(&self) -> String {
        self.validate();

        let entt = self.get_world()
            .bind()
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();
        entt.name().into()
    }

    fn set_name(&self, mut value: String) {
        self.validate();

        let world = self.get_world();
        let entt = world
            .bind()
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();

        // Ensure name is unique
        while world.bind().world.lookup(&value).is_some() {
            increment_name(&mut value);
        }

        entt.named(&value);
    }

    fn add_relation(&mut self, relation:Variant, target:Variant) {
        self.validate();

        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _GlecsBaseWorld::_id_from_variant(world.clone(), relation),
            _GlecsBaseWorld::_id_from_variant(world, target),
        ) };
        unsafe { flecs::ecs_add_id(raw_world, self_id, pair) };
    }

    fn remove_relation(&mut self, relation: Variant, target:Variant) {
        self.validate();

        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _GlecsBaseWorld::_id_from_variant(world.clone(), relation),
            _GlecsBaseWorld::_id_from_variant(world, target),
        ) };
        unsafe { flecs::ecs_remove_id(raw_world, self_id, pair) };
    }

    /// Panics if the entity or its world were deleted.
    fn validate(&self) {
        if !self.is_valid() {
            show_error!(
                "Entity validation failed",
                "Entity or its world was deleted.",
            );
            Err(()).unwrap()
        }
    }
}

fn increment_name(name:&mut String) {
    let mut end_number = String::new();
    for x in name.chars() {
        if x.is_numeric() {
            end_number.insert(0, x);
        } else {
            break;
        }
    }

    if end_number.len() == 0 {
        name.push('1');
        return
    }

    name.truncate(name.len()-end_number.len());

    let number = end_number.parse::<u32>().unwrap();
    let new_number = number+1;

    end_number.push_str(&format!("{new_number}"));

}