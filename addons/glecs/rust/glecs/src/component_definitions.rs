
use std::alloc::Layout;

use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsBaseComponent;
use crate::world::_GlecsBaseWorld;
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
    pub(crate) fn new(
        component: Gd<Script>,
        world: &mut _GlecsBaseWorld,
    ) -> Self {
        
        // Assemble component properties
        let members_map = Callable::from_object_method(
            &component,
            "_get_members",
        ).callv(Array::default()).to::<Dictionary>();
        let mut component_properties = Vec::default();
        let mut offset = 0;
        for (key, value) in members_map.iter_shared() {
            let mut property_type = value.get_type();
            if property_type == VariantType::Nil {
                property_type = VariantType::Object;
            }

            let name = match key.get_type() {
                VariantType::String => StringName::from(key.to::<String>()),
                VariantType::StringName => key.to::<StringName>(),
                _ => panic!(
                    "Expected component member name to be a String or StringName, but got \"{}\"",
                    key,
                ),
            };

            component_properties.push(
                ComponetProperty {
                    name,
                    gd_type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type as usize];
        }

        // Assemble definition
        let property_count = component_properties.len();
        let name = component.to_string();
        let layout = _GlecsBaseWorld::layout_from_properties(&component_properties);
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
        if property_count != 0 {
            _GlecsBaseComponent::set_hooks_in_component(
                world,
                comp_id,
            );
        }

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

    pub(crate) fn get_property_default_value(
        &self,
        property: Variant,
    ) -> Variant {
        let script = Gd::<Script>::from_instance_id(self.script_id);
        let members_map = Callable::from_object_method(
            &script,
            "_get_members",
        ).callv(Array::default()).to::<Dictionary>();
        
        members_map.get(property).unwrap()
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

