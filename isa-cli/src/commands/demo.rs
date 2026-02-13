use anyhow::Result;
use colored::*;
use std::process::Command;
use std::path::PathBuf;

pub fn run(port: u16, variant: String) -> Result<()> {
    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Web Demo".white().bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();

    // Find the web-demo directory
    let web_demo_dir = find_web_demo_dir()?;

    let page = match variant.as_str() {
        "iot" => "index.html",
        "government" => "government.html",
        "mobile-money" => "mobile-money.html",
        _ => {
            println!("  {} Unknown variant '{}'. Available: iot, government, mobile-money", 
                "!".yellow(), variant);
            println!("  Defaulting to IoT demo...");
            "index.html"
        }
    };

    println!("  {} {}", "Demo:".dimmed(), variant.cyan());
    println!("  {} {}", "Port:".dimmed(), port.to_string().yellow());
    println!("  {} {}", "URL:".dimmed(), format!("http://localhost:{}/{}", port, page).green().underline());
    println!();
    println!("  {} Make sure you have built the WASM package first:", "ℹ".blue());
    println!("    {}", "cd isa-ffi && wasm-pack build --target web --out-dir ../web-demo/pkg".dimmed());
    println!();
    println!("  {} Starting server... (press Ctrl+C to stop)", "▶".green().bold());
    println!();

    // Try python3 first, then python
    let result = Command::new("python3")
        .args(["-m", "http.server", &port.to_string()])
        .current_dir(&web_demo_dir)
        .status();

    match result {
        Ok(status) => {
            if !status.success() {
                println!("  {} Server exited with status: {}", "!".yellow(), status);
            }
        }
        Err(_) => {
            // Try python as fallback
            let result2 = Command::new("python")
                .args(["-m", "http.server", &port.to_string()])
                .current_dir(&web_demo_dir)
                .status();

            match result2 {
                Ok(status) => {
                    if !status.success() {
                        println!("  {} Server exited with status: {}", "!".yellow(), status);
                    }
                }
                Err(_) => {
                    anyhow::bail!(
                        "Could not start HTTP server. Please install Python or run manually:\n  cd {} && python3 -m http.server {}",
                        web_demo_dir.display(),
                        port
                    );
                }
            }
        }
    }

    Ok(())
}

fn find_web_demo_dir() -> Result<PathBuf> {
    // Check relative to current directory
    let candidates = vec![
        PathBuf::from("web-demo"),
        PathBuf::from("../web-demo"),
        PathBuf::from("../../web-demo"),
    ];

    for candidate in candidates {
        if candidate.exists() && candidate.join("index.html").exists() {
            return Ok(candidate);
        }
    }

    // Check relative to the binary location
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let from_bin = parent.join("../web-demo");
            if from_bin.exists() {
                return Ok(from_bin);
            }
        }
    }

    anyhow::bail!(
        "Could not find web-demo directory. Make sure you're in the isa-project root or a subdirectory."
    );
}
