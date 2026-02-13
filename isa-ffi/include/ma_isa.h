#ifndef MA_ISA_H
#define MA_ISA_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    ISA_SUCCESS = 0,
    ISA_NULL_POINTER = 1,
    ISA_INVALID_HANDLE = 2,
    ISA_INVALID_STATE = 3,
    ISA_ENTROPY_FAILED = 4,
    ISA_PERSISTENCE_FAILED = 5,
    ISA_TIME_FAILED = 6,
    ISA_BUFFER_TOO_SMALL = 7,
    ISA_UNKNOWN = 255
} isa_error_t;

typedef enum {
    ISA_AXIS_FINANCE = 0,
    ISA_AXIS_TIME = 1,
    ISA_AXIS_HARDWARE = 2
} isa_axis_t;

typedef struct {
    uint8_t finance[32];
    uint8_t time[32];
    uint8_t hardware[32];
} isa_state_vector_t;

typedef size_t isa_runtime_handle_t;

isa_runtime_handle_t isa_runtime_new(
    const uint8_t* master_seed,
    const char* persistence_path
);

isa_runtime_handle_t isa_runtime_load_or_create(
    const uint8_t* master_seed,
    const char* persistence_path
);

isa_error_t isa_runtime_free(isa_runtime_handle_t handle);

isa_error_t isa_record_sale(
    isa_runtime_handle_t handle,
    const uint8_t* sale_data,
    size_t sale_len,
    isa_state_vector_t* out_vector
);

isa_error_t isa_record_event(
    isa_runtime_handle_t handle,
    uint8_t axis,
    const uint8_t* event_data,
    size_t event_len,
    isa_state_vector_t* out_vector
);

isa_error_t isa_save(isa_runtime_handle_t handle);

isa_error_t isa_get_state_vector(
    isa_runtime_handle_t handle,
    isa_state_vector_t* out_vector
);

void* isa_axis_new(const uint8_t* seed);

void isa_axis_free(void* axis_ptr);

isa_error_t isa_axis_accumulate(
    void* axis_ptr,
    const uint8_t* event_data,
    size_t event_len,
    const uint8_t* entropy,
    size_t entropy_len,
    uint64_t delta_t
);

isa_error_t isa_axis_get_state(
    const void* axis_ptr,
    uint8_t* out_state
);

void* isa_state_new(const uint8_t* master_seed);

void isa_state_free(void* state_ptr);

isa_error_t isa_get_version(
    uint16_t* major,
    uint16_t* minor,
    uint16_t* patch
);

#ifdef __cplusplus
}
#endif

#endif
