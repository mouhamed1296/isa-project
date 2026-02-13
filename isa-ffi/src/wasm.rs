use wasm_bindgen::prelude::*;
use isa_core::{AxisAccumulator, IntegrityState, MultiAxisStateExt};

#[wasm_bindgen]
pub struct WasmAxisAccumulator {
    inner: AxisAccumulator,
}

#[wasm_bindgen]
impl WasmAxisAccumulator {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8]) -> Result<WasmAxisAccumulator, JsValue> {
        if seed.len() != 32 {
            return Err(JsValue::from_str("Seed must be exactly 32 bytes"));
        }

        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(seed);

        Ok(WasmAxisAccumulator {
            inner: AxisAccumulator::new(seed_array),
        })
    }

    pub fn accumulate(&mut self, event: &[u8], entropy: &[u8], delta_t: u64) {
        self.inner.accumulate(event, entropy, delta_t);
    }

    pub fn state(&self) -> Vec<u8> {
        self.inner.state().to_vec()
    }

    pub fn counter(&self) -> u64 {
        self.inner.counter()
    }
}

#[wasm_bindgen]
pub struct WasmMultiAxisState {
    inner: IntegrityState<3>,
}

#[wasm_bindgen]
impl WasmMultiAxisState {
    #[wasm_bindgen(constructor)]
    pub fn new(master_seed: &[u8]) -> Result<WasmMultiAxisState, JsValue> {
        if master_seed.len() != 32 {
            return Err(JsValue::from_str("Master seed must be exactly 32 bytes"));
        }

        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(master_seed);

        Ok(WasmMultiAxisState {
            inner: IntegrityState::from_master_seed(seed_array),
        })
    }

    // Domain-agnostic dimension accessors
    #[wasm_bindgen(js_name = getDimensionState)]
    pub fn get_dimension_state(&self, index: usize) -> Result<Vec<u8>, JsValue> {
        if index >= 3 {
            return Err(JsValue::from_str("Dimension index out of bounds (0-2)"));
        }
        self.inner.dimension(index)
            .map(|dim| dim.state().to_vec())
            .ok_or_else(|| JsValue::from_str("Invalid dimension index"))
    }

    #[wasm_bindgen(js_name = getDimensionCount)]
    pub fn get_dimension_count(&self) -> usize {
        3
    }

    // Legacy compatibility methods (deprecated - use getDimensionState instead)
    #[wasm_bindgen(js_name = getFinanceState)]
    pub fn get_finance_state(&self) -> Vec<u8> {
        self.inner.finance().state().to_vec()
    }

    #[wasm_bindgen(js_name = getTimeState)]
    pub fn get_time_state(&self) -> Vec<u8> {
        self.inner.time().state().to_vec()
    }

    #[wasm_bindgen(js_name = getHardwareState)]
    pub fn get_hardware_state(&self) -> Vec<u8> {
        self.inner.hardware().state().to_vec()
    }

    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .to_bytes()
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<WasmMultiAxisState, JsValue> {
        IntegrityState::from_bytes(bytes)
            .map(|inner| WasmMultiAxisState { inner })
            .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {}", e)))
    }
}

#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    format!(
        "{}.{}.{}",
        isa_core::VERSION_MAJOR,
        isa_core::VERSION_MINOR,
        isa_core::VERSION_PATCH
    )
}
