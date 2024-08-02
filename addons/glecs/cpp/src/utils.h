
#ifndef GLECS_UTILS_H
#define GLECS_UTILS_H

#include <godot_cpp/variant/utility_functions.hpp>

#define ERR(return_value, ...) \
	UtilityFunctions::printerr(__VA_ARGS__); \
	UtilityFunctions::push_error(__VA_ARGS__); \
	return return_value;

#endif