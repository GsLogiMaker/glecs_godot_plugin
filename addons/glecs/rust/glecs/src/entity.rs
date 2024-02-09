
use std::collections::HashMap;
use std::fmt::Debug;

use flecs::EntityId;
use godot::engine::notify::ObjectNotification;
use godot::engine::Script;
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
#[class(base=Object)]
pub struct _BaseGEEntity {
    #[base] pub(crate) base: Base<Object>,
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
    
    /// Attaches the component of the given class to this entity.
    #[func]
    fn add_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        EntityLike::add_component(self, component)
    }

    /// Returns a componently previously attached to this entity.
    #[func]
    fn get_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        EntityLike::get_component(self, component)
    }

    /// Removes the given component from this entity.
    #[func]
    fn remove_component(&mut self, component:Gd<Script>) {
        EntityLike::remove_component(self, component);
    }

    /// Returns the entity's name.
    #[func]
    fn get_name(&self) -> String {
        EntityLike::get_name(self)
    }

    /// Sets the entity's name.
    #[func]
    fn set_name(&self, value:String) {
        EntityLike::set_name(self, value)
    }

    /// Adds a relationship from this entity to another.
    #[func]
    fn add_relation(
        &mut self,
        relation:Variant,
        mut with_entity:Gd<_BaseGEEntity>,
    ) {
        EntityLike::add_relation(self, relation, &mut with_entity)
    }

    /// Removes a previously initiated relationship.
    #[func]
    fn remove_relation(
        &mut self,
        relation:Gd<_BaseGEEntity>,
        mut with_entity:Gd<_BaseGEEntity>,
    ) {
        EntityLike::remove_relation(self, &relation, &mut with_entity)
    }

    fn get_owned_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        return None;
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

    fn add_component_to_owned(
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
    
    fn get_owned_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
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

    fn add_component_to_owned(
        &mut self,
        component_gd:Gd<_BaseGEComponent>,
    ) {
        self.bind_mut().add_component_to_owned(component_gd)
    }
    
    fn get_owned_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        self.bind().get_owned_component(flecs_id)
    }
}

pub(crate) trait EntityLike: Debug {
    fn get_world(&self) -> Gd<_BaseGEWorld>;
    fn get_flecs_id(&self) -> EntityId;

    fn add_component_to_owned(
        &mut self,
        component_gd:Gd<_BaseGEComponent>,
    ) {
    }

    fn get_owned_component(&self, flecs_id:EntityId) -> Option<Gd<_BaseGEComponent>> {
        return None;
    }

    fn add_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let mut world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_definition = world_gd
            .bind_mut()
            .get_or_add_component(&component);

        let world = world_gd.bind();

        unsafe {
            flecs::ecs_add_id(
                world.world.raw(),
                flecs_id,
                component_definition.flecs_id,
            )
        };

        // Get component data
        let Some(mut entt) = world.world.find_entity(flecs_id)
            else { 
                show_error!(
                    "Failed to get component from entity",
                    "Entity {:?} was freed.",
                    self,
                );
                unreachable!();
                return None;
            };
        if !entt.has_id(component_definition.flecs_id) {
            show_error!(
                "Failed to get component from entity",
                "Component {} has not been added to entity {:?}.",
                    component,
                    self,
            );
            return None;
        }
        let component_data = entt.get_mut_dynamic(
            &component_definition.name.to_string()
        );

        // Initialize component properties
        // TODO: Initialize properties in deterministic order
        for property_name in component_definition.parameters.keys() {
            // TODO: Get default values of properties
            let default_value = Variant::nil();
            _BaseGEComponent::_initialize_property(
                component_data,
                component_definition.as_ref(),
                property_name.clone(),
                default_value,
            );
        }

        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                data: component_data,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn get_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let mut world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();
        
        let world = world_gd.bind();

        // Get component description
        let Some(component_definition) = world
            .get_component_description(&component)
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
            self.get_owned_component(component_definition.flecs_id)
        {
            return Some(component.clone());
        }

        // Get flecs entity
        let component_symbol = component_definition.name.to_string();
        let Some(mut entt) = world.world.find_entity(flecs_id)
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
        let component_data = entt.get_mut_dynamic(&component_symbol);

        
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                data: component_data,
                component_definition: component_definition.clone(),
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        // Add to cache
        self.add_component_to_owned(
            comp.clone(),
        );

        Some(comp)
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

    fn remove_component(&mut self, component:Gd<Script>) {
        let mut world_gd = self.get_world();
        let flecs_id = self.get_flecs_id();

        let component_definition = world_gd
            .bind_mut()
            .get_or_add_component(&component);

        let world = world_gd.bind();

        unsafe { flecs::ecs_remove_id(
            world.world.raw(),
            flecs_id,
            component_definition.flecs_id,
        ) };
    }


    fn add_relation(&mut self, relation:Variant, with_entity:&impl EntityLike) {
        let self_id = self.get_flecs_id();

        let raw_world = self.get_world().bind_mut().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            self.get_world().bind_mut().get_or_add_tag_entity(relation),
            with_entity.get_flecs_id()
        ) };
        unsafe { flecs::ecs_add_id(raw_world, self_id, pair) };
    }

    fn remove_relation(&mut self, relation:&impl EntityLike, with_entity:&impl EntityLike) {
        let self_id = self.get_flecs_id();

        let raw_world = self.get_world().bind_mut().world.raw();
        let pair = unsafe { flecs::ecs_make_pair(
            relation.get_flecs_id(),
            with_entity.get_flecs_id()
        ) };
        unsafe { flecs::ecs_remove_id(raw_world, self_id, pair) };
    }
}