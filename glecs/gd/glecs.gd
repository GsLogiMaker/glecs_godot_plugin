
class_name Glecs

## An event that's emitted when component is added to an entity. This is
## before the entity gets set.
static var ON_ADD:= _GlecsBindings._flecs_on_add()
## An event that's emitted when component is removed from an entity.
## TODO: Find out when exactly this is emitted and explain it here.
static var ON_REMOVE:= _GlecsBindings._flecs_on_remove()
## An event that's emitted after any property in a component is changed.
static var ON_SET:= _GlecsBindings._flecs_on_set()
static var UN_SET:= _GlecsBindings._flecs_un_set()
static var MONITOR:= _GlecsBindings._flecs_monitor()
static var ON_DELETE:= _GlecsBindings._flecs_on_delete()
static var ON_TABLE_CREATE:= _GlecsBindings._flecs_on_table_create()
static var ON_TABLE_DELETE:= _GlecsBindings._flecs_on_table_delete()
static var ON_TABLE_EMPTY:= _GlecsBindings._flecs_on_table_empty()
static var ON_TABLE_FILL:= _GlecsBindings._flecs_on_table_fill()
## A tag which designates an entity as a prefab.
static var PREFAB:= _GlecsBindings._flecs_prefab()
## A tag for use in pairs to designate inheritance.
static var IS_A:= _GlecsBindings._flecs_is_a()

## An pipeline that's ran during [method Node._process].
static var PROCESS:= 0
## An pipeline that's ran during [method Node._physics_process].
static var PHYSICS_PROCESS:= 0





