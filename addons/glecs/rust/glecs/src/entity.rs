
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Debug;

use flecs::EntityId;
use godot::engine::notify::ObjectNotification;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::show_error;
use crate::world::_BaseGEWorld;

pub(crate) static FREED_BY_ENTITY_TAG:&str = "freed_by_entity";

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


#[derive(GodotClass, Debug)]
#[class(base=Object, no_init)]
pub struct _BaseGEEntity {
    pub(crate) base: Base<Object>,
    /// The world this entity is from.
    pub(crate) world: Gd<_BaseGEWorld>,
    /// The ID of this entity.
    pub(crate) id: EntityId,
    /// Is *true* when the world is deleting this entity.
    pub(crate) world_deletion: bool,
    /// Maps Flecs component type ID to Godot component objects.
    pub(crate) gd_components_map:HashMap<EntityId, Gd<_BaseGEComponent>>,
}
#[godot_api]
impl _BaseGEEntity {
    
    #[func]
    fn _add_component(
        &mut self,
        component: Variant,
        data:Variant,
    ) -> Option<Gd<_BaseGEComponent>> {
        EntityLike::add_component(self, component, data)
    }

    /// Returns a componently previously attached to this entity.
    #[func]
    fn _get_component(&mut self, component: Variant) -> Option<Gd<_BaseGEComponent>> {
        EntityLike::get_component(self, component)
    }

    /// Removes the given component from this entity.
    #[func]
    fn _remove_component(&mut self, component: Variant) {
        EntityLike::remove_component(self, component);
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

    pub(crate) fn free_component(&self, mut component:Gd<_BaseGEComponent>) {
        {
            let mut bind = component.bind_mut();
            let mut comp_base = bind.base_mut();
            comp_base.set_meta(FREED_BY_ENTITY_TAG.into(), Variant::from(true));
        }
        component.free();
    }

    fn on_free(&mut self) {
        // Free component data references
        for (_, component) in &self.gd_components_map {
            self.free_component(component.clone());
        }

        // Remove reference to self from the world
        if !self.world_deletion {
            let entt = self.world.bind().world.find_entity(self.id).unwrap();
            entt.destruct();
            let id = self.id;
            self.world.bind_mut().on_entity_freed(id);
        }
    }
}
#[godot_api]
impl IObject for _BaseGEEntity {
    fn on_notification(&mut self, what: ObjectNotification) {
        match what {
            ObjectNotification::Predelete => {
                self.on_free()
            },
            _ => {},
        }
    }

    fn to_string(&self) -> GString {
        GString::from(format!(
            "{}:<{}#{}>",
            EntityLike::get_name(self),
            self.base().get_class(),
            self.base().instance_id(),
        ))
    }
}
impl EntityLike for _BaseGEEntity {
    fn get_world(&self) -> Gd<_BaseGEWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.id
    }

    fn add_component_to_cache(
        &mut self,
        component_gd:Gd<_BaseGEComponent>,
    ) {
        let component_definition = component_gd
            .bind()
            .component_definition
            .clone();
        self.gd_components_map.insert(
            component_definition.flecs_id,
            component_gd.clone(),
        );
    }
    
    fn get_cached_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        self.gd_components_map
            .get(&flecs_id)
            .map(|x| {(*x).clone()})
    }
}
 impl EntityLike for Gd<_BaseGEEntity> {
    fn get_world(&self) -> Gd<_BaseGEWorld> {
        self.bind().get_world()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.bind().get_flecs_id()
    }

    fn add_component_to_cache(
        &mut self,
        component_gd:Gd<_BaseGEComponent>,
    ) {
        self.bind_mut().add_component_to_cache(component_gd)
    }
    
    fn get_cached_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        self.bind().get_cached_component(flecs_id)
    }
}

pub(crate) trait EntityLike: Debug {
    fn get_world(&self) -> Gd<_BaseGEWorld>;
    fn get_flecs_id(&self) -> EntityId;

    fn add_component_to_cache(
        &mut self,
        _component_gd:Gd<_BaseGEComponent>,
    ) {
    }

    fn get_cached_component(&self, _flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        return None;
    }

    fn add_component(
        &mut self,
        component: Variant,
        with_data: Variant,
    ) -> Option<Gd<_BaseGEComponent>> {
        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        
        Self::add_component_raw(
            world_gd.clone(),
            flecs_id,
            component.clone(),
            with_data,
        );
        
        let component_id = _BaseGEWorld::variant_to_entity_id(
            world_gd.clone(),
            component.clone(),
        );
        // Create Godot wrapper
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                world: world_gd.clone(),
                get_data_fn_ptr: _BaseGEComponent::new_default_data_getter(
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
        world_gd: Gd<_BaseGEWorld>,
        raw_entity: EntityId,
        component: Variant,
        with_data: Variant,
    ) {
        let component_id = _BaseGEWorld
            ::variant_to_entity_id(world_gd.clone(), component);

        let world_raw = world_gd.bind().world.raw();
        let initial_data = _BaseGEComponent
            ::create_initial_data(
                &world_gd.bind()
                    .get_component_description(component_id)
                    .unwrap(),
                with_data,
            );

        // Add component to entity
        // TODO: Fix zero sized components
        unsafe { flecs::ecs_set_id(
            world_raw,
            raw_entity,
            component_id,
            initial_data.len(),
            initial_data.as_ptr().cast::<c_void>(),
        ) };
    }

    fn get_component(&mut self, component: Variant) -> Option<Gd<_BaseGEComponent>> {
        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();
        
        // Get component ID
        let c_id = _BaseGEWorld::variant_to_entity_id(
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

        // Return early if the component object is cached
        if let Some(component) =
            self.get_cached_component(component_definition.flecs_id)
        {
            return Some(component.clone());
        }

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
            let base_comp = _BaseGEComponent {
                base,
                world: world_gd_clone,
                get_data_fn_ptr: _BaseGEComponent::new_default_data_getter(
                    self.get_flecs_id()
                ),
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        // Add to cache
        self.add_component_to_cache(
            comp.clone(),
        );

        Some(comp)
    }

    fn remove_component(&mut self, component: Variant) {
        let world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_id = _BaseGEWorld::variant_to_entity_id(
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

    fn get_name(&self) -> String {
        let entt = self.get_world()
            .bind()
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();
        entt.name().into()
    }

    fn set_name(&self, value:String) {
        self.set_name_by_ref(value, &self.get_world().bind());
    }

    fn set_name_by_ref(&self, mut value:String, world:&_BaseGEWorld) {
        let entt = world
            .world
            .find_entity(self.get_flecs_id())
            .unwrap();

        while world.world.lookup(&value).is_some() {
            increment_name(&mut value);
        }
        entt.named(&value);
    }

    fn add_relation(&mut self, relation:Variant, target:Variant) {
        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _BaseGEWorld::variant_to_entity_id(world.clone(), relation),
            _BaseGEWorld::variant_to_entity_id(world, target),
        ) };
        unsafe { flecs::ecs_add_id(raw_world, self_id, pair) };
    }

    fn remove_relation(&mut self, relation: Variant, target:Variant) {
        let self_id = self.get_flecs_id();
        let world = self.get_world();

        let raw_world = world.bind().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            _BaseGEWorld::variant_to_entity_id(world.clone(), relation),
            _BaseGEWorld::variant_to_entity_id(world, target),
        ) };
        unsafe { flecs::ecs_remove_id(raw_world, self_id, pair) };
    }
}