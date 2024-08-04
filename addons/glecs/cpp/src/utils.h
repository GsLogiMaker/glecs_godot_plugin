
#ifndef GLECS_UTILS_H
#define GLECS_UTILS_H

#include <flecs.h>
#include <godot_cpp/variant/utility_functions.hpp>

#define ERR(return_value, ...) \
	UtilityFunctions::printerr(__VA_ARGS__); \
	UtilityFunctions::push_error(__VA_ARGS__); \
	return return_value;

template <typename T, typename E>
class Result {

public:
	Result(T value_): value(value_) {
		_is_ok = true;
	}
	Result(E error_): error(error_) {
		_is_ok = false;
	}
	~Result() {
		if (is_ok()) {
			value.~T();
		} else {
			error.~E();
		}
	}

	static Result Ok(T value) {
		return Result(value);
	}
	static Result Err(E error) {
		return Result(error);
	}

	bool is_ok() {
		return _is_ok;
	}
	T unwrap() {
		if (!is_ok()) {
			throw "Tried to convert Error to Ok";
		}
		return value;
	};
	E unwrap_err() {
		if (is_ok()) {
			throw "Tried to convert Ok to Error";
		}
		return error;
	};

private:
	bool _is_ok;
	union {
        T value;
        E error;
    };
};

#define CHECK_VARIANT(VALUE, VARIANT_TYPE) \
		if (value.get_type() != VARIANT_TYPE) { ERR(/**/, \
			"Expected variant value, ", VALUE, ", to be of type ", \
			Variant::get_type_name(VARIANT_TYPE), \
			", but is of type ", Variant::get_type_name(VALUE.get_type()) \
		); } \

namespace godot {
	typedef Result<ecs_entity_t, String> EntityResult;
	typedef Result<int8_t, String> VoidResult;

	class Utils {
	public:
		static VoidResult check_variant_matches(Variant value, Variant::Type type) {
			if (value.get_type() != type) {
				return VoidResult(
					String("Expected variant value \"") + String(value)
					+ String("\" to be of type ") + Variant::get_type_name(type)
					+ String(", but it is of type ")
					+ Variant::get_type_name(value.get_type())
				);
			}
			return VoidResult::Ok(0);
		}

		/// Converts a Variant::Type to an Entity ID
		static EntityResult variant_type_to_id(Variant::Type type);

	};

}

#endif