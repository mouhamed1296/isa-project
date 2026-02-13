# isa-ffi

C ABI and WASM bindings for MA-ISA.

## C Bindings

### Building

```bash
cargo build --release
```

The shared library will be at `target/release/libisa_ffi.{so,dylib,dll}`.

### Header

Include `include/ma_isa.h` in your C/C++ project.

### Example

```c
#include "ma_isa.h"

uint8_t seed[32] = {0};
isa_runtime_handle_t rt = isa_runtime_new(seed, "./state.bin");

uint8_t sale[] = "sale:100.00";
isa_state_vector_t vec;
isa_record_sale(rt, sale, sizeof(sale), &vec);

isa_save(rt);
isa_runtime_free(rt);
```

## WASM Bindings

### Building

```bash
wasm-pack build --target web
```

### Usage

```javascript
import init, { WasmMultiAxisState } from './pkg/isa_ffi.js';

await init();
const state = new WasmMultiAxisState(new Uint8Array(32));
const vector = state.getStateVector();
```

## Language Support

The C ABI can be consumed by:
- C/C++
- Swift (via bridging header)
- Go (via cgo)
- Python (via ctypes/cffi)
- Java/Kotlin (via JNI)
- .NET (via P/Invoke)

## Safety

All unsafe code is isolated to this crate. Null pointer checks and handle validation are performed on all FFI boundaries.
