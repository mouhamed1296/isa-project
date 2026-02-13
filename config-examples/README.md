# MA-ISA Configuration Examples

This directory contains example configuration files that can be used from **any programming language** without modifying Rust code.

## Supported Formats

- **YAML** (`policies.yaml`) - Most human-readable
- **JSON** (`policies.json`) - Universal support
- **TOML** (`policies.toml`) - Rust-friendly
- **Environment Variables** (`.env`) - Container-friendly

## Usage from Different Languages

### Python

```python
import yaml
import json
import subprocess

# Option 1: Load YAML and pass to MA-ISA via FFI
with open('policies.yaml') as f:
    config = yaml.safe_load(f)

# Option 2: Use environment variables
import os
os.environ['ISA_DIM0_THRESHOLD'] = '1000'
os.environ['ISA_DIM0_STRATEGY'] = 'ImmediateHeal'

# Option 3: Call MA-ISA CLI with config file
subprocess.run(['isa-cli', '--config', 'policies.yaml', 'verify'])
```

### JavaScript/Node.js

```javascript
const yaml = require('js-yaml');
const fs = require('fs');

// Option 1: Load YAML
const config = yaml.load(fs.readFileSync('policies.yaml', 'utf8'));

// Option 2: Load JSON
const config = JSON.parse(fs.readFileSync('policies.json', 'utf8'));

// Option 3: Use environment variables
process.env.ISA_DIM0_THRESHOLD = '1000';
process.env.ISA_DIM0_STRATEGY = 'ImmediateHeal';

// Option 4: Call MA-ISA via FFI/WASM
const isa = require('./isa-wasm');
isa.configure(config);
```

### Go

```go
package main

import (
    "encoding/json"
    "gopkg.in/yaml.v3"
    "io/ioutil"
    "os"
)

// Option 1: Load YAML
data, _ := ioutil.ReadFile("policies.yaml")
var config map[string]interface{}
yaml.Unmarshal(data, &config)

// Option 2: Load JSON
data, _ := ioutil.ReadFile("policies.json")
json.Unmarshal(data, &config)

// Option 3: Use environment variables
os.Setenv("ISA_DIM0_THRESHOLD", "1000")
os.Setenv("ISA_DIM0_STRATEGY", "ImmediateHeal")
```

### Java

```java
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;

// Option 1: Load YAML
ObjectMapper yamlMapper = new ObjectMapper(new YAMLFactory());
Map<String, Object> config = yamlMapper.readValue(
    new File("policies.yaml"), 
    Map.class
);

// Option 2: Load JSON
ObjectMapper jsonMapper = new ObjectMapper();
Map<String, Object> config = jsonMapper.readValue(
    new File("policies.json"),
    Map.class
);

// Option 3: Use environment variables
System.setProperty("ISA_DIM0_THRESHOLD", "1000");
```

### C#/.NET

```csharp
using YamlDotNet.Serialization;
using Newtonsoft.Json;

// Option 1: Load YAML
var yaml = File.ReadAllText("policies.yaml");
var deserializer = new DeserializerBuilder().Build();
var config = deserializer.Deserialize<Dictionary<string, object>>(yaml);

// Option 2: Load JSON
var json = File.ReadAllText("policies.json");
var config = JsonConvert.DeserializeObject<Dictionary<string, object>>(json);

// Option 3: Use environment variables
Environment.SetEnvironmentVariable("ISA_DIM0_THRESHOLD", "1000");
```

## Configuration via FFI

When using MA-ISA through FFI (C API, WASM, etc.), you have two options:

### Option A: Pass Configuration as JSON String

```c
// C/C++ example
const char* config_json = read_file("policies.json");
isa_configure_from_json(state, config_json);
```

### Option B: Use Environment Variables

```bash
# Set environment variables before starting your application
export ISA_DIM0_NAME="Financial Transactions"
export ISA_DIM0_THRESHOLD=1000
export ISA_DIM0_STRATEGY=ImmediateHeal
export ISA_DIM0_CRITICAL=true

# Then start your application
./your-app
```

## Configuration via CLI

The `isa-cli` tool can read configuration files directly:

```bash
# Verify state with YAML config
isa-cli --config policies.yaml verify state.bin

# Record event with JSON config
isa-cli --config policies.json record --event "transaction" --value 100

# Show state with TOML config
isa-cli --config policies.toml show state.bin
```

## Docker/Kubernetes Deployment

### Using ConfigMaps (Kubernetes)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: isa-config
data:
  policies.yaml: |
    global:
      learning_rate: 0.1
      min_observations: 10
    dimensions:
      - index: 0
        name: "Financial Transactions"
        threshold: 1000
        strategy: "ImmediateHeal"
        critical: true
```

Mount as volume:
```yaml
volumes:
  - name: config
    configMap:
      name: isa-config
volumeMounts:
  - name: config
    mountPath: /etc/isa
```

### Using Environment Variables (Docker)

```dockerfile
# Dockerfile
FROM your-app
ENV ISA_DIM0_THRESHOLD=1000
ENV ISA_DIM0_STRATEGY=ImmediateHeal
ENV ISA_LEARNING_RATE=0.1
```

Or via docker-compose:
```yaml
services:
  app:
    environment:
      - ISA_DIM0_THRESHOLD=1000
      - ISA_DIM0_STRATEGY=ImmediateHeal
      - ISA_LEARNING_RATE=0.1
```

## Configuration Fields Reference

### Global Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `learning_rate` | float | 0.1 | Adaptive learning rate (0.0-1.0) |
| `min_observations` | int | 10 | Min observations before adapting |
| `master_seed` | string | null | Hex-encoded master seed (optional) |

### Dimension Settings

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `index` | int | Yes | Dimension index (0-based) |
| `name` | string | Yes | Human-readable name |
| `threshold` | int | Yes | Divergence threshold |
| `strategy` | string | Yes | Recovery strategy (see below) |
| `critical` | bool | No | Is this dimension critical? |
| `weight` | float | No | Importance weight (0.0-1.0) |
| `enabled` | bool | No | Is dimension enabled? |

### Recovery Strategies

- `ImmediateHeal` - Apply convergence immediately
- `MonitorOnly` - Log but don't intervene
- `Quarantine` - Disable until manual intervention
- `FullRecovery` - Trigger system-wide recovery
- `Custom:N` - Use custom strategy N

### Constraint Types

- `MaxRatio` - Dimension A ≤ ratio × Dimension B
- `SumBelow` - Sum of dimensions < threshold
- `ConditionalCheck` - If A exceeds, check B
- `Correlation` - Maintain statistical correlation
- `Custom:N` - Use custom constraint N

## Hot Reload

To enable configuration hot-reload without restarting:

```python
import watchdog
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class ConfigReloader(FileSystemEventHandler):
    def on_modified(self, event):
        if event.src_path.endswith('policies.yaml'):
            # Reload configuration
            with open('policies.yaml') as f:
                new_config = yaml.safe_load(f)
            isa.reconfigure(new_config)

observer = Observer()
observer.schedule(ConfigReloader(), path='.')
observer.start()
```

## Best Practices

1. **Version control your configs** - Track changes in git
2. **Use environment-specific files** - `policies.dev.yaml`, `policies.prod.yaml`
3. **Validate before deployment** - Use schema validation
4. **Document your dimensions** - Add comments explaining each dimension
5. **Test configuration changes** - In staging before production
6. **Use secrets management** - For master seeds (never commit to git)
7. **Monitor configuration drift** - Alert on unexpected changes

## Schema Validation

You can validate configuration files before deployment:

```bash
# Using JSON Schema
jsonschema -i policies.json isa-config-schema.json

# Using yamllint
yamllint policies.yaml
```

## Examples by Use Case

- **Financial System**: See `policies.yaml` (transaction monitoring)
- **IoT Devices**: See `examples/iot-config.yaml` (sensor data)
- **Supply Chain**: See `examples/supply-chain-config.yaml` (logistics)
- **Healthcare**: See `examples/healthcare-config.yaml` (patient data)

## Support

For questions or issues with configuration:
- Check the main documentation: `ENHANCEMENTS.md`
- See examples: `isa-runtime/examples/`
- Open an issue on GitHub
