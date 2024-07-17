
class_name Glecs

## An event that's emitted when component is added to an entity, before the
## component gets set. If you need a similar event that is emit after the
## component is set, use [member ON_INIT] instead.
static var ON_ADD:= _GlecsBindings._flecs_on_add()
## An event that's emitted when component is removed from an entity.
## TODO: Find out when exactly this is emitted and explain it here.
static var ON_REMOVE:= _GlecsBindings._flecs_on_remove()
## An event that's emitted after any property in a component is changed.
static var ON_SET:= 0 # Set by Glecs singleton
static var MONITOR:= _GlecsBindings._flecs_monitor()
static var ON_DELETE:= _GlecsBindings._flecs_on_delete()
static var ON_TABLE_CREATE:= _GlecsBindings._flecs_on_table_create()
static var ON_TABLE_DELETE:= _GlecsBindings._flecs_on_table_delete()
static var ON_TABLE_EMPTY:= _GlecsBindings._flecs_on_table_empty()
static var ON_TABLE_FILL:= _GlecsBindings._flecs_on_table_fill()
## A tag which designates an entity as a prefab. See also [member IS_A].
static var PREFAB:= _GlecsBindings._flecs_prefab()
## A tag for use in pairs to designate that an entity inherits a prefab.
## See also [member PREFAB].
static var IS_A:= _GlecsBindings._flecs_is_a()

## An pipeline that's ran during [method Node._process].
static var PROCESS:= 0 # Set by Glecs singleton
## An pipeline that's ran during [method Node._physics_process].
static var PHYSICS_PROCESS:= 0 # Set by Glecs singleton
## An event that is emitted after a component is added and set to an entity.
## If you need a similar event that is emit before the
## component is set, use [member ON_ADD] instead.
static var ON_INIT:= 0 # Set by Glecs singleton





