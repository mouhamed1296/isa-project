# isa-cli

Command-line tool for MA-ISA state management.

## Installation

```bash
cargo install isa-cli
```

Or build from source:

```bash
cd isa-cli
cargo install --path .
```

## Quick Start

```bash
# Initialize a new device
isa init my-pos-terminal

# Record a sale
isa record-sale --file my-pos-terminal.state --amount 1999 --currency USD

# Verify state integrity
isa verify my-pos-terminal.state

# Show current state
isa show my-pos-terminal.state

# Compare two devices
isa compare device1.state device2.state
```

## Commands

### `isa init`

Initialize a new device state.

```bash
isa init <device-id> [OPTIONS]

Options:
  -o, --output <FILE>    Output file path (default: <device-id>.state)
  -s, --seed <HEX>       Master seed as hex string (generates random if not provided)
```

**Examples:**

```bash
# Generate random seed
isa init pos_terminal_001

# Use specific seed
isa init pos_terminal_001 --seed 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# Custom output path
isa init pos_terminal_001 --output /var/lib/isa/terminal.state
```

### `isa record-sale`

Record a sale transaction.

```bash
isa record-sale [OPTIONS] --amount <CENTS> --currency <CODE>

Options:
  -f, --file <FILE>         State file path (default: device.state)
  -a, --amount <CENTS>      Sale amount in cents
  -c, --currency <CODE>     Currency code (default: USD)
  -m, --metadata <STRING>   Additional metadata
```

**Examples:**

```bash
# Record $19.99 sale
isa record-sale --file terminal.state --amount 1999

# With metadata
isa record-sale --file terminal.state --amount 2500 --currency EUR --metadata "item:coffee"
```

### `isa record`

Record a custom event.

```bash
isa record [OPTIONS] <EVENT>

Options:
  -f, --file <FILE>       State file path (default: device.state)
  -e, --entropy <DATA>    Entropy data (hex or string)
  -d, --delta-t <MS>      Time delta in milliseconds (default: 1000)
```

**Examples:**

```bash
# Record string event
isa record --file terminal.state "user_login:alice"

# Record hex event with custom entropy
isa record --file terminal.state deadbeef --entropy cafebabe --delta-t 5000
```

### `isa verify`

Verify state integrity.

```bash
isa verify <FILE> [OPTIONS]

Options:
  -v, --verbose    Show detailed output
```

**Examples:**

```bash
# Basic verification
isa verify terminal.state

# Verbose output
isa verify terminal.state --verbose
```

### `isa show`

Show current state.

```bash
isa show <FILE> [OPTIONS]

Options:
  -f, --format <FORMAT>    Output format: text, json, hex (default: text)
```

**Examples:**

```bash
# Human-readable output
isa show terminal.state

# JSON output
isa show terminal.state --format json

# Raw hex output
isa show terminal.state --format hex
```

### `isa compare`

Compare two device states.

```bash
isa compare <FILE1> <FILE2> [OPTIONS]

Options:
  -f, --format <FORMAT>    Output format: text, json (default: text)
```

**Examples:**

```bash
# Compare two terminals
isa compare terminal1.state terminal2.state

# JSON output
isa compare terminal1.state terminal2.state --format json
```

## Output Examples

### Init

```
✓ Initialized device: pos_terminal_001
  State file: pos_terminal_001.state
  Seed: a1b2c3d4e5f6...

⚠ Keep the seed secure! It cannot be recovered if lost.
```

### Record Sale

```
✓ Sale recorded
  Amount: 19.99 USD
  State updated:
    Finance:  4a3f2e1d2c3b4a5f
    Time:     9b8c7a6f5e4d3c2b
    Hardware: 2d1e0f3c4b5a6978
```

### Verify

```
✓ State is valid
  File: terminal.state
```

### Show

```
Device State
============================================================

Finance Axis:
  00  4a 3f 2e 1d 2c 3b 4a 5f
  08  6e 7d 8c 9b aa b9 c8 d7
  10  e6 f5 04 13 22 31 40 4f
  18  5e 6d 7c 8b 9a a9 b8 c7

Time Axis:
  00  9b 8c 7a 6f 5e 4d 3c 2b
  ...

Hardware Axis:
  00  2d 1e 0f 3c 4b 5a 69 78
  ...
```

### Compare

```
State Comparison
============================================================

  File 1: terminal1.state
  File 2: terminal2.state

Divergence:
  Finance:  0a1b2c3d4e5f6a7b (magnitude: 245)
  Time:     1122334455667788 (magnitude: 312)
  Hardware: 99aabbccddeeff00 (magnitude: 891)

  Total Magnitude: 1448

! States have diverged significantly
```

## Exit Codes

- `0` - Success
- `1` - Error (invalid arguments, file not found, verification failed, etc.)

## Environment Variables

None currently used.

## Files

- `*.state` - Binary state files (versioned, serialized)
- Default location: Current directory

## Security Notes

1. **Seed Security:** The master seed is displayed during `init`. Store it securely!
2. **File Permissions:** State files should have restricted permissions (600 on Unix)
3. **Backup:** Regular backups of state files recommended
4. **Verification:** Run `isa verify` regularly to ensure integrity

## Troubleshooting

### "Failed to load state"

- Check file exists and is readable
- Verify file is not corrupted
- Ensure file was created with compatible version

### "Invalid hex string"

- Hex strings must be even length
- Only characters 0-9, a-f, A-F allowed
- Seeds must be exactly 64 hex characters (32 bytes)

### "Failed to save state"

- Check write permissions on directory
- Ensure sufficient disk space
- Verify path is valid

## Development

```bash
# Run from source
cargo run -- init test-device

# Run tests
cargo test

# Build release
cargo build --release
```

## License

Dual-licensed under MIT OR Apache-2.0
