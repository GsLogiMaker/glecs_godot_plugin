
use godot::engine::Script;
use godot::prelude::*;

use crate::Int;
use crate::TYPE_SIZES;

#[derive(GodotClass, Debug, Clone)]
#[class(base=Object, no_init)]
pub(crate) struct GdComponentData {
    pub(crate) properties: Vec<ComponentProperty>,
    pub(crate) script: Gd<Script>,
}
#[godot_api]
impl GdComponentData {
    #[func]
    pub fn get_component_script(&self) -> Gd<Script> {
        return self.script.clone()
    }

    #[func]
    pub fn size(&self) -> Int {
        let mut size = 0 as usize;
        for p in &self.properties {
            size += TYPE_SIZES[p.gd_type_id.ord() as usize] as usize;
        }
        size as Int
    }

    pub(crate) fn get_property(&self, property:&StringName) -> Option<&ComponentProperty> {
        self.properties.iter()
            .find(|x| &x.name == property)
    }

    pub(crate) fn get_property_by_offset(&self, offset:usize) -> Option<&ComponentProperty> {
        self.properties.iter()
            .find(|x| x.offset == offset)
    }

    pub(crate) fn name() -> String {
        "GdComponentData".into()
    }
} impl<'a> IntoIterator for &'a GdComponentData {
    type Item = &'a ComponentProperty;

    type IntoIter = std::slice::Iter<'a, ComponentProperty>;

    fn into_iter(self) -> Self::IntoIter {
        self.properties.iter()
    }
} impl From<Gd<Script>> for GdComponentData {
    fn from(script: Gd<Script>) -> Self {
        // Make properties list
        let mut properties = Vec::default();
        let members_map = Callable::from_object_method(
            &script,
            "_get_members",
        ).callv(Array::default()).to::<Dictionary>();
        let mut offset = 0;
        for (key, value) in members_map.iter_shared() {
            let mut property_type = value.get_type();
            if property_type == VariantType::NIL {
                property_type = VariantType::OBJECT;
            }

            let name = match key.get_type() {
                VariantType::STRING => StringName::from(key.to::<String>()),
                VariantType::STRING_NAME => key.to::<StringName>(),
                _ => panic!(
                    "Expected component member name to be a String or StringName, but got \"{}\"",
                    key,
                ),
            };

            properties.push(
                ComponentProperty {
                    name,
                    gd_type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type.ord() as usize];
        }        
        
        GdComponentData { properties, script }
    }
}

/// The definition for one property in a component's definition.
#[derive(Debug, Clone)]
pub(crate) struct ComponentProperty {
    pub(crate) name: StringName,
    pub(crate) gd_type_id: VariantType,
    pub(crate) offset: usize,
} impl ComponentProperty {
    pub(crate) fn default_value(&self) -> Variant {
        Variant::nil()
    }
} impl Default for ComponentProperty {
    fn default() -> Self {
        Self { 
            name: Default::default(),
            gd_type_id: VariantType::NIL,
            offset: Default::default(),
        }
    }
}

