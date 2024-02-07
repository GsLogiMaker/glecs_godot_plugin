
use std::alloc::Layout;
use std::collections::HashMap;
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
    pub(crate) parameters: HashMap<StringName, ComponetProperty>,
    pub(crate) flecs_id: EntityId,
    pub(crate) script_id: InstanceId,
    pub(crate) layout: Layout,
} impl ComponetDefinition {
    pub(crate) fn new(
        mut component: Gd<Script>,
        world: &mut _BaseGEWorld,
    ) -> Self {
        let script_properties = component
            .get_script_property_list();

        let mut component_properties = HashMap::default();
        let mut offset = 0;
        let mut i = 0;
        while i != script_properties.len() {
            let property = script_properties.get(i);
            let property_type = property
                .get(StringName::from("type"))
                .unwrap()
                .to::<VariantType>();
            if property_type == VariantType::Nil {
                i += 1;
                continue;
            }
            let property_name:StringName = property
                .get(StringName::from("name"))
                .unwrap()
                .to::<String>()
                .into();

            component_properties.insert(
                property_name.clone(),
                ComponetProperty {
                    name: property_name,
                    gd_type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type as usize];
            i += 1;
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
                    godot_error!("{msg}");
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
        for key in &self.script_id_map {
            godot_print!("ADD NEW COMP {}", key.0);
        }
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

    pub(crate) fn has(&self, map:impl Into<ComponentDefinitionsMapKey>) -> bool{
        map.into().get_index_maybe(self).is_some()
    }
}