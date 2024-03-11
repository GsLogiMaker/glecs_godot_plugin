
use core::panic;
use std::alloc::Layout;
use std::collections::HashMap;
use std::mem::size_of;
use std::rc::Rc;

use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

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

        let name = component.to_string();
        let layout = _BaseGEWorld::layout_from_properties(&component_properties);
        let mut script_component = Self {
            name: name.clone().into(),
            parameters: component_properties,
            flecs_id: 0,
            script_id: component.instance_id(),
            layout,
        };
        script_component.flecs_id = world.world
            .component_dynamic(name, layout);
        
        script_component
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

#[derive(Eq, PartialEq, Hash)]
pub(crate) enum ComponentDefinitionsMapKey {
    Name(StringName),
    FlecsId(EntityId),
    ScriptId(InstanceId),
} impl ComponentDefinitionsMapKey {
    pub(crate) fn get_index(&self, d:&ComponentDefinitions) -> usize {
        use ComponentDefinitionsMapKey::Name;
        use ComponentDefinitionsMapKey::FlecsId;
        use ComponentDefinitionsMapKey::ScriptId;
        self.get_index_maybe(d).unwrap_or_else(|| {
            match self {
                Name(k) => { !unimplemented!() },
                FlecsId(k) => { !unimplemented!() },
                ScriptId(k) => {
                    let script:Gd<Script> = Gd::from_instance_id(*k);
                    let msg = format!(
                        "No component has been registered with script \"{}\"",
                        script,
                    );
                    panic!("{msg}");
                },
            }
        })
    }

    pub(crate) fn get_index_maybe(&self, d:&ComponentDefinitions) -> Option<usize> {
        use ComponentDefinitionsMapKey::Name;
        use ComponentDefinitionsMapKey::FlecsId;
        use ComponentDefinitionsMapKey::ScriptId;
        match self {
            Name(k) => d.name_map.get(k).map(|x| *x),
            FlecsId(k) => d.flecs_id_map.get(k).map(|x| *x),
            ScriptId(k) => d.script_id_map.get(k).map(|x| *x),
        }
    }

    pub(crate) fn get_value(&self, d:&ComponentDefinitions) -> Option<Rc<ComponetDefinition>> {
        d.data.get(self.get_index_maybe(d)?).map(|x| {
            x.clone()
        })
    }
} impl From<StringName> for ComponentDefinitionsMapKey {
    fn from(value: StringName) -> Self {
        ComponentDefinitionsMapKey::Name(value)
    }
} impl From<EntityId> for ComponentDefinitionsMapKey {
    fn from(value: EntityId) -> Self {
        ComponentDefinitionsMapKey::FlecsId(value)
    }
} impl From<Gd<Script>> for ComponentDefinitionsMapKey {
    fn from(value: Gd<Script>) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.instance_id())
    }
} impl From<&Gd<Script>> for ComponentDefinitionsMapKey {
    fn from(value: &Gd<Script>) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.instance_id())
    }
} impl From<InstanceId> for ComponentDefinitionsMapKey {
    fn from(value: InstanceId) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value)
    }
} impl From<&StringName> for ComponentDefinitionsMapKey {
    fn from(value: &StringName) -> Self {
        ComponentDefinitionsMapKey::Name(value.clone())
    }
} impl From<&EntityId> for ComponentDefinitionsMapKey {
    fn from(value: &EntityId) -> Self {
        ComponentDefinitionsMapKey::FlecsId(value.clone())
    }
} impl From<&InstanceId> for ComponentDefinitionsMapKey {
    fn from(value: &InstanceId) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.clone())
    }
}

#[derive(Debug, Default)]
pub(crate) struct ComponentDefinitions {
    pub(crate) data: Vec<Rc<ComponetDefinition>>,
    pub(crate) name_map:HashMap<StringName, usize>,
    pub(crate) flecs_id_map:HashMap<EntityId, usize>,
    pub(crate) script_id_map:HashMap<InstanceId, usize>,
    pub(crate) back_map:HashMap<EntityId, Gd<Script>>,
} impl ComponentDefinitions {
    pub(crate) fn add_mapping(
        &mut self,
        index: usize,
        name_map: StringName,
        flecs_id_map: EntityId,
        script_id_map: InstanceId,
    ) {
        self.name_map.insert(name_map, index);
        self.flecs_id_map.insert(flecs_id_map, index);
        self.script_id_map.insert(script_id_map, index);
        self.back_map.insert(flecs_id_map, Gd::from_instance_id(script_id_map));
    }

    pub(crate) fn insert(&mut self, element:ComponetDefinition) -> Rc<ComponetDefinition> {
        let len: usize = self.data.len();
        self.add_mapping(
            len,
            element.name.clone(),
            element.flecs_id,
            element.script_id.clone(),
        );
        let rc = Rc::new(element);
        self.data.push(rc.clone());
        rc
    }

    pub(crate) fn get(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<Rc<ComponetDefinition>> {
        let x = key.into();
        x.get_value(self)
    }

    pub(crate) fn get_script(
        &self,
        key:&EntityId,
    ) -> Option<Gd<Script>> {
        self.back_map.get(key).map(|x| x.clone())
    }

    pub(crate) fn has(&self, map:impl Into<ComponentDefinitionsMapKey>) -> bool{
        map.into().get_index_maybe(self).is_some()
    }
}