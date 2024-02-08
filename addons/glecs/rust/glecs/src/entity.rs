
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
    #[func]
    fn get_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let world = self.world.bind();

        // Get component description
        let Some(component_definition) = world
            .get_component_description(&component)
            else {
                show_error!(
                    "Failed to get component from entity",
                    "Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
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
        let Some(mut entt) = world.world.find_entity(self.id)
            else {
                show_error!(
                    "Failed to get component from entity",
                    "Entity {} was freed.",
                    self.to_gd(),
                );
                unreachable!();
            };
        
        // Get component data
        if !entt.has_id(component_definition.flecs_id) {
            show_error!(
                "Failed to get component from entity",
                "Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
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
        self.gd_components_map
            .insert(component_definition.flecs_id, comp.clone());

        Some(comp)
    }

    #[func]
    fn add_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        EcsIdAttachment::add_component(self, component)
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
}
impl EcsIdAttachment for _BaseGEEntity {
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

pub(crate) trait EcsIdAttachment: Debug {
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
}