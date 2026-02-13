use anyhow::Result;
use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};

const REPO_URL: &str = "https://github.com/mouhamed1296/isa-project.git";

pub fn run(target: Option<String>, output: Option<String>) -> Result<()> {
    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Install Guide".white().bold());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    let target_str = if let Some(t) = target {
        t
    } else {
        let targets = vec!["nestjs", "tauri", "rust", "wasm"];
        let descriptions = vec![
            "NestJS — TypeScript backend with WASM bindings",
            "Tauri — Rust backend + React frontend",
            "Rust — Native Rust library integration",
            "WASM — Browser-based WebAssembly",
        ];
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("  Select your target platform")
            .items(&descriptions)
            .default(0)
            .interact()?;
        targets[idx].to_string()
    };

    let guide = match target_str.as_str() {
        "nestjs" => generate_nestjs_guide(),
        "tauri" => generate_tauri_guide(),
        "rust" => generate_rust_guide(),
        "wasm" => generate_wasm_guide(),
        _ => {
            anyhow::bail!("Unknown target '{}'. Use: nestjs, tauri, rust, wasm", target_str);
        }
    };

    if let Some(out_path) = output {
        // Strip ANSI colors for file output
        let plain = strip_ansi(&guide);
        std::fs::write(&out_path, plain)?;
        println!("  {} Install guide written to {}", "✓".green().bold(), out_path.yellow());
    } else {
        println!("{}", guide);
    }

    Ok(())
}

fn generate_nestjs_guide() -> String {
    let mut s = String::new();

    s.push_str(&format!("  {}\n\n", "NestJS Integration".yellow().bold()));

    s.push_str(&format!("  {} Install the WASM package from GitHub:\n\n", "Step 1:".cyan().bold()));
    s.push_str(&format!("    {}\n\n", "npm install github:mouhamed1296/isa-project#main --save".white()));

    s.push_str(&format!("  {} Import in your service:\n\n", "Step 2:".cyan().bold()));
    s.push_str(&format!("    {}\n", "// src/isa/isa.service.ts".dimmed()));
    s.push_str(&format!("    {}\n\n", "const { WasmAxisAccumulator } = require('isa-project');".white()));

    s.push_str(&format!("  {} Create the ISA module:\n\n", "Step 3:".cyan().bold()));
    s.push_str(&format!("    {}\n", "nest generate module isa".white()));
    s.push_str(&format!("    {}\n\n", "nest generate service isa".white()));

    s.push_str(&format!("  {} Create accumulators per dimension:\n\n", "Step 4:".cyan().bold()));
    s.push_str(&format!("    {}\n", "// For each dimension, create a WasmAxisAccumulator".dimmed()));
    s.push_str(&format!("    {}\n", "const seed = crypto.randomBytes(32);".white()));
    s.push_str(&format!("    {}\n", "const accumulator = new WasmAxisAccumulator(seed);".white()));
    s.push_str(&format!("    {}\n\n", "const refAccumulator = new WasmAxisAccumulator(seed);".white()));

    s.push_str(&format!("  {} Record events on transactions:\n\n", "Step 5:".cyan().bold()));
    s.push_str(&format!("    {}\n", "// In your transaction service".dimmed()));
    s.push_str(&format!("    {}\n", "const eventBytes = new TextEncoder().encode(`sale_${id}_${amount}`);".white()));
    s.push_str(&format!("    {}\n", "const entropy = crypto.randomBytes(16);".white()));
    s.push_str(&format!("    {}\n\n", "accumulator.accumulate(eventBytes, entropy, BigInt(Date.now()));".white()));

    s.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    s.push_str(&format!("  {} See docs/nestjs-pos-implementation-prompt.md for full guide\n", "ℹ".blue()));
    s.push_str(&format!("  {} {}\n\n", "Repo:".dimmed(), REPO_URL.cyan()));

    s
}

fn generate_tauri_guide() -> String {
    let mut s = String::new();

    s.push_str(&format!("  {}\n\n", "Tauri Integration".yellow().bold()));

    s.push_str(&format!("  {} Add Cargo dependencies in src-tauri/Cargo.toml:\n\n", "Step 1:".cyan().bold()));
    s.push_str(&format!("    {}\n", "[dependencies]".dimmed()));
    s.push_str(&format!("    {}\n", format!("isa-core = {{ git = \"{}\", path = \"isa-core\" }}", REPO_URL).white()));
    s.push_str(&format!("    {}\n", format!("isa-ffi = {{ git = \"{}\", path = \"isa-ffi\" }}", REPO_URL).white()));
    s.push_str(&format!("    {}\n\n", format!("isa-runtime = {{ git = \"{}\", path = \"isa-runtime\" }}", REPO_URL).white()));

    s.push_str(&format!("  {} Create Tauri commands in src-tauri/src/isa.rs:\n\n", "Step 2:".cyan().bold()));
    s.push_str(&format!("    {}\n", "use isa_core::AxisAccumulator;".white()));
    s.push_str(&format!("    {}\n\n", "#[tauri::command]".white()));
    s.push_str(&format!("    {}\n", "fn isa_record_event(dimension: usize, data: String) -> Result<String, String> { ... }".white()));
    s.push_str(&format!("    {}\n\n", "fn isa_get_divergence(dimension: usize) -> Result<u64, String> { ... }".white()));

    s.push_str(&format!("  {} Register commands in main.rs:\n\n", "Step 3:".cyan().bold()));
    s.push_str(&format!("    {}\n\n", ".invoke_handler(tauri::generate_handler![isa_record_event, isa_get_divergence])".white()));

    s.push_str(&format!("  {} Call from React frontend:\n\n", "Step 4:".cyan().bold()));
    s.push_str(&format!("    {}\n", "import { invoke } from '@tauri-apps/api/core';".white()));
    s.push_str(&format!("    {}\n\n", "const result = await invoke('isa_record_event', { dimension: 0, data: 'sale_123' });".white()));

    s.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    s.push_str(&format!("  {} {}\n\n", "Repo:".dimmed(), REPO_URL.cyan()));

    s
}

fn generate_rust_guide() -> String {
    let mut s = String::new();

    s.push_str(&format!("  {}\n\n", "Rust Integration".yellow().bold()));

    s.push_str(&format!("  {} Add to Cargo.toml:\n\n", "Step 1:".cyan().bold()));
    s.push_str(&format!("    {}\n", "[dependencies]".dimmed()));
    s.push_str(&format!("    {}\n", format!("isa-core = {{ git = \"{}\", path = \"isa-core\", features = [\"serde\"] }}", REPO_URL).white()));
    s.push_str(&format!("    {}\n\n", format!("isa-runtime = {{ git = \"{}\", path = \"isa-runtime\" }}", REPO_URL).white()));

    s.push_str(&format!("  {} Basic usage:\n\n", "Step 2:".cyan().bold()));
    s.push_str(&format!("    {}\n", "use isa_core::AxisAccumulator;".white()));
    s.push_str(&format!("    {}\n\n", "let mut acc = AxisAccumulator::new(seed);".white()));
    s.push_str(&format!("    {}\n", "acc.accumulate(event_bytes, entropy, delta_t);".white()));
    s.push_str(&format!("    {}\n\n", "let state = acc.state(); // [u8; 32]".white()));

    s.push_str(&format!("  {} With runtime (persistence + entropy):\n\n", "Step 3:".cyan().bold()));
    s.push_str(&format!("    {}\n", "use isa_runtime::{DeviceRuntime, FilePersistence, EventAxis};".white()));
    s.push_str(&format!("    {}\n", "let persistence = FilePersistence::new(\"device.state\");".white()));
    s.push_str(&format!("    {}\n", "let mut runtime = DeviceRuntime::load_or_create(seed, persistence)?;".white()));
    s.push_str(&format!("    {}\n\n", "runtime.record_event(EventAxis::Finance, b\"sale_data\")?;".white()));

    s.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    s.push_str(&format!("  {} {}\n\n", "Repo:".dimmed(), REPO_URL.cyan()));

    s
}

fn generate_wasm_guide() -> String {
    let mut s = String::new();

    s.push_str(&format!("  {}\n\n", "WASM (Browser) Integration".yellow().bold()));

    s.push_str(&format!("  {} Install from GitHub:\n\n", "Step 1:".cyan().bold()));
    s.push_str(&format!("    {}\n\n", "npm install github:mouhamed1296/isa-project#main --save".white()));

    s.push_str(&format!("  {} Import the WASM module:\n\n", "Step 2:".cyan().bold()));
    s.push_str(&format!("    {}\n", "import init, { WasmAxisAccumulator } from 'isa-project';".white()));
    s.push_str(&format!("    {}\n\n", "await init(); // Initialize WASM".white()));

    s.push_str(&format!("  {} Create accumulators:\n\n", "Step 3:".cyan().bold()));
    s.push_str(&format!("    {}\n", "const seed = crypto.getRandomValues(new Uint8Array(32));".white()));
    s.push_str(&format!("    {}\n\n", "const accumulator = new WasmAxisAccumulator(seed);".white()));

    s.push_str(&format!("  {} Accumulate events:\n\n", "Step 4:".cyan().bold()));
    s.push_str(&format!("    {}\n", "const eventBytes = new TextEncoder().encode('my_event_data');".white()));
    s.push_str(&format!("    {}\n", "const entropy = crypto.getRandomValues(new Uint8Array(16));".white()));
    s.push_str(&format!("    {}\n\n", "accumulator.accumulate(eventBytes, entropy, BigInt(Date.now()));".white()));

    s.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    s.push_str(&format!("  {} See web-demo/ for complete working examples\n", "ℹ".blue()));
    s.push_str(&format!("  {} {}\n\n", "Repo:".dimmed(), REPO_URL.cyan()));

    s
}

fn strip_ansi(s: &str) -> String {
    let re = regex_lite(s);
    re
}

// Simple ANSI stripping without regex dependency
fn regex_lite(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm'
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
