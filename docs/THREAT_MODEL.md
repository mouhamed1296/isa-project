# Threat Model

## System Overview

MA-ISA is a cryptographic state accumulator for offline-first systems (POS terminals, IoT devices, etc.). It provides tamper-proof state tracking without requiring network connectivity.

## Assets

### Critical Assets
1. **Master Seed** (32 bytes)
   - Used to derive all axis seeds
   - If compromised, entire security model breaks
   - Must be kept secret

2. **Device State** (96 bytes)
   - Current state across 3 axes (finance, time, hardware)
   - Represents cumulative device history
   - Must maintain integrity

3. **Event Counter** (8 bytes per axis)
   - Tracks number of accumulations
   - Prevents replay attacks
   - Must increment monotonically

### Secondary Assets
1. State files on disk
2. Divergence calculations
3. Merkle tree roots (for batch verification)

## Trust Boundaries

### Trusted
- Device hardware (assumed not physically compromised)
- Rust compiler and standard library
- Cryptographic primitives (BLAKE3, SHA-256)
- Operating system (for file I/O, RNG)

### Untrusted
- Network (if any)
- External inputs (events, entropy)
- State files (could be tampered with)
- Other devices (in multi-device scenarios)

## Threat Actors

### 1. Malicious Operator
**Capability:** Physical access to device, can modify software
**Goal:** Commit fraud without detection
**Attacks:**
- Modify state files directly
- Roll back to previous state
- Skip recording events
- Manipulate timestamps

### 2. External Attacker
**Capability:** Network access, no physical access
**Goal:** Compromise device integrity remotely
**Attacks:**
- Man-in-the-middle during sync
- Replay old network messages
- DOS attacks

### 3. Insider Threat
**Capability:** Source code access, development environment
**Goal:** Insert backdoor or find vulnerability
**Attacks:**
- Code injection
- Dependency poisoning
- Supply chain attack

### 4. Sophisticated Adversary
**Capability:** Advanced tools, cryptanalysis
**Goal:** Break cryptographic guarantees
**Attacks:**
- Collision attacks on hash functions
- Side-channel attacks (timing, power)
- Quantum computing (future threat)

## Attack Scenarios

### Scenario 1: State File Tampering
**Attacker:** Malicious operator with file system access
**Attack Steps:**
1. Stop device
2. Modify state file bytes directly
3. Restart device

**Expected Defense:**
- State integrity check on load
- Divergence detection when syncing
- Formal verification: irreversibility property

**Test:** Modify state file, verify detection

### Scenario 2: Rollback Attack
**Attacker:** Malicious operator
**Attack Steps:**
1. Record legitimate transactions
2. Save state file
3. Perform fraudulent transactions
4. Restore old state file
5. Resume operations

**Expected Defense:**
- Counter increments prevent rollback
- Divergence from expected state detected
- Formal verification: counter increment property

**Test:** Restore old state, verify detection

### Scenario 3: Replay Attack
**Attacker:** Network attacker
**Attack Steps:**
1. Capture legitimate event
2. Replay same event multiple times
3. Attempt to inflate counters

**Expected Defense:**
- Each accumulation changes state uniquely
- Counter prevents duplicate detection
- Formal verification: determinism property

**Test:** Replay same event, verify different states

### Scenario 4: Collision Attack
**Attacker:** Sophisticated adversary
**Attack Steps:**
1. Find two different inputs with same hash
2. Substitute legitimate event with malicious one
3. State appears valid

**Expected Defense:**
- BLAKE3 collision resistance (2^128 security)
- Formal verification: no collisions property
- Multiple axes make collision harder

**Test:** Birthday attack simulation (infeasible)

### Scenario 5: Seed Extraction
**Attacker:** Malicious operator with memory access
**Attack Steps:**
1. Extract master seed from memory/disk
2. Create forged device with same seed
3. Generate arbitrary states

**Expected Defense:**
- **NONE** - This is out of scope
- Mitigation: HSM, secure enclave, key encryption

**Test:** N/A - Known limitation

### Scenario 6: Time Manipulation
**Attacker:** Malicious operator
**Attack Steps:**
1. Set device clock backwards
2. Record events with old timestamps
3. Claim events happened earlier

**Expected Defense:**
- Time axis tracks clock changes
- Divergence detection shows anomaly
- System doesn't prevent (by design for offline)

**Test:** Change clock, verify divergence

### Scenario 7: Partial State Modification
**Attacker:** Malicious operator
**Attack Steps:**
1. Modify only finance axis state
2. Leave time/hardware axes unchanged
3. Hope change goes unnoticed

**Expected Defense:**
- Avalanche effect: small change → large divergence
- All axes contribute to overall divergence
- Formal verification: avalanche property

**Test:** Flip single bit, verify large divergence

### Scenario 8: Denial of Service
**Attacker:** External attacker
**Attack Steps:**
1. Flood device with events
2. Exhaust counter (u64::MAX)
3. Cause overflow/crash

**Expected Defense:**
- Counter wraps correctly at u64::MAX
- Formal verification: counter wrapping property
- 18 quintillion events before wrap

**Test:** Counter at MAX, verify wrap to 0

### Scenario 9: Side-Channel Attack
**Attacker:** Sophisticated adversary with physical proximity
**Attack Steps:**
1. Measure timing of operations
2. Infer secret state from timing variations
3. Extract partial seed information

**Expected Defense:**
- Constant-time comparisons (subtle crate)
- BLAKE3 designed for timing resistance
- **Partial** - Not formally analyzed

**Test:** Timing analysis (requires specialized tools)

### Scenario 10: Supply Chain Attack
**Attacker:** Insider threat
**Attack Steps:**
1. Compromise dependency
2. Inject malicious code
3. Backdoor gets deployed

**Expected Defense:**
- Minimal dependencies (6 direct)
- cargo audit in CI
- Dependency pinning
- **Partial** - Depends on supply chain security

**Test:** Audit dependencies, check for CVEs

## Attack Surface

### 1. Input Validation
**Surface:** Event data, entropy, delta_t parameters
**Risk:** Medium
**Mitigations:**
- No parsing of complex formats
- Fixed-size inputs preferred
- Rust type system prevents buffer overflows

### 2. State Persistence
**Surface:** File I/O, serialization
**Risk:** High
**Mitigations:**
- Integrity checks on load
- Versioned serialization
- Encrypted storage (user responsibility)

### 3. FFI Boundary
**Surface:** C API, WASM bindings
**Risk:** Medium
**Mitigations:**
- Minimal unsafe code
- Clear ownership semantics
- Null pointer checks

### 4. Cryptographic Primitives
**Surface:** BLAKE3, SHA-256 usage
**Risk:** Low (if used correctly)
**Mitigations:**
- Standard library implementations
- Formal verification of usage patterns
- No custom crypto

## Security Requirements

### Must Have
1. ✅ State tampering must be detectable
2. ✅ Rollback attacks must be detectable
3. ✅ Deterministic across platforms
4. ✅ No unsafe code in isa-core
5. ✅ Formal verification of critical properties

### Should Have
1. ⚠️ Timing attack resistance (partially implemented)
2. ⚠️ Side-channel resistance (not formally analyzed)
3. ✅ Minimal dependencies
4. ✅ Clear documentation of limitations

### Nice to Have
1. ❌ Quantum resistance (future work)
2. ❌ Hardware key storage (user responsibility)
3. ❌ Automatic anomaly detection (application layer)

## Out of Scope

### Explicitly NOT Protected
1. **Physical attacks** - Hardware tampering, chip-level attacks
2. **Seed extraction** - If seed is compromised, game over
3. **Legitimate fraud** - System records what it's told
4. **Network security** - TLS, authentication separate
5. **Availability** - DOS protection minimal

### User Responsibilities
1. Secure seed generation (use hardware RNG)
2. Seed protection (HSM, encryption at rest)
3. Access controls on state files
4. Network security if syncing
5. Business logic validation

## Risk Assessment

| Threat | Likelihood | Impact | Risk | Mitigation |
|--------|-----------|--------|------|------------|
| State tampering | High | High | **Critical** | Formal verification, integrity checks |
| Rollback attack | High | High | **Critical** | Counter increment, divergence detection |
| Seed extraction | Medium | Critical | **High** | HSM recommendation, documentation |
| Collision attack | Very Low | High | **Low** | BLAKE3 security, formal verification |
| Side-channel | Low | Medium | **Medium** | Constant-time ops, needs analysis |
| Supply chain | Low | High | **Medium** | Dependency audit, minimal deps |
| Time manipulation | High | Low | **Medium** | Divergence detection, by design |
| DOS | Medium | Low | **Low** | Counter wrapping, resource limits |

## Recommendations for Auditors

### Focus Areas
1. **Cryptographic correctness** - Is BLAKE3/SHA-256 used properly?
2. **Formal verification soundness** - Are Kani proofs complete?
3. **State integrity** - Can state be tampered undetected?
4. **Side-channel resistance** - Timing attack analysis
5. **Integer safety** - Overflow/underflow in critical paths

### Testing Approach
1. Fuzz testing with arbitrary inputs
2. Timing analysis for constant-time violations
3. Symbolic execution for edge cases
4. Differential testing (SIMD vs scalar)
5. Cross-platform determinism verification

### Questions to Answer
1. Can you find any way to tamper with state undetected?
2. Are there collision attacks on the hash construction?
3. Is the formal verification approach sound?
4. Are there timing side-channels?
5. What happens at boundary conditions (MAX values)?

## References

- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [STRIDE Threat Model](https://en.wikipedia.org/wiki/STRIDE_(security))
- [Cryptographic Doom Principle](https://moxie.org/2011/12/13/the-cryptographic-doom-principle.html)
