
use std::alloc::Layout;
use std::ffi::c_char;
use std::ffi::c_void;
use std::ffi::CString;
use std::ffi::CStr;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;

use flecs::*;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsBaseComponent;
use crate::component_definitions::GdComponentData;
use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponentProperty;
use crate::queries::_GlecsQuery;
use crate::show_error;
use crate::world::_GlecsBaseWorld;
use crate::Float;
use crate::Int;
use crate::TYPE_SIZES;

#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _GlecsBindings {
	pub(crate) base: Base<Object>,
}
#[godot_api]
impl _GlecsBindings {
    #[func]
    pub fn initialize_glecs_entities(world: Gd<_GlecsBaseWorld>) {
        _GlecsComponents::_define_raw(
            &world.bind(),
            size_of::<GdComponentData>() as i32,
            &CString::new("ComponentProperties").unwrap(),
        );
    }

    #[func]
    pub(crate) fn emit_event(world: Gd<_GlecsBaseWorld>, event:EntityId, to_entity:EntityId, components:PackedInt64Array) {
        let world_raw = world.bind().raw();
        let mut event_desc = ecs_event_desc_t {
            event: event,
            ids: &ecs_type_t {
                array: (&mut (components[0] as EntityId)) as *mut EntityId,
                count: components.len() as i32,
            },
            entity: to_entity,
            ..Default::default()
        };
        unsafe { ecs_emit(world_raw, &mut event_desc) };
    }

    #[func]
    pub(crate) fn new_id(world: Gd<_GlecsBaseWorld>) -> EntityId {
        Self::new_id_from_ref(&world.bind())
    }

    #[func]
    pub(crate) fn module_init(
        world: Gd<_GlecsBaseWorld>,
        name: GString,
        source_id: EntityId,
    ) -> EntityId {
        Self::module_init_from_ref(&world.bind(), name, source_id)
    }

    #[func]
    pub(crate) fn get_name(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
    ) -> GString {
        Self::get_name_from_ref(&world.bind(), entity)
    }

    #[func]
    pub(crate) fn set_name(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        name: GString,
    ) -> EntityId {
        Self::set_name_from_ref(&world.bind(), entity, name)
    }

    #[func]
    pub(crate) fn pair(
        first: EntityId,
        second: EntityId,
    ) -> EntityId {
        unsafe { ecs_make_pair(first, second) }
    }

    #[func]
    pub(crate) fn pair_first(
        pair: EntityId,
    ) -> EntityId {
        ((pair & ECS_COMPONENT_MASK) >> 32) as u32 as EntityId
    }

    #[func]
    pub(crate) fn pair_second(
        pair: EntityId,
    ) -> EntityId {
        pair as u32 as EntityId
    }

    #[func]
    pub(crate) fn id_is_alive(
        world: Gd<_GlecsBaseWorld>,
        id: EntityId,
    ) -> bool {
        if !world.is_instance_valid() {
            // World is deleted
            return false
        }

        if Self::id_is_pair(id) {
            let first_id = Self::pair_first(id);
            let second_id = Self::pair_second(id);
            let first_alive = unsafe {ecs_is_alive(world.bind().raw(), first_id)};
            let second_alive = unsafe {ecs_is_alive(world.bind().raw(), second_id)};

            return first_alive && second_alive;
        }

        unsafe { ecs_is_alive(world.bind().raw(), id) }
    }

    #[func]
    pub(crate) fn id_is_pair(
        entity: EntityId,
    ) -> bool {
        unsafe { ecs_id_is_pair(entity) }
    }

    #[func]
    pub(crate) fn has_id(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        id: EntityId,
    ) -> bool {
        unsafe { ecs_has_id(world.bind().raw(), entity, id) }
    }

    #[func]
    pub(crate) fn _add_id(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        id: EntityId,
    ) {
        Self::add_id_from_ref(&world.bind(), entity, id);
    }

    #[func]
    pub(crate) fn lookup(
        world: Gd<_GlecsBaseWorld>,
        name: GString,
    ) -> EntityId {
        Self::lookup_from_ref(&world.bind(), name)
    }

    #[func]
    pub(crate) fn lookup_child(
        world: Gd<_GlecsBaseWorld>,
        parent: EntityId,
        name: GString,
    ) -> EntityId {
        Self::lookup_child_from_ref(&world.bind(), parent, name)
    }

    #[func]
    pub(crate) fn add_pair(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        relation: EntityId,
        target: EntityId,
    ) {
        Self::add_pair_from_ref(&world.bind(), entity, relation, target);
    }

	#[func]
    pub(crate) fn _flecs_on_add() -> EntityId {
        unsafe { flecs::EcsOnAdd }
    }
    #[func]
    pub(crate) fn _flecs_on_remove() -> EntityId {
        unsafe { flecs::EcsOnRemove }
    }
    #[func]
    pub(crate) fn _flecs_on_set() -> EntityId {
        unsafe { flecs::EcsOnSet }
    }
    #[func]
    pub(crate) fn _flecs_monitor() -> EntityId {
        unsafe { flecs::EcsMonitor }
    }
    #[func]
    pub(crate) fn _flecs_on_delete() -> EntityId {
        unsafe { flecs::EcsOnDelete }
    }
    #[func]
    pub(crate) fn _flecs_on_table_create() -> EntityId {
        unsafe { flecs::EcsOnTableCreate }
    }
    #[func]
    pub(crate) fn _flecs_on_table_delete() -> EntityId {
        unsafe { flecs::EcsOnTableDelete }
    }
    #[func]
    pub(crate) fn _flecs_on_table_empty() -> EntityId {
        unsafe { flecs::EcsOnTableEmpty }
    }
    #[func]
    pub(crate) fn _flecs_on_table_fill() -> EntityId {
        unsafe { flecs::EcsOnTableFill }
    }
    #[func]
    pub(crate) fn _flecs_prefab() -> EntityId {
        unsafe { flecs::EcsPrefab }
    }
    #[func]
    pub(crate) fn _flecs_child_of() -> EntityId {
        unsafe { flecs::EcsChildOf }
    }
    #[func]
    pub(crate) fn _flecs_is_a() -> EntityId {
        unsafe { flecs::EcsIsA }
    }

    #[func]
    pub fn id_component() -> EntityId {
        unsafe { FLECS_IDEcsComponentID_ }
    }
    #[func]
    pub fn id_pred_eq() -> EntityId {
        unsafe { EcsPredEq }
    }
    #[func]
    pub fn id_is_name() -> EntityId {
        EcsIsName
    }

    pub(crate) fn new_id_from_ref(world: &_GlecsBaseWorld) -> EntityId {
        unsafe { flecs::ecs_new(world.raw()) }
    }

    pub(crate) fn module_init_from_ref(
        world: &_GlecsBaseWorld,
        name: GString,
        source_id: EntityId,
    ) -> EntityId {
        let mut desc = flecs::ecs_component_desc_t::default();
        desc.entity = source_id;
        unsafe { ecs_module_init(
            world.raw(),
            gstring_to_cstring(name).as_ptr(),
            &desc,
        ) }
    }

    pub(crate) fn get_name_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
    ) -> GString {
        GString::from(
            Self::get_name_cstr_from_ref(world, entity)
                .to_owned()
                .into_string()
                .unwrap()
        )
    }

    pub(crate) fn get_name_cstr_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
    ) -> &CStr {
        let name_ptr = unsafe { flecs::ecs_get_name(
            world.raw(),
            entity,
        ) };
        if name_ptr == std::ptr::null() {
            return cstr::cstr!(b"");
        }
        let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
        
        name_cstr
    }
    
    pub(crate) fn set_name_c(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        name: CString,
    ) -> EntityId {
        unsafe { flecs::ecs_set_name(
            world.raw(),
            entity,
            name.as_ptr(),
        ) }
    }
    
    pub(crate) fn set_name_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        name: GString,
    ) -> EntityId {
        Self::set_name_c(world, entity, gstring_to_cstring(name))
    }

    pub(crate) fn add_id_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        id: EntityId,
    ) {
        unsafe { flecs::ecs_add_id(
            world.raw(),
            entity,
            id,
        ) };
    }

    pub(crate) fn lookup_from_ref(
        world: &_GlecsBaseWorld,
        name: GString,
    ) -> EntityId {
        let path = gstring_to_cstring(name);
        Self::lookup_c(world, path.as_ptr())
    }

    pub(crate) fn lookup_c(
        world: &_GlecsBaseWorld,
        name: *const c_char,
    ) -> EntityId {
        let path = name;
        let sep = CString::new("/").unwrap();
        let prefix = CString::new("").unwrap();
        let got = unsafe {
            flecs::ecs_lookup_path_w_sep(
                world.raw(),
                0,
                path,
                sep.as_ptr(),
                prefix.as_ptr(),
                false,
            )
        };
        
        got
    }

    pub(crate) fn lookup_c_recursive(
        world: &_GlecsBaseWorld,
        name: &CStr,
    ) -> EntityId {
        let sep = CString::new("/").unwrap();
        let prefix = CString::new("").unwrap();
        let got = unsafe {
            flecs::ecs_lookup_path_w_sep(
                world.raw(),
                0,
                name.as_ptr(),
                sep.as_ptr(),
                prefix.as_ptr(),
                true,
            )
        };
        
        got
    }

    pub(crate) fn lookup_child_c(
        world: &_GlecsBaseWorld,
        parent: EntityId,
        name: &CStr,
    ) -> EntityId {
        let got = unsafe {
            flecs::ecs_lookup_child(
                world.raw(),
                parent,
                name.as_ptr(),
            )
        };
        
        got
    }


    pub(crate) fn lookup_child_from_ref(
        world: &_GlecsBaseWorld,
        parent: EntityId,
        name: GString,
    ) -> EntityId {
        let path = gstring_to_cstring(name);
        let got = unsafe {
            flecs::ecs_lookup_child(
                world.raw(),
                parent,
                path.as_ptr(),
            )
        };
        
        got
    }

    pub(crate) fn add_pair_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        relation: EntityId,
        target: EntityId,
    ) {
        Self::add_id_from_ref(
            world,
            entity,
            Self::pair(relation, target),
        );
    }
}

fn gstring_to_cstring(text: GString) -> CString {
    unsafe { CString::from_vec_unchecked(Vec::from(text.to_string())) }
}

#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _GlecsComponents {
	pub(crate) base: Base<Object>,
}
#[godot_api]
impl _GlecsComponents {
    #[func]
    pub fn define(
        world: Gd<_GlecsBaseWorld>,
        script: Gd<Script>,
        name: GString,
    ) -> EntityId {
        Self::_define(&world.bind(), script, &gstring_to_cstring(name))
    }

    #[func]
    pub fn define_raw(
        world: Gd<_GlecsBaseWorld>,
        size: Int,
        name: GString,
    ) -> EntityId {
        Self::_define_raw(&world.bind(), size as i32, &gstring_to_cstring(name))
    }

    #[func]
    pub fn emit_on_set(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        component: EntityId,
    ) {
        let on_set_path_ptr = unsafe {
            CString::from_vec_unchecked(Vec::from("Glecs/OnSet"))
        };
        _GlecsBindings::emit_event(
            world.clone(),
            _GlecsBindings::lookup_c(&world.bind(), on_set_path_ptr.as_ptr()),
            entity,
            vec![component as Int].into(),
        );
    }
    
    #[func]
    pub fn id_gd_component_data(w:Gd<_GlecsBaseWorld>) -> EntityId {
        Self::_get_id_component_properties(&w.bind())
    }

    /// Returns a String for identifying an Entity.
    pub(crate) fn debug_identifier(world:*const ecs_world_t, component:EntityId) -> String {
        let name_ptr = unsafe { ecs_get_name(world, component) };
        let name = if name_ptr.is_null() {
            "".into()
        } else {
            unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap()
        };
        format!("{}#{}",
            name,
            component,
        )
    }

    pub(crate) fn _define(
        world: &_GlecsBaseWorld,
        script: Gd<Script>,
        name: &CStr,
    ) -> EntityId {
        let gd_component_data = GdComponentData::from(script);
        let component = Self::_define_raw(world, gd_component_data.size() as i32, name);

        // Add properties component to the new component
        let d = Box::new(gd_component_data);
        let component_properties_id = _GlecsComponents::_get_id_component_properties(world);
        unsafe { ecs_set_id(
            world.raw(),
            component,
            component_properties_id,
            size_of::<GdComponentData>(),
            Box::leak(d) as *mut GdComponentData as *mut c_void,
        ) };

        // Set hooks
        let hooks = ecs_type_hooks_t {
            ctor: Some(HookContext::ctor_hook),
            dtor: Some(HookContext::dtor_hook),
            move_: Some(HookContext::move_hook),
            ctor_move_dtor: Some(HookContext::ctor_move_dtor_hook),
            binding_ctx: HookContext::new(world, component)
                .to_leaked() as *mut c_void,
            binding_ctx_free: Some(HookContext::binding_ctx_free),
            ..Default::default()
        };
        unsafe { ecs_set_hooks_id(world.raw(), component, &hooks) };

        component
    }

    pub(crate) fn _define_raw(
        world: &_GlecsBaseWorld,
        size: i32,
        name: &CStr,
    ) -> EntityId {
        let world_raw = world.raw();
        let desc = ecs_component_desc_t {
            type_: ecs_type_info_t {
                size: size as i32,
                alignment: 8,
                name: name.as_ptr(),
                ..Default::default()
            },
            ..Default::default()
        };
        let component = unsafe { ecs_component_init(world_raw, &desc )};

        component
    }

    /// Returns a reference to a component's Godot data
    pub(crate) fn _get_gd_component_data<'a>(
        world:*const ecs_world_t,
        component:EntityId,
    ) -> Result<&'a GdComponentData, String> {
        let name = CString::new(GdComponentData::name()).unwrap().as_ptr();
        let id = unsafe { ecs_lookup_child(
            world,
            component,
            name,
        ) };
        if id == 0 {
            return Err(format!(
                "Component {} is not mapped to Godot.",
                _GlecsComponents::debug_identifier(world, component)
            ));
        }
        
        Ok(unsafe { (ecs_get_id(
            world,
            component,
            id,
        )  as *const GdComponentData).as_ref() }.unwrap())
    }

    /// Returns the component ID of GdCoomponentData
    pub(crate) fn _get_id_component_properties(world:&_GlecsBaseWorld) -> EntityId{
        _GlecsBindings::lookup_c_recursive(
            world,
            &CString::new(GdComponentData::name()).unwrap()
        )
    }

}


pub(crate) struct HookContext {
    component_id: EntityId,
    world_raw: *mut ecs_world_t,
} impl HookContext {
    pub(crate) fn new(world: &_GlecsBaseWorld, component_id: EntityId) -> Self {
        let world_raw = world.raw();
        Self {
            world_raw,
            component_id
        }
    }

    fn init_component(
        comp_data: NonNull<u8>,
        properties: &GdComponentData,
    ) {
        for p in properties {
            let initial_value = p.default_value();
            Self::init_component_property(comp_data, initial_value, p.offset, p.gd_type_id);
        }
    }

    /// Sets `data` to a Variant without calling data's destructor.
    pub(crate) fn init_component_property(
        data: NonNull<u8>,
        value: Variant,
        offset: usize,
        variant_type: VariantType,
    ) {
        let data = unsafe { NonNull::new_unchecked(data.as_ptr().add(offset)) };
        match variant_type {
            VariantType::NIL => panic!("Can't init \"Nil\" type in component"),
            VariantType::BOOL => Self::init_property::<bool>(data, value, &|| bool::default().to_variant()),
            VariantType::INT => Self::init_property::<Int>(data, value, &|| Int::default().to_variant()),
            VariantType::FLOAT => Self::init_property::<Float>(data, value, &|| Float::default().to_variant()),
            VariantType::STRING => Self::init_property::<GString>(data, value, &|| GString::default().to_variant()),
            VariantType::VECTOR2 => Self::init_property::<Vector2>(data, value, &|| Vector2::default().to_variant()),
            VariantType::VECTOR2I => Self::init_property::<Vector2i>(data, value, &|| Vector2i::default().to_variant()),
            VariantType::RECT2 => Self::init_property::<Rect2>(data, value, &|| Rect2::default().to_variant()),
            VariantType::RECT2I => Self::init_property::<Rect2i>(data, value, &|| Rect2i::default().to_variant()),
            VariantType::VECTOR3 => Self::init_property::<Vector3>(data, value, &|| Vector3::default().to_variant()),
            VariantType::VECTOR3I => Self::init_property::<Vector3i>(data, value, &|| Vector3i::default().to_variant()),
            VariantType::TRANSFORM2D => Self::init_property::<Transform2D>(data, value, &|| Transform2D::default().to_variant()),
            VariantType::VECTOR4 => Self::init_property::<Vector4>(data, value, &|| Vector4::default().to_variant()),
            VariantType::VECTOR4I => Self::init_property::<Vector4i>(data, value, &|| Vector4i::default().to_variant()),
            VariantType::PLANE => Self::init_property::<Plane>(data, value, &|| Plane::invalid().to_variant()),
            VariantType::QUATERNION => Self::init_property::<Quaternion>(data, value, &|| Quaternion::default().to_variant()),
            VariantType::AABB => Self::init_property::<Aabb>(data, value, &|| Aabb::default().to_variant()),
            VariantType::BASIS => Self::init_property::<Basis>(data, value, &|| Basis::default().to_variant()),
            VariantType::TRANSFORM3D => Self::init_property::<Transform3D>(data, value, &|| Transform3D::default().to_variant()),
            VariantType::PROJECTION => Self::init_property::<Projection>(data, value, &|| Projection::default().to_variant()),
            VariantType::COLOR => Self::init_property::<Color>(data, value, &|| Color::default().to_variant()),
            VariantType::STRING_NAME => Self::init_property::<StringName>(data, value, &|| StringName::default().to_variant()),
            VariantType::NODE_PATH => Self::init_property::<NodePath>(data, value, &|| NodePath::default().to_variant()),
            VariantType::RID => Self::init_property::<Rid>(data, value, &|| Rid::new(0).to_variant()),
            VariantType::OBJECT => Self::init_property_variant(data, value),
            VariantType::CALLABLE => Self::init_property::<Callable>(data, value, &|| Callable::invalid().to_variant()),
            VariantType::SIGNAL => Self::init_property::<Signal>(data, value, &|| Signal::invalid().to_variant()),
            VariantType::DICTIONARY => Self::init_property_variant(data, value),
            VariantType::ARRAY => Self::init_property_variant(data, value),
            VariantType::PACKED_BYTE_ARRAY => Self::init_property::<PackedByteArray>(data, value, &|| PackedByteArray::default().to_variant()),
            VariantType::PACKED_INT32_ARRAY => Self::init_property::<PackedInt32Array>(data, value, &|| PackedInt32Array::default().to_variant()),
            VariantType::PACKED_INT64_ARRAY => Self::init_property::<PackedInt64Array>(data, value, &|| PackedInt64Array::default().to_variant()),
            VariantType::PACKED_FLOAT32_ARRAY => Self::init_property::<PackedFloat32Array>(data, value, &|| PackedFloat32Array::default().to_variant()),
            VariantType::PACKED_FLOAT64_ARRAY => Self::init_property::<PackedFloat64Array>(data, value, &|| PackedFloat64Array::default().to_variant()),
            VariantType::PACKED_STRING_ARRAY => Self::init_property::<PackedStringArray>(data, value, &|| PackedStringArray::default().to_variant()),
            VariantType::PACKED_VECTOR2_ARRAY => Self::init_property::<PackedVector2Array>(data, value, &|| PackedVector2Array::default().to_variant()),
            VariantType::PACKED_VECTOR3_ARRAY => Self::init_property::<PackedVector3Array>(data, value, &|| PackedVector3Array::default().to_variant()),
            VariantType::PACKED_COLOR_ARRAY => Self::init_property::<PackedColorArray>(data, value, &|| PackedColorArray::default().to_variant()),
            _ => unreachable!(),
        }
    }
    
    /// Sets `data` to a Variant without calling data's destructor,
    /// narrowed down to a specific type.
    fn init_property<T: FromGodot>(
        data: NonNull<u8>,
        value: Variant,
        default: &dyn Fn() -> Variant,
    ) {
         let default_value = if value != Variant::nil() {
            value
        } else {
            (default)()
        };
        unsafe {
            let param_ptr: *mut u8 = &mut *data.as_ptr();
            let param_slice = std::slice
                ::from_raw_parts_mut(param_ptr, size_of::<T>());
            let value_ptr: *const ManuallyDrop<T> = &ManuallyDrop::new(
                default_value.to::<T>()
            );
            let value_slice = std::slice::from_raw_parts(
                value_ptr.cast::<u8>(),
                size_of::<T>(),
            );
            param_slice.copy_from_slice(value_slice);
        }
    }

    /// Sets `data` to a Variant without calling data's destructor,
    /// not narrowed down to any specific type. Sets `data` to the
    /// Variant pointer.
    fn init_property_variant(
        data: NonNull<u8>,
        value: Variant,
    ) {
        let default_value = if value != Variant::nil() {
            value
        } else {
            Variant::default()
        };
        unsafe {
            let param_ptr:*mut u8 = &mut *data.as_ptr();
            let param_slice = std::slice
                ::from_raw_parts_mut(param_ptr, size_of::<Variant>());
            let value_ptr:*const ManuallyDrop<Variant> = &ManuallyDrop::new(
                default_value
            );
            let value_slice = std::slice::from_raw_parts(
                value_ptr.cast::<u8>(),
                size_of::<Variant>(),
            );
            param_slice.copy_from_slice(value_slice);
        }
    }
    
    /// Drop an entire component, via a pointer.
    pub(crate) fn deinit_component(
        component_ptr: NonNull<u8>,
        properties: &GdComponentData,
    ) {
        for p in properties {
            Self::deinit_component_property(component_ptr, p.offset, p.gd_type_id);
        }
    }

    /// Drops a component's property.
    pub(crate) fn deinit_component_property(
        component_ptr: NonNull<u8>,
        offset: usize,
        variant_type: VariantType,
    ) {
        let data = unsafe { NonNull::new_unchecked(component_ptr.as_ptr().add(offset)) };
        match variant_type {
            VariantType::NIL => panic!("Can't deinit \"Nil\" type in component"),
            VariantType::BOOL => Self::deinit_property::<bool>(data),
            VariantType::INT => Self::deinit_property::<Int>(data),
            VariantType::FLOAT => Self::deinit_property::<Float>(data),
            VariantType::STRING => Self::deinit_property::<GString>(data),
            VariantType::VECTOR2 => Self::deinit_property::<Vector2>(data),
            VariantType::VECTOR2I => Self::deinit_property::<Vector2i>(data),
            VariantType::RECT2 => Self::deinit_property::<Rect2>(data),
            VariantType::RECT2I => Self::deinit_property::<Rect2i>(data),
            VariantType::VECTOR3 => Self::deinit_property::<Vector3>(data),
            VariantType::VECTOR3I => Self::deinit_property::<Vector3i>(data),
            VariantType::TRANSFORM2D => Self::deinit_property::<Transform2D>(data),
            VariantType::VECTOR4 => Self::deinit_property::<Vector4>(data),
            VariantType::VECTOR4I => Self::deinit_property::<Vector4i>(data),
            VariantType::PLANE => Self::deinit_property::<Plane>(data),
            VariantType::QUATERNION => Self::deinit_property::<Quaternion>(data),
            VariantType::AABB => Self::deinit_property::<Aabb>(data),
            VariantType::BASIS => Self::deinit_property::<Basis>(data),
            VariantType::TRANSFORM3D => Self::deinit_property::<Transform3D>(data),
            VariantType::PROJECTION => Self::deinit_property::<Projection>(data),
            VariantType::COLOR => Self::deinit_property::<Color>(data),
            VariantType::STRING_NAME => Self::deinit_property::<StringName>(data),
            VariantType::NODE_PATH => Self::deinit_property::<NodePath>(data),
            VariantType::RID => Self::deinit_property::<Rid>(data),
            VariantType::OBJECT => Self::deinit_property_variant(data),
            VariantType::CALLABLE => Self::deinit_property::<Callable>(data),
            VariantType::SIGNAL => Self::deinit_property::<Signal>(data),
            VariantType::DICTIONARY => Self::deinit_property_variant(data),
            VariantType::ARRAY => Self::deinit_property_variant(data),
            VariantType::PACKED_BYTE_ARRAY => Self::deinit_property::<PackedByteArray>(data),
            VariantType::PACKED_INT32_ARRAY => Self::deinit_property::<PackedInt32Array>(data),
            VariantType::PACKED_INT64_ARRAY => Self::deinit_property::<PackedInt64Array>(data),
            VariantType::PACKED_FLOAT32_ARRAY => Self::deinit_property::<PackedFloat32Array>(data),
            VariantType::PACKED_FLOAT64_ARRAY => Self::deinit_property::<PackedFloat64Array>(data),
            VariantType::PACKED_STRING_ARRAY => Self::deinit_property::<PackedStringArray>(data),
            VariantType::PACKED_VECTOR2_ARRAY => Self::deinit_property::<PackedVector2Array>(data),
            VariantType::PACKED_VECTOR3_ARRAY => Self::deinit_property::<PackedVector3Array>(data),
            VariantType::PACKED_COLOR_ARRAY => Self::deinit_property::<PackedColorArray>(data),
            _ => unreachable!(),
        }
    }

    /// Drops `data`'s specifc Variant type.
    fn deinit_property<T> (
        data: NonNull<u8>,
    ) {
        let property = unsafe {
            data.as_ptr()
                .cast::<ManuallyDrop<T>>()
                .as_mut()
                .unwrap()
        };

        drop(unsafe { ManuallyDrop::take(property) })
    }

    /// Drops `data`'s Variant pointer.
    fn deinit_property_variant(
        data: NonNull<u8>,
    ) {
        let property = unsafe {
            data.as_ptr()
                .cast::<ManuallyDrop<Variant>>()
                .as_mut()
                .unwrap()
        };
        
        drop(unsafe { ManuallyDrop::take(property) })
    }

    fn ref_leaked<'a>(from: *mut c_void) -> &'a mut Self {
        unsafe { (from as *mut Self).as_mut().unwrap() }
    }

    unsafe fn take_leaked(from: *mut c_void) -> Box<Self> {
        unsafe { Box::from_raw(from as *mut Self) }
    }

    pub(crate) fn to_leaked(self) -> *mut Self {
        let ptr:*mut Self = Box::leak(Box::new(self));
        ptr
    }

    extern "C" fn ctor_hook(
        ptr: *mut c_void,
        count: i32,
        type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let component =  unsafe { (*type_info).component };
        let size = unsafe {*type_info}.size as usize;
        let component_properties = _GlecsComponents
            ::_get_gd_component_data(hook_context.world_raw, component)
            .unwrap_or_else(|e| show_error!(
                "Failed to run constructor on component.",
                "{e}",
            ));
    
        for i in 0..count {
            let counted_ptr = unsafe {
                ptr.add(i * size)
            };
    
            // Write sane defaults to data
            let data = unsafe {
                NonNull::new_unchecked(counted_ptr as *mut u8)
            };
            Self::init_component(
                data,
                &component_properties,
            );
        }
    }
    
    extern "C" fn dtor_hook(
        ptr: *mut c_void,
        count: i32,
        type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let component = unsafe {*type_info}.component;
        let size = unsafe {*type_info}.size as usize;
        let component_properties = _GlecsComponents
            ::_get_gd_component_data(hook_context.world_raw, component)
            .unwrap_or_else(|e| show_error!(
                "Failed to run destructor on component.",
                "{e}",
            ));
    
        for i in 0..count {
            let counted_ptr = unsafe {
                ptr.add(i * size)
            };
    
            // Call destructor for each property
            let data = unsafe {
                NonNull::new_unchecked(counted_ptr as *mut u8)
            };
            Self::deinit_component(
                data,
                component_properties,
            );
        }
    }
    
    extern "C" fn move_hook(
        dst_ptr: *mut c_void,
        src_ptr: *mut c_void,
        count: i32,
        type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let component = unsafe {*type_info}.component;
        let size = unsafe {*type_info}.size as usize;
        let component_properties = _GlecsComponents
            ::_get_gd_component_data(hook_context.world_raw, component)
            .unwrap_or_else(|e| show_error!(
                "Failed to move component.",
                "{e}",
            ));
    
        for i in 0..count {
            let src = unsafe {
                std::slice::from_raw_parts_mut(
                    src_ptr.add(i * size)
                        as *mut u8,
                    size,
                )
            };
            let dst = unsafe {
                std::slice::from_raw_parts_mut(
                    dst_ptr.add(i * size)
                        as *mut u8,
                    size,
                )
            };
    
            // Move contents
            dst.copy_from_slice(src);
    
            // TODO: Determin if this is still neccessary
            // Reset src so that the destructor does not attempt to deinit
            // the moved data
            Self::init_component(
                unsafe { NonNull::new_unchecked(src.as_mut_ptr()) },
                &component_properties,
            );
        }
    }
    
    extern "C" fn ctor_move_dtor_hook(
        dst_ptr: *mut c_void,
        src_ptr: *mut c_void,
        count: i32,
        type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let size = unsafe {*type_info}.size as usize;
    
        for i in 0..count {
            let src = unsafe {
                std::slice::from_raw_parts_mut(
                    src_ptr.add(i * size)
                        as *mut u8,
                    size,
                )
            };
            let dst = unsafe {
                std::slice::from_raw_parts_mut(
                    dst_ptr.add(i * size)
                        as *mut u8,
                    size,
                )
            };
    
            // Move contents
            dst.copy_from_slice(src);
        }
    }

    pub(crate) extern "C" fn binding_ctx_free(ctx: *mut c_void) {
        drop(unsafe { Self::take_leaked(ctx ) } )
    }
}

#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _GlecsQueries {
	pub(crate) base: Base<Object>,
}
#[godot_api]
impl _GlecsQueries {
    #[func]
    fn new_query() -> Gd<_GlecsQuery> {
        Gd::from_init_fn(_GlecsQuery::init)
    }

    #[func]
    /// Adds a new term to the query.
    fn push_term(mut query:Gd<_GlecsQuery>, id:EntityId) {
        let mut q = query.bind_mut();
        if q.term_count == q.desc.terms.len() {
            panic!("Can't add more terms. Max term count is {}", q.desc.terms.len())
        }

        q.term_count += 1;
        let i = q.term_count-1;
        q.desc.terms[i].id = id;
    }

    #[func]
    ///Sets the access mode (inout) on the most recent term.
    fn set_term_access_mode(mut query:Gd<_GlecsQuery>, mode:Int) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].inout = mode as i16;
    }

    #[func]
    ///Sets the operation on the most recent term.
    fn set_term_oper(mut query:Gd<_GlecsQuery>, oper:Int) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].oper = oper as i16;
    }

    #[func]
    ///Sets the cache kind for the query.
    fn set_cache_kind(mut query:Gd<_GlecsQuery>, kind:Int) {
        let mut q = query.bind_mut();
        q.desc.cache_kind = kind as u32;
    }

    #[func]
    ///Sets the cache kind for the query.
    fn set_expr(mut query:Gd<_GlecsQuery>, expr:GString) {
        let mut q = query.bind_mut();
        q.desc.expr = gstring_to_cstring(expr).into_raw();
    }

    #[func]
    fn set_term_first_id(mut query:Gd<_GlecsQuery>, id:EntityId) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].first.id = id;
    }

    #[func]
    fn set_term_first_name(mut query:Gd<_GlecsQuery>, name:GString) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].first.name = gstring_to_cstring(name).into_raw();
    }

    #[func]
    fn set_term_second_id(mut query:Gd<_GlecsQuery>, id:EntityId) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].second.id = id;
    }

    #[func]
    fn set_term_second_name(mut query:Gd<_GlecsQuery>, name:GString) {
        let mut q = query.bind_mut();
        let i = q.term_count-1;
        q.desc.terms[i].second.name = gstring_to_cstring(name).into_raw();
    }

    #[func]
    fn iterate(world:Gd<_GlecsBaseWorld>, mut query:Gd<_GlecsQuery>, callable:Callable) -> () {
        let mut query_bind = query.bind_mut();
        let terms = query_bind.desc.terms.split_at(query_bind.term_count).0;
        query_bind.desc.binding_ctx = QueryIterationContext::new(
            callable,
            world.clone(),
            terms,
            Box::from([]),
        ).leak() as *mut c_void;
        query_bind.desc.binding_ctx_free = Some(Self::query_iteration_contex_drop);

        drop(query_bind);
        
        let iter = Self::to_iterable(world, query);
        Self::iterate_fn(iter, &|x|
            Self::query_iteration(x)
        );
    }

    #[func]
    ///Sets the flags for the query.
    fn set_flags(mut query:Gd<_GlecsQuery>, flags:Int) {
        let mut q = query.bind_mut();
        q.desc.cache_kind = flags as u32;
    }

    fn to_iterable(world:Gd<_GlecsBaseWorld>, query:Gd<_GlecsQuery>) -> ecs_iter_t {
        let world_raw = world.bind().raw();
        let query_ref = query.bind();
        let q = unsafe { ecs_query_init(world_raw, &query_ref.desc) };
        let mut iter = unsafe { ecs_query_iter(world_raw, q) };
        iter.ctx = query_ref.desc.ctx;
        iter.binding_ctx = query_ref.desc.binding_ctx;

        iter
    }

    /// Iterates over a query via `func`.
    fn iterate_fn(
        mut iter: ecs_iter_t,
        func:impl Fn(*mut ecs_iter_t),
    ) {
        while unsafe { ecs_query_next(&mut iter) } {
            func(&mut iter);
        }
    }

    extern "C" fn query_iteration(iter_ptr:*mut ecs_iter_t) {
		let context = unsafe { iter_ptr.as_mut()
            .unwrap()
            .get_binding_context_mut::<QueryIterationContext>()
		};

        // Update extra variables
        let mut system_args_ref = context.system_args.clone();
        for (i, getter) in
            context.additional_arg_getters.iter().enumerate()
        {
            system_args_ref.set(i, getter.callv(Array::default()));
        }

        // Cache important values TODO: Move to context
        let world = context.term_accesses
            .first()
            .unwrap()
            .bind()
            .world
            .clone();
        let term_ids = context.term_accesses
            .iter()
            .map(|t| t.bind().component_id)
            .collect::<Vec<_>>();

        let entity_count = unsafe {*iter_ptr}.count as usize;
		for entity_i in 0..entity_count {
            let entity = unsafe { *(*iter_ptr).entities.add(entity_i) };
            let field_count = unsafe {*iter_ptr}.field_count as usize;
            
			// Update cached component arguments
			for field_i in 0..field_count {
                if context.is_term_optional(field_i) {
                    // The term is optional
                    // TODO: create a function dedicated to handling systems with no optional parameters for performance. (benchmark) 
                    // TODO: Record from last iteration if term was absent for performance (benchmark)
                    let has_id = _GlecsBindings::has_id(world.clone(), entity, term_ids[field_i]);
                    context.set_term_absent(field_i, !has_id);
                    if !has_id {
                        continue;
                    }
                }

                let mut term_bind = context
                    .term_accesses[field_i]
                    .bind_mut();
				term_bind.entity_id = entity;
			}
			
			let _result = context.callable.callv(
				context.system_args.clone()
			);
		}
    }

    extern "C" fn query_iteration_contex_drop(context_ptr:*mut c_void) {
        let boxed = unsafe { Box::from_raw(
            context_ptr.cast::<QueryIterationContext>()
        ) };
        drop(boxed)
	}
}

#[derive(Debug, Clone)]
pub(crate) struct QueryIterationContext {
    callable: Callable,
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_GlecsBaseComponent>]>,
    /// A list of getters for extra arguments in a pipeline.
    additional_arg_getters: Box<[Callable]>,
    /// A bitmap of wether a term is optional or not
    optional:u32,
} impl QueryIterationContext {
    fn new(
        callable: Callable,
        world: Gd<_GlecsBaseWorld>,
        terms: &[ecs_term_t],
        additional_arg_getters: Box<[Callable]>,
    ) -> Self {
        // Make arguments list
        let mut args = array![];
        for _v in additional_arg_getters.iter() {
            args.push(Variant::nil());
        }

        // Create component accesses
        let mut tarm_accesses: Vec<Gd<_GlecsBaseComponent>> = vec![];
        let mut optional_map = 0u32;
        let mut last_oper = ecs_oper_kind_t_EcsAnd as ecs_oper_kind_t;
        for (i, term) in terms.iter().enumerate() {
            match term.oper as ecs_oper_kind_t {
                ecs_oper_kind_t_EcsAnd => { /* pass */ },
                ecs_oper_kind_t_EcsOr => {
                    optional_map |= 1 << i;
                },
                ecs_oper_kind_t_EcsNot => { continue },
                ecs_oper_kind_t_EcsOptional => {
                    optional_map |= 1 << i;
                },
                ecs_oper_kind_t_EcsAndFrom => { todo!() },
                ecs_oper_kind_t_EcsOrFrom => { todo!() },
                ecs_oper_kind_t_EcsNotFrom => { todo!() },
                _ => unimplemented!("Operation {} not implemented", term.oper),
            }

            if last_oper == ecs_oper_kind_t_EcsOr {
                optional_map |= 1 << i;
            }

            match term.inout as ecs_inout_kind_t {
                ecs_inout_kind_t_EcsInOutDefault => { todo!() },
                ecs_inout_kind_t_EcsInOutFilter => { /* pass */ },
                ecs_inout_kind_t_EcsInOutNone => { continue },
                ecs_inout_kind_t_EcsInOut => { /* pass */ },
                ecs_inout_kind_t_EcsIn => { todo!() },
                ecs_inout_kind_t_EcsOut => { todo!() },
                _ => unimplemented!("Inout mode {} not implemented", term.inout),
            }

            let mut compopnent_access = Gd::from_init_fn(|base| {
                let base_comp = _GlecsBaseComponent {
                    base,
                    entity_id: 0, // ID should be changed by the iterator
                    component_id: term.id,
                    world: world.clone(),
                };
                base_comp
            });

            if let Ok(gd_component_data) = _GlecsComponents::_get_gd_component_data(
                world.bind().raw(),
                term.id,
            ) {
                // Component has been mapped to Godot. Add script to
                // component access.
                compopnent_access.set_script(
                    gd_component_data.script.to_variant()
                );
            }
            
            // Add term access
            args.push(compopnent_access.to_variant());
            tarm_accesses.push(compopnent_access);

            last_oper = term.oper as ecs_oper_kind_t;
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        Self {
            callable: callable,
            system_args: args,
            term_accesses: term_args_fast,
            additional_arg_getters,
            optional: optional_map,
        }
    }
    
    fn leak(self) -> *mut Self {
        Box::leak(Box::new(self)) as *mut QueryIterationContext
    }

    fn set_term_absent(&mut self, index:usize, absent:bool) {
        if absent {
            self.system_args.set(
                index+self.additional_arg_getters.len(),
                Variant::nil(),
            );
        } else {
            self.system_args.set(
                index+self.additional_arg_getters.len(),
                self.term_accesses[index].to_variant(),
                );
        }
    }
    
    fn is_term_optional(&self, term:usize) -> bool {
        return (self.optional & (1u32 << term)) != 0
    }
}