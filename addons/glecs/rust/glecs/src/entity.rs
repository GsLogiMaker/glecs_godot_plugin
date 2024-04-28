
use std::ffi::c_void;
use std::fmt::Debug;

use flecs::EntityId;
use godot::engine::Engine;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsComponent;
use crate::Int;
use crate::show_error;
use crate::world::_GlecsWorld;

pub(crate) static FREED_BY_ENTITY_TAG:&str = "freed_by_entity";


#[derive(GodotClass, Debug)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsEntity {
    pub(crate) base: Base<RefCounted>,
    /// The world this entity is from.
    pub(crate) world: Gd<_GlecsWorld>,
    /// The ID of this entity.
    pub(crate) id: EntityId,
}
#[godot_api]
impl _GlecsEntity {
    #[func]
    fn _spawn(id: Int, world: Option<Gd<_GlecsWorld>>) -> Gd<Self> {
        // Use a default world if world is none
        let world = match world {
            Some(w) => w,
            None => {
                Engine::singleton().get_singleton("".into())
                    .unwrap()
                    .cast::<_GlecsWorld>()
            },
        };

        // Create new entity if ID is zero
        let entity_id = if id == 0 {
            let world_ptr = world.bind().raw();
            unsafe { flecs::ecs_new_id(world_ptr) }
        } else {
            id as EntityId
        };

        let mut entity = Gd::from_init_fn(|base| {
            Self { base, id: entity_id, world }
        });
        entity.set_script(
            load::<Script>("res://addons/glecs/gd/entity.gd").to_variant(),
        );
        entity
    }

    #[func]
    fn _add_component(
        &mut self,
        component: Variant,
        data:Variant,
    ) -> Option<Gd<_GlecsComponent>> {
        EntityLike::add_component(self, component, data)
    }

    /// Returns a componently previously attached to this entity.
    #[func]
    fn _get_component(&mut self, component: Variant) -> Option<Gd<_GlecsComponent>> {
        EntityLike::get_component(self, component)
    }

    /// Removes the given component from this entity.
    #[func]
    fn _remove_component(&mut self, component: Variant) {
        EntityLike::remove_component(self, component);
    }

    #[func]
    fn _delete(&self) {
        unsafe { flecs::ecs_delete(self.world.bind().raw(), self.id) };
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
    fn _get_world(&self) -> Gd<_GlecsWorld> {
        EntityLike::get_world(self)
    }
}
#[godot_api]
impl IRefCounted for _GlecsEntity {
    fn to_string(&self) -> GString {
        GString::from(format!(
            "{}:<{}#{}>",
            EntityLike::get_name(self),
            self.base().get_class(),
            self.base().instance_id(),
        ))
    }
}
impl EntityLike for _GlecsEntity {
    fn get_world(&self) -> Gd<_GlecsWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.id
    }
}
 impl EntityLike for Gd<_GlecsEntity> {
    fn get_world(&self) -> Gd<_GlecsWorld> {
        let world = self.bind()._get_world();
        world
    }

    fn get_flecs_id(&self) -> EntityId {
        let id = self.bind().get_flecs_id();
        id
    }
}

pub(crate) trait EntityLike: Debug {
    fn get_world(&self) -> Gd<_GlecsWorld>;
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
    ) -> Option<Gd<_GlecsComponent>> {
        self.validate();

        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_id = _GlecsWorld::variant_to_entity_id(
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
            let base_comp = _GlecsComponent {
                base,
                world: world_gd.clone(),
                get_data_fn_ptr: _GlecsComponent::new_default_data_getter(
                    self.get_flecs_id()
                ),
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
        world_gd: Gd<_GlecsWorld>,
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
            let initial_data = _GlecsComponent
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

    fn get_component(&mut self, component: Variant) -> Option<Gd<_GlecsComponent>> {
        self.validate();

        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();
        
        // Get component ID
        let c_id = _GlecsWorld::variant_to_entity_id(
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
            let base_comp = _GlecsComponent {
                base,
                world: world_gd_clone,
                get_data_fn_ptr: _GlecsComponent::new_default_data_getter(
                    self.get_flecs_id()
                ),
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

        let component_id = _GlecsWorld::variant_to_entity_id(
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

    fn add_id(&mut self) { todo!() }
    fn has_id(&mut self) { todo!() }
    fn remove_id(&mut self) { todo!() }

    fn get_name(&self) -> String {
        self.validate();

        let entt = self.get_world()
            .bind()
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();
        entt.name().into()
    }

    fn set_name(&self, value: String) {
        self.validate();

        let world = self.get_world();
        let entt = world
            .bind()
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();

        entt.named(&value);
    }

    fn add_relation(&mut self, relation:Variant, target:Variant) {
        self.validate();

        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _GlecsWorld::variant_to_entity_id(world.clone(), relation),
            _GlecsWorld::variant_to_entity_id(world, target),
        ) };
        unsafe { flecs::ecs_add_id(raw_world, self_id, pair) };
    }

    fn remove_relation(&mut self, relation: Variant, target:Variant) {
        self.validate();

        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _GlecsWorld::variant_to_entity_id(world.clone(), relation),
            _GlecsWorld::variant_to_entity_id(world, target),
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
        }
    }
}