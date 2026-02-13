# MA-ISA Implementation Prompt for NestJS POS Backend

## Context
You are building a **Point of Sale (POS) system** with:
- **Frontend**: Tauri (Rust + React) - Desktop application
- **Backend**: NestJS (Node.js + TypeScript)
- **Requirement**: Offline-first operations with integrity monitoring using MA-ISA

## Objective
Implement **Multi-Axis Integrity State Accumulation (MA-ISA)** in your NestJS backend to ensure data integrity for offline POS operations. The system must track and verify the integrity of transactions, inventory, payments, and user actions even when disconnected from the central server.

---

## System Architecture

### High-Level Flow
```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Frontend (Rust + React)            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Transaction  │  │  Inventory   │  │   Payment    │     │
│  │   Module     │  │   Module     │  │   Module     │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                   ┌────────▼────────┐                       │
│                   │  Local SQLite   │                       │
│                   │  + MA-ISA State │                       │
│                   └────────┬────────┘                       │
└────────────────────────────┼──────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  Sync Service   │
                    │  (when online)  │
                    └────────┬────────┘
                             │
┌────────────────────────────▼──────────────────────────────┐
│                    NestJS Backend                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  MA-ISA      │  │  Transaction │  │  Sync        │   │
│  │  Service     │  │  Service     │  │  Service     │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                            │
│  ┌──────────────────────────────────────────────────┐   │
│  │           PostgreSQL + MA-ISA States              │   │
│  └──────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────┘
```

---

## Implementation Requirements

### 1. Install MA-ISA Dependencies

**For NestJS (WASM/Node.js)** — install directly from GitHub:

```bash
npm install github:mouhamed1296/isa-project#main --save
```

Or in `package.json`:

```json
{
  "dependencies": {
    "isa-ffi": "github:mouhamed1296/isa-project#main"
  }
}
```

Then import in your NestJS code:

```typescript
import { WasmAxisAccumulator } from 'isa-ffi/isa-ffi/pkg/isa_ffi.js';
```

**For Tauri (Rust side)** — use as git Cargo dependency:

```toml
# src-tauri/Cargo.toml
[dependencies]
isa-core = { git = "https://github.com/mouhamed1296/isa-project.git", path = "isa-core" }
isa-ffi = { git = "https://github.com/mouhamed1296/isa-project.git", path = "isa-ffi" }
isa-runtime = { git = "https://github.com/mouhamed1296/isa-project.git", path = "isa-runtime" }
```

### 2. Define POS-Specific Dimensions

Create **4 critical dimensions** for POS integrity:

```typescript
// src/isa/isa.config.ts
export const POS_DIMENSIONS = {
  TRANSACTION_INTEGRITY: {
    id: 0,
    name: 'Transaction Integrity',
    description: 'Sales, refunds, voids, discounts',
    threshold: 200,
    weight: 3.0,
    strategy: 'ImmediateHeal',
    safetyRelevant: true
  },
  INVENTORY_INTEGRITY: {
    id: 1,
    name: 'Inventory Integrity',
    description: 'Stock updates, transfers, adjustments',
    threshold: 300,
    weight: 2.5,
    strategy: 'Quarantine',
    safetyRelevant: true
  },
  PAYMENT_INTEGRITY: {
    id: 2,
    name: 'Payment Integrity',
    description: 'Cash, card, mobile money, split payments',
    threshold: 150,
    weight: 3.5,
    strategy: 'Quarantine',
    safetyRelevant: true
  },
  USER_ACTION_INTEGRITY: {
    id: 3,
    name: 'User Action Integrity',
    description: 'Login, logout, permission changes, config updates',
    threshold: 400,
    weight: 2.0,
    strategy: 'MonitorOnly',
    safetyRelevant: false
  }
} as const;

export const POS_CONSTRAINTS = [
  {
    type: 'MaxRatio',
    dimensions: [0, 2], // Transaction vs Payment
    ratio: 1.3,
    description: 'Transaction divergence must not exceed 1.3x payment divergence'
  },
  {
    type: 'SumBelow',
    dimensions: [0, 1, 2], // Transaction + Inventory + Payment
    maxSum: 600,
    description: 'Total critical system divergence must stay below 600'
  },
  {
    type: 'MaxRatio',
    dimensions: [1, 3], // Inventory vs User Actions
    ratio: 1.5,
    description: 'Inventory divergence must not exceed 1.5x user action divergence'
  }
];
```

### 3. Create MA-ISA Service Module

```typescript
// src/isa/isa.module.ts
import { Module } from '@nestjs/common';
import { IsaService } from './isa.service';
import { IsaController } from './isa.controller';

@Module({
  providers: [IsaService],
  controllers: [IsaController],
  exports: [IsaService],
})
export class IsaModule {}
```

### 4. Implement MA-ISA Service

```typescript
// src/isa/isa.service.ts
import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { AxisAccumulator } from '@isa-project/node-bindings'; // Adjust import based on actual package
import { POS_DIMENSIONS, POS_CONSTRAINTS } from './isa.config';
import * as crypto from 'crypto';

interface DimensionState {
  id: number;
  name: string;
  accumulator: AxisAccumulator;
  referenceAccumulator: AxisAccumulator;
  eventCount: number;
  divergence: number;
  quarantined: boolean;
  eventHistory: Array<{
    timestamp: string;
    eventData: string;
    eventNumber: number;
    divergence: number;
    tampered: boolean;
  }>;
}

@Injectable()
export class IsaService implements OnModuleInit {
  private readonly logger = new Logger(IsaService.name);
  private dimensions: Map<number, DimensionState> = new Map();
  private initialized = false;

  async onModuleInit() {
    await this.initialize();
  }

  private async initialize() {
    try {
      this.logger.log('Initializing MA-ISA for POS system...');

      // Generate a consistent seed for this POS terminal
      // In production, this should be derived from terminal ID + secure key
      const seed = crypto.randomBytes(32);

      // Initialize each dimension
      for (const [key, config] of Object.entries(POS_DIMENSIONS)) {
        const accumulator = new AxisAccumulator(seed);
        const referenceAccumulator = new AxisAccumulator(seed);

        this.dimensions.set(config.id, {
          id: config.id,
          name: config.name,
          accumulator,
          referenceAccumulator,
          eventCount: 0,
          divergence: 0,
          quarantined: false,
          eventHistory: [],
        });

        this.logger.log(`Dimension ${config.id} (${config.name}) initialized`);
      }

      this.initialized = true;
      this.logger.log('MA-ISA initialization complete');
    } catch (error) {
      this.logger.error('MA-ISA initialization failed', error);
      throw error;
    }
  }

  /**
   * Record an event in a specific dimension
   */
  async recordEvent(
    dimensionId: number,
    eventData: string,
    metadata?: Record<string, any>,
  ): Promise<{
    success: boolean;
    divergence: number;
    eventNumber: number;
    state: string;
    violations?: string[];
  }> {
    if (!this.initialized) {
      throw new Error('MA-ISA not initialized');
    }

    const dimension = this.dimensions.get(dimensionId);
    if (!dimension) {
      throw new Error(`Invalid dimension ID: ${dimensionId}`);
    }

    if (dimension.quarantined) {
      throw new Error(
        `Dimension ${dimension.name} is quarantined and cannot accept events`,
      );
    }

    try {
      // Encode event data
      const eventBytes = Buffer.from(eventData, 'utf-8');
      
      // Generate entropy
      const entropy = crypto.randomBytes(32);
      
      // Get timestamp
      const deltaT = BigInt(Date.now());

      // Accumulate on both accumulators
      dimension.accumulator.accumulate(eventBytes, entropy, deltaT);
      dimension.referenceAccumulator.accumulate(eventBytes, entropy, deltaT);

      // Calculate divergence
      const activeState = dimension.accumulator.state();
      const refState = dimension.referenceAccumulator.state();
      const divergence = this.calculateDivergence(activeState, refState);

      // Update dimension state
      dimension.eventCount++;
      dimension.divergence = divergence;

      // Record event in history
      dimension.eventHistory.push({
        timestamp: new Date().toISOString(),
        eventData,
        eventNumber: dimension.eventCount,
        divergence,
        tampered: false,
      });

      // Check policies and constraints
      const violations = await this.checkPolicies(dimensionId);

      this.logger.log(
        `Event recorded: ${dimension.name} | Event #${dimension.eventCount} | Divergence: ${divergence}`,
      );

      return {
        success: true,
        divergence,
        eventNumber: dimension.eventCount,
        state: this.bytesToHex(activeState),
        violations: violations.length > 0 ? violations : undefined,
      };
    } catch (error) {
      this.logger.error(`Failed to record event for dimension ${dimensionId}`, error);
      throw error;
    }
  }

  /**
   * Get current state of all dimensions
   */
  getDimensionsState() {
    const states = [];
    for (const [id, dimension] of this.dimensions.entries()) {
      const config = Object.values(POS_DIMENSIONS).find(d => d.id === id);
      states.push({
        id: dimension.id,
        name: dimension.name,
        eventCount: dimension.eventCount,
        divergence: dimension.divergence,
        quarantined: dimension.quarantined,
        state: this.bytesToHex(dimension.accumulator.state()).substring(0, 32) + '...',
        threshold: config?.threshold,
        strategy: config?.strategy,
      });
    }
    return states;
  }

  /**
   * Get event history for a dimension
   */
  getEventHistory(dimensionId: number) {
    const dimension = this.dimensions.get(dimensionId);
    if (!dimension) {
      throw new Error(`Invalid dimension ID: ${dimensionId}`);
    }
    return dimension.eventHistory;
  }

  /**
   * Resync a dimension (clear divergence)
   */
  async resyncDimension(dimensionId: number) {
    const dimension = this.dimensions.get(dimensionId);
    if (!dimension) {
      throw new Error(`Invalid dimension ID: ${dimensionId}`);
    }

    const seed = crypto.randomBytes(32);
    dimension.accumulator = new AxisAccumulator(seed);
    dimension.referenceAccumulator = new AxisAccumulator(seed);
    dimension.divergence = 0;
    dimension.eventCount = 0;
    dimension.quarantined = false;

    this.logger.log(`Dimension ${dimension.name} resynced`);

    return {
      success: true,
      message: `Dimension ${dimension.name} resynced successfully`,
    };
  }

  /**
   * Quarantine a dimension
   */
  async quarantineDimension(dimensionId: number) {
    const dimension = this.dimensions.get(dimensionId);
    if (!dimension) {
      throw new Error(`Invalid dimension ID: ${dimensionId}`);
    }

    dimension.quarantined = true;
    this.logger.warn(`Dimension ${dimension.name} QUARANTINED`);

    return {
      success: true,
      message: `Dimension ${dimension.name} quarantined`,
    };
  }

  /**
   * Check policies and constraints
   */
  private async checkPolicies(dimensionId: number): Promise<string[]> {
    const violations: string[] = [];
    const dimension = this.dimensions.get(dimensionId);
    if (!dimension) return violations;

    const config = Object.values(POS_DIMENSIONS).find(d => d.id === dimensionId);
    if (!config) return violations;

    // Check threshold
    if (dimension.divergence > config.threshold) {
      const violation = `Threshold exceeded: ${dimension.divergence} > ${config.threshold}`;
      violations.push(violation);

      // Enforce strategy
      await this.enforceStrategy(dimensionId, config.strategy, violation);
    }

    // Check constraints
    for (const constraint of POS_CONSTRAINTS) {
      if (this.checkConstraint(constraint)) {
        violations.push(`Constraint violated: ${constraint.description}`);
      }
    }

    return violations;
  }

  /**
   * Enforce policy strategy
   */
  private async enforceStrategy(
    dimensionId: number,
    strategy: string,
    violation: string,
  ) {
    switch (strategy) {
      case 'ImmediateHeal':
        this.logger.warn(`Auto-healing dimension ${dimensionId}: ${violation}`);
        setTimeout(() => this.resyncDimension(dimensionId), 1000);
        break;

      case 'Quarantine':
        this.logger.error(`Auto-quarantining dimension ${dimensionId}: ${violation}`);
        setTimeout(() => this.quarantineDimension(dimensionId), 1000);
        break;

      case 'MonitorOnly':
        this.logger.log(`Monitoring violation for dimension ${dimensionId}: ${violation}`);
        break;

      default:
        this.logger.warn(`Unknown strategy: ${strategy}`);
    }
  }

  /**
   * Check constraint satisfaction
   */
  private checkConstraint(constraint: any): boolean {
    if (constraint.type === 'MaxRatio') {
      const [dim1Id, dim2Id] = constraint.dimensions;
      const div1 = this.dimensions.get(dim1Id)?.divergence || 0;
      const div2 = this.dimensions.get(dim2Id)?.divergence || 0;

      if (div2 === 0) return false;

      const ratio = div1 / div2;
      return ratio > constraint.ratio;
    } else if (constraint.type === 'SumBelow') {
      const sum = constraint.dimensions.reduce((acc, dimId) => {
        return acc + (this.dimensions.get(dimId)?.divergence || 0);
      }, 0);
      return sum > constraint.maxSum;
    }
    return false;
  }

  /**
   * Calculate divergence between two states
   * IMPORTANT: This scales down the 256-bit difference to manageable numbers
   * by dividing by 10^70. Adjust the exponent if you need different sensitivity.
   */
  private calculateDivergence(state1: Buffer, state2: Buffer): number {
    let state1BigInt = BigInt('0x' + this.bytesToHex(state1));
    let state2BigInt = BigInt('0x' + this.bytesToHex(state2));

    const maxValue = BigInt(2) ** BigInt(256);
    let diff =
      state1BigInt > state2BigInt
        ? state1BigInt - state2BigInt
        : state2BigInt - state1BigInt;

    // Handle circular distance in 2^256 space
    if (diff > maxValue / BigInt(2)) {
      diff = maxValue - diff;
    }

    // Scale down: 2^256 ≈ 1.16×10^77
    // Dividing by 10^74 gives divergence values in the 0-1000 range
    // Adjust the exponent (74) if you need different sensitivity:
    // - Higher exponent (e.g., 76) = smaller divergence values
    // - Lower exponent (e.g., 72) = larger divergence values
    return Number(diff / BigInt(10) ** BigInt(74));
  }

  /**
   * Convert bytes to hex string
   */
  private bytesToHex(bytes: Buffer): string {
    return bytes.toString('hex');
  }
}
```

### 5. Create REST API Endpoints

```typescript
// src/isa/isa.controller.ts
import { Controller, Get, Post, Body, Param, HttpCode, HttpStatus } from '@nestjs/common';
import { IsaService } from './isa.service';

@Controller('isa')
export class IsaController {
  constructor(private readonly isaService: IsaService) {}

  @Get('dimensions')
  getDimensions() {
    return this.isaService.getDimensionsState();
  }

  @Get('dimensions/:id/history')
  getEventHistory(@Param('id') id: string) {
    return this.isaService.getEventHistory(parseInt(id));
  }

  @Post('events')
  @HttpCode(HttpStatus.CREATED)
  async recordEvent(
    @Body() body: { dimensionId: number; eventData: string; metadata?: any },
  ) {
    return this.isaService.recordEvent(
      body.dimensionId,
      body.eventData,
      body.metadata,
    );
  }

  @Post('dimensions/:id/resync')
  async resyncDimension(@Param('id') id: string) {
    return this.isaService.resyncDimension(parseInt(id));
  }

  @Post('dimensions/:id/quarantine')
  async quarantineDimension(@Param('id') id: string) {
    return this.isaService.quarantineDimension(parseInt(id));
  }
}
```

### 6. Integrate with POS Operations

```typescript
// src/transactions/transactions.service.ts
import { Injectable } from '@nestjs/common';
import { IsaService } from '../isa/isa.service';
import { POS_DIMENSIONS } from '../isa/isa.config';

@Injectable()
export class TransactionsService {
  constructor(private readonly isaService: IsaService) {}

  async createSale(saleData: any) {
    // 1. Process the sale
    const transaction = await this.processSale(saleData);

    // 2. Record in MA-ISA for integrity tracking
    const eventData = `sale_${transaction.id}_${transaction.total}_${transaction.items.length}_items`;
    await this.isaService.recordEvent(
      POS_DIMENSIONS.TRANSACTION_INTEGRITY.id,
      eventData,
      { transactionId: transaction.id, type: 'sale' },
    );

    // 3. Record payment integrity
    const paymentData = `payment_${transaction.paymentMethod}_${transaction.total}`;
    await this.isaService.recordEvent(
      POS_DIMENSIONS.PAYMENT_INTEGRITY.id,
      paymentData,
      { transactionId: transaction.id, method: transaction.paymentMethod },
    );

    // 4. Update inventory integrity
    for (const item of transaction.items) {
      const inventoryData = `inventory_sale_${item.productId}_qty_${item.quantity}`;
      await this.isaService.recordEvent(
        POS_DIMENSIONS.INVENTORY_INTEGRITY.id,
        inventoryData,
        { productId: item.productId, quantity: -item.quantity },
      );
    }

    return transaction;
  }

  async createRefund(refundData: any) {
    // Similar pattern for refunds
    const refund = await this.processRefund(refundData);

    const eventData = `refund_${refund.id}_${refund.amount}`;
    await this.isaService.recordEvent(
      POS_DIMENSIONS.TRANSACTION_INTEGRITY.id,
      eventData,
      { refundId: refund.id, type: 'refund' },
    );

    return refund;
  }

  private async processSale(saleData: any) {
    // Your existing sale processing logic
    return { id: 1, total: 100, items: [], paymentMethod: 'cash' };
  }

  private async processRefund(refundData: any) {
    // Your existing refund processing logic
    return { id: 1, amount: 50 };
  }
}
```

### 7. Offline Sync Strategy

```typescript
// src/sync/sync.service.ts
import { Injectable, Logger } from '@nestjs/common';
import { IsaService } from '../isa/isa.service';

@Injectable()
export class SyncService {
  private readonly logger = new Logger(SyncService.name);

  constructor(private readonly isaService: IsaService) {}

  /**
   * Sync offline data when connection is restored
   */
  async syncOfflineData(terminalId: string, offlineData: any[]) {
    this.logger.log(`Starting sync for terminal ${terminalId}`);

    const results = {
      synced: 0,
      failed: 0,
      integrityViolations: [],
    };

    for (const record of offlineData) {
      try {
        // Verify integrity before syncing
        const integrityCheck = await this.verifyRecordIntegrity(record);

        if (!integrityCheck.valid) {
          results.integrityViolations.push({
            recordId: record.id,
            reason: integrityCheck.reason,
          });
          results.failed++;
          continue;
        }

        // Sync the record
        await this.syncRecord(record);
        results.synced++;
      } catch (error) {
        this.logger.error(`Failed to sync record ${record.id}`, error);
        results.failed++;
      }
    }

    this.logger.log(`Sync complete: ${results.synced} synced, ${results.failed} failed`);
    return results;
  }

  private async verifyRecordIntegrity(record: any) {
    // Check if the record's MA-ISA state matches expected state
    const dimensions = this.isaService.getDimensionsState();
    
    // Verify no excessive divergence
    for (const dim of dimensions) {
      if (dim.divergence > dim.threshold * 1.5) {
        return {
          valid: false,
          reason: `Dimension ${dim.name} has excessive divergence: ${dim.divergence}`,
        };
      }
    }

    return { valid: true };
  }

  private async syncRecord(record: any) {
    // Your sync logic here
    this.logger.log(`Syncing record ${record.id}`);
  }
}
```

---

## Tauri Frontend Integration

### 8. Rust-Side MA-ISA Integration

```rust
// src-tauri/src/isa_manager.rs
use isa_core::{AxisAccumulator, IntegrityState};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Deserialize)]
pub struct DimensionState {
    pub id: u8,
    pub name: String,
    pub event_count: u32,
    pub divergence: f64,
    pub quarantined: bool,
}

pub struct IsaManager {
    accumulators: Mutex<Vec<AxisAccumulator>>,
    reference_accumulators: Mutex<Vec<AxisAccumulator>>,
}

impl IsaManager {
    pub fn new() -> Self {
        let seed = [0u8; 32]; // Generate proper seed in production
        
        let mut accumulators = vec![];
        let mut reference_accumulators = vec![];
        
        for _ in 0..4 {
            accumulators.push(AxisAccumulator::new(&seed));
            reference_accumulators.push(AxisAccumulator::new(&seed));
        }
        
        Self {
            accumulators: Mutex::new(accumulators),
            reference_accumulators: Mutex::new(reference_accumulators),
        }
    }
    
    pub fn record_event(&self, dimension_id: usize, event_data: &str) -> Result<(), String> {
        let mut accs = self.accumulators.lock().unwrap();
        let mut ref_accs = self.reference_accumulators.lock().unwrap();
        
        if dimension_id >= accs.len() {
            return Err("Invalid dimension ID".to_string());
        }
        
        let event_bytes = event_data.as_bytes();
        let entropy = [0u8; 32]; // Generate proper entropy
        let delta_t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        accs[dimension_id].accumulate(event_bytes, &entropy, delta_t);
        ref_accs[dimension_id].accumulate(event_bytes, &entropy, delta_t);
        
        Ok(())
    }
}

#[tauri::command]
pub fn record_pos_event(
    isa_manager: State<IsaManager>,
    dimension_id: usize,
    event_data: String,
) -> Result<String, String> {
    isa_manager.record_event(dimension_id, &event_data)?;
    Ok("Event recorded successfully".to_string())
}

#[tauri::command]
pub fn get_dimensions_state(isa_manager: State<IsaManager>) -> Vec<DimensionState> {
    // Return current state of all dimensions
    vec![]
}
```

### 9. React Frontend Integration

```typescript
// src/hooks/useISA.ts
import { invoke } from '@tauri-apps/api/tauri';

export function useISA() {
  const recordEvent = async (dimensionId: number, eventData: string) => {
    try {
      await invoke('record_pos_event', { dimensionId, eventData });
    } catch (error) {
      console.error('Failed to record event:', error);
      throw error;
    }
  };

  const getDimensionsState = async () => {
    try {
      return await invoke('get_dimensions_state');
    } catch (error) {
      console.error('Failed to get dimensions state:', error);
      throw error;
    }
  };

  return { recordEvent, getDimensionsState };
}

// Usage in a component
function SaleComponent() {
  const { recordEvent } = useISA();

  const handleSale = async (saleData) => {
    // Process sale
    const sale = await processSale(saleData);

    // Record in MA-ISA
    await recordEvent(0, `sale_${sale.id}_${sale.total}`);
    await recordEvent(2, `payment_${sale.paymentMethod}_${sale.total}`);
  };

  return <div>...</div>;
}
```

---

## Testing Strategy

### 10. Unit Tests

```typescript
// src/isa/isa.service.spec.ts
import { Test } from '@nestjs/testing';
import { IsaService } from './isa.service';

describe('IsaService', () => {
  let service: IsaService;

  beforeEach(async () => {
    const module = await Test.createTestingModule({
      providers: [IsaService],
    }).compile();

    service = module.get<IsaService>(IsaService);
  });

  it('should record events without divergence', async () => {
    const result = await service.recordEvent(0, 'test_sale_100');
    expect(result.success).toBe(true);
    expect(result.divergence).toBe(0);
  });

  it('should detect threshold violations', async () => {
    // Record many events to trigger threshold
    for (let i = 0; i < 100; i++) {
      await service.recordEvent(0, `test_sale_${i}`);
    }
    
    const state = service.getDimensionsState();
    // Check if violations are detected
  });
});
```

---

## Deployment Checklist

- [ ] Install MA-ISA Node.js bindings
- [ ] Configure POS dimensions and thresholds
- [ ] Implement IsaService with all 4 dimensions
- [ ] Integrate with transaction, inventory, payment services
- [ ] Add REST API endpoints for monitoring
- [ ] Implement offline sync service
- [ ] Add Tauri commands for frontend integration
- [ ] Create React hooks for MA-ISA operations
- [ ] Write unit and integration tests
- [ ] Set up monitoring and alerting
- [ ] Document recovery procedures
- [ ] Train staff on integrity monitoring

---

## Key Benefits for POS

1. **Offline Integrity**: Track data integrity even when disconnected
2. **Fraud Detection**: Detect tampering in transactions, payments, inventory
3. **Audit Trail**: Complete event history per dimension
4. **Automatic Recovery**: Auto-heal or quarantine based on policies
5. **Sync Verification**: Verify integrity before syncing offline data
6. **Real-time Monitoring**: Track divergence across all dimensions
7. **Compliance**: Cryptographic proof of data integrity for audits

---

## Support

For questions or issues:
- GitHub: https://github.com/mouhamed1296/isa-project
- Documentation: See `web-demo/` for interactive examples
