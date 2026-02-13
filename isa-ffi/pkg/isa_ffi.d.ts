/* tslint:disable */
/* eslint-disable */

export class WasmAxisAccumulator {
    free(): void;
    [Symbol.dispose](): void;
    accumulate(event: Uint8Array, entropy: Uint8Array, delta_t: bigint): void;
    counter(): bigint;
    constructor(seed: Uint8Array);
    state(): Uint8Array;
}

export class WasmMultiAxisState {
    free(): void;
    [Symbol.dispose](): void;
    static fromBytes(bytes: Uint8Array): WasmMultiAxisState;
    getDimensionCount(): number;
    getDimensionState(index: number): Uint8Array;
    getFinanceState(): Uint8Array;
    getHardwareState(): Uint8Array;
    getTimeState(): Uint8Array;
    constructor(master_seed: Uint8Array);
    toBytes(): Uint8Array;
}

export function getVersion(): string;
