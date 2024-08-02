
#ifndef GLECS_UTILS_H
#define GLECS_UTILS_H

#include <godot_cpp/variant/utility_functions.hpp>

#define ERR(return_value, ...) \
	UtilityFunctions::printerr(__VA_ARGS__); \
	UtilityFunctions::push_error(__VA_ARGS__); \
	return return_value;

template <typename T, typename E>
class Result {

public:
	Result(T value_) {
		_is_ok = true;
		value = value_;
	}
	Result(E error_) {
		_is_ok = false;
		error = error_;
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

#endif