
use std::alloc::Layout;

use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::world::_BaseGEWorld;
use crate::TYPE_SIZES;

/// The metadata regarding a component's structure.
#[derive(Debug, Clone)]
pub(crate) struct ComponetDefinition {
    pub(crate) name: StringName,
    pub(crate) parameters: Vec<ComponetProperty>,
    pub(crate) flecs_id: EntityId,
    pub(crate) script_id: InstanceId,
    pub(crate) layout: Layout,
} impl ComponetDefinition {
    pub const PROPERTY_PREFIX:&'static str = "_VAR_";

    pub(crate) fn new(
        mut component: Gd<Script>,
        world: &mut _BaseGEWorld,
    ) -> Self {
        // Assemble component properties
        let mut component_properties = Vec::default();
        let mut offset = 0;
        for (key, value) in component.get_script_constant_map().iter_shared() {
            let key = key.to::<GString>();
            let mut key_string = key.to_string();

            if !(key_string.starts_with(Self::PROPERTY_PREFIX)) {
                // Key is not a component variable definition
                continue
            }
            if key.len() == Self::PROPERTY_PREFIX.len() {
                // Key does not contain a variable name
                continue
            }            

            let property_name = StringName::from(key_string.split_off(
                Self::PROPERTY_PREFIX.len()
            ));
            let mut property_type = value.get_type();
            if property_type == VariantType::Nil {
                property_type = VariantType::Object;
            }

            component_properties.push(
                ComponetProperty {
                    name: property_name,
                    gd_type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type as usize];
        }

        // Assemble definition
        let name = component.to_string();
        let layout = _BaseGEWorld::layout_from_properties(&component_properties);
        let comp_id = world.world
            .component_dynamic(name.clone(), layout);
        let component_def = Self {
            name: name.into(),
            parameters: component_properties,
            flecs_id: comp_id,
            script_id: component.instance_id(),
            layout,
        };

        // Settup hooks
        _BaseGEComponent::set_hooks_in_component(
            world,
            comp_id,
        );

        component_def
    }

    pub(crate) fn get_property(
        &self,
        name:&StringName,
    ) -> Option<&ComponetProperty> {
        for p in self.parameters.iter() {
            if &p.name == name {
                return Some(&p)
            }
        }
        None
    }

    pub(crate) fn get_property_default_value(&self, property: &str) -> Variant {
        let mut script = Gd::<Script>::from_instance_id(self.script_id);
        script.get_script_constant_map()
            .get(format!("{}{}", Self::PROPERTY_PREFIX, property))
            .unwrap()
    }

}

/// The definition for one property in a component's definition.
#[derive(Debug, Clone)]
pub(crate) struct ComponetProperty {
    pub(crate) name: StringName,
    pub(crate) gd_type_id: VariantType,
    pub(crate) offset: usize,
} impl Default for ComponetProperty {
    fn default() -> Self {
        Self { 
            name: Default::default(),
            gd_type_id: VariantType::Nil,
            offset: Default::default(),
        }
    }
}

