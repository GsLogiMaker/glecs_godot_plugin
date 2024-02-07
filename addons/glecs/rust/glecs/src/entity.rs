
use flecs::EntityId;
use godot::engine::notify::ObjectNotification;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::world::_BaseGEWorld;


#[derive(GodotClass)]
#[class(base=Object)]
pub struct _BaseGEEntity {
    #[base] pub(crate) base: Base<Object>,
    /// The world this entity is from.
    pub(crate) world: Gd<_BaseGEWorld>,
    /// The ID of this entity.
    pub(crate) id: EntityId,
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
                godot_error!(
                    "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
                );
                return None;
            };

        // Get flecs entity
        let component_symbol = component_definition.name.to_string();
        let Some(mut entt) = world.world.find_entity(self.id)
            else { 
                godot_error!(
                    "Failed to get component from entity. Entity {} was freed.",
                    self.to_gd(),
                );
                return None;
            };
        
        // Get component data
        if !entt.has_id(component_definition.flecs_id) {
            godot_error!(
                "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
            );
            return None;
        }
        let component_data = entt.get_mut_dynamic(&component_symbol);

        
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                flecs_id: component_definition.flecs_id,
                data: component_data,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    #[func]
    fn add_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let component_definition = self.world
            .bind_mut()
            .get_or_add_component(&component);

        let world = self.world.bind();

        unsafe {
            flecs::ecs_add_id(
                world.world.raw(),
                self.id,
                component_definition.flecs_id,
            )
        };

        // Get component data
        let Some(mut entt) = world.world.find_entity(self.id)
            else { 
                godot_error!(
                    "Failed to get component from entity. Entity {} was freed.",
                    self.to_gd(),
                );
                return None;
            };
        if !entt.has_id(component_definition.flecs_id) {
            godot_error!(
                "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
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
                flecs_id: component_definition.flecs_id,
                data: component_data,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn flecs_free(&self) {
        let entt = self.world.bind().world.find_entity(self.id).unwrap();
        entt.destruct()
    }
}
#[godot_api]
impl IObject for _BaseGEEntity {
    fn on_notification(&mut self, what: ObjectNotification) {
        match what {
            ObjectNotification::Predelete => {
                self.flecs_free()
            },
            _ => {},
        }
    }
}