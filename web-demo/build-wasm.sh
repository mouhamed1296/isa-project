#!/bin/bash
# Build MA-ISA WASM bindings for web demo

set -e

echo "🔨 Building MA-ISA WASM bindings..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    echo ""
    echo "Please install wasm-pack:"
    echo "  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    echo ""
    echo "Or with cargo:"
    echo "  cargo install wasm-pack"
    exit 1
fi

# Build WASM from isa-ffi
cd ../isa-ffi
echo "📦 Building isa-ffi for wasm32..."
wasm-pack build --target web --out-dir ../web-demo/pkg

cd ../web-demo
echo "✅ WASM build complete!"
echo ""
echo "📁 WASM files generated in: web-demo/pkg/"
echo "🚀 You can now run the web demo:"
echo "   python3 -m http.server 8000"
echo "   Then open: http://localhost:8000"
