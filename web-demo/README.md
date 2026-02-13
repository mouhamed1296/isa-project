# MA-ISA Web Demo - IoT Device Integrity Monitor

**A web application using REAL MA-ISA WASM bindings** to demonstrate multi-dimensional integrity monitoring in an IoT device context.

⚠️ **This uses actual MA-ISA cryptography (BLAKE3-based state accumulation), not simulation.**

## 🎯 Demo Context

This demo simulates a **smart IoT device** (e.g., environmental sensor, smart home device) that monitors its own integrity across four dimensions:

1. **Sensor Integrity** - Temperature, humidity, motion sensor readings
2. **Network Integrity** - Connectivity and communication patterns
3. **Firmware Integrity** - Firmware version and update history
4. **Power Integrity** - Battery level and power consumption

## 🚀 Quick Start

### Step 1: Build WASM Bindings

```bash
cd web-demo

# Option A: Use build script
./build-wasm.sh

# Option B: Manual build
cd ../isa-ffi
wasm-pack build --target web --out-dir ../web-demo/pkg
cd ../web-demo
```

**Prerequisites:**
- Rust toolchain (install from https://rustup.rs)
- wasm-pack: `cargo install wasm-pack`

### Step 2: Run Local Server

```bash
# IMPORTANT: Must use a local server (WASM requires HTTP/HTTPS)
python3 -m http.server 8000

# Then open: http://localhost:8000
```

⚠️ **Cannot open index.html directly** - WASM modules require HTTP/HTTPS protocol.

## 📋 Features

### ✅ NORMATIVE Features (Required)
- **Multi-dimensional state tracking** - 4 independent integrity dimensions
- **Configuration-driven policies** - Thresholds and strategies from `config.yaml`
- **Divergence monitoring** - Real-time circular distance calculation
- **Event accumulation** - Irreversible state transitions

### 🔧 OPTIONAL Features (Demonstrated)
- **Cross-dimension constraints** - MaxRatio and SumBelow constraints
- **Recovery strategies** - MonitorOnly, ImmediateHeal, Quarantine

### 🎮 Interactive Controls
- **Record Sensor Reading** - Simulate temperature/humidity sensor events
- **Record Network Event** - Simulate network communication
- **Simulate Firmware Update** - Simulate firmware version changes
- **Record Battery Event** - Simulate battery level changes
- **Reset State** - Reset all dimensions to initial state

## 📊 What You'll See

1. **Dimension Cards** - Real-time state and divergence for each dimension
2. **Status Indicators** - Green (OK), Yellow (Warning), Red (Error)
3. **Event Log** - Chronological log of all integrity events
4. **Configuration Display** - YAML configuration with multi-language examples

## 🔐 How It Works

### Real State Accumulation (BLAKE3-based)

```javascript
// This demo uses REAL MA-ISA WASM bindings:
import init, { WasmAxisAccumulator } from './pkg/isa_ffi.js';

await init();

// Create real BLAKE3-based accumulator
const seed = new Uint8Array(32);
crypto.getRandomValues(seed);
const accumulator = new WasmAxisAccumulator(seed);

// Real irreversible state accumulation
accumulator.accumulate(eventBytes, entropy, deltaT);

// Get cryptographic state (32 bytes from BLAKE3)
const state = accumulator.state();
```

### Configuration Loading

The demo loads configuration from `config.yaml`:

```yaml
dimensions:
  - id: 0
    name: "Sensor Integrity"
    threshold: 1000
    strategy: MonitorOnly
```

In production, you would:
- Load YAML/JSON/TOML in your language
- Set environment variables for sensitive values
- Pass configuration to MA-ISA runtime

## 🌍 Multi-Language Usage

This demo shows MA-ISA WASM integration in JavaScript. The same WASM module works in:

### JavaScript/TypeScript (This Demo)
```javascript
import init, { WasmAxisAccumulator } from './pkg/isa_ffi.js';
await init();
const accumulator = new WasmAxisAccumulator(seed);
accumulator.accumulate(event, entropy, deltaT);
```

### Node.js
```javascript
const { WasmAxisAccumulator } = require('./pkg/isa_ffi.js');
// Same API as browser
```

### Python
```python
import yaml
with open('config.yaml') as f:
    config = yaml.safe_load(f)
# Use config['dimensions'][0]['threshold']
```

### Go
```go
import "gopkg.in/yaml.v3"
config := Config{}
yaml.Unmarshal(data, &config)
```

### Environment Variables
```bash
export ISA_DIM0_THRESHOLD=1000
export ISA_DIM1_STRATEGY=ImmediateHeal
# MA-ISA runtime reads these automatically
```

## 📁 Files

```
web-demo/
├── index.html       # Frontend UI with dimension visualization
├── app.js           # JavaScript integration with WASM
├── config.yaml      # Configuration file (NORMATIVE)
├── build-wasm.sh    # Build script for WASM bindings
├── pkg/             # WASM build output (gitignored)
│   ├── isa_ffi.js
│   ├── isa_ffi_bg.wasm
│   └── ...
└── README.md        # This file
```

## ✅ What's Real vs Simulated

### ✅ REAL (Using isa-core WASM)
- **BLAKE3 state accumulation** - Actual cryptographic hashing
- **Irreversible state transitions** - Cannot reverse or forge
- **Circular distance calculation** - Real 2^256 modular arithmetic
- **Event counters** - Tracked by MA-ISA accumulator
- **32-byte cryptographic states** - From BLAKE3 KDF

### 📊 Demo-Specific (Not from MA-ISA)
- **UI visualization** - Dimension cards and event log
- **Configuration loading** - YAML parsing (would use isa-runtime in production)
- **Threshold checking** - JavaScript logic (would use isa-runtime policies)
- **Recovery strategies** - Not implemented (optional feature)

## 🎓 Learning Points

This demo illustrates:

1. **Real Cryptography** - Actual BLAKE3-based state accumulation
2. **WASM Integration** - Rust cryptography in JavaScript
3. **Domain-Agnostic Design** - Not tied to finance/time/hardware
4. **Configuration-First** - Policies defined in YAML, not code
5. **Irreversible State** - Cannot forge or reverse state transitions
6. **Zero Unsafe Code** - isa-core uses only safe Rust
7. **Cross-Platform** - Same WASM works in browser and Node.js

## 📚 Documentation

- **[ENHANCEMENTS.md](../ENHANCEMENTS.md)** - Full feature documentation
- **[CONFORMANCE.md](../CONFORMANCE.md)** - ISO/IEC conformance spec
- **[config-examples/README.md](../config-examples/README.md)** - Configuration guide
- **[README.md](../README.md)** - Main project documentation

## 🤝 Extending This Demo

Ideas for enhancement:

- **Add WebSocket** - Real-time updates from actual IoT devices
- **Integrate WASM** - Use actual isa-core instead of simulation
- **Add Charts** - Visualize divergence trends over time
- **Add Alerts** - Trigger notifications on threshold violations
- **Add Export** - Export state and event log to JSON

## 📄 License

This demo is part of the MA-ISA project.

Licensed under MIT OR Apache-2.0 (same as MA-ISA).

---

**MA-ISA v0.1.0** | Domain-Agnostic Integrity State Accumulator  
ISO/IEC Conformance Draft | 58/58 Tests Passing
