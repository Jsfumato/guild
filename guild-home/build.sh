#!/bin/bash

# Rust 빌드 스크립트

echo "🔨 Building P2P Program Runtime Platform..."

# 의존성 체크
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 빌드 시작
echo "📦 Building in debug mode..."
cargo build

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 To run the server:"
    echo "   cargo run"
    echo ""
    echo "📝 Environment variables:"
    echo "   NODE_ID=validator-1"
    echo "   RPC_PORT=8899"
    echo "   P2P_PORT=8000"
    echo "   BOOTSTRAP_PEERS=/ip4/192.168.1.100/tcp/8000/p2p/..."
else
    echo "❌ Build failed. Please check the errors above."
    exit 1
fi

# 릴리즈 빌드 옵션
if [ "$1" == "--release" ]; then
    echo ""
    echo "📦 Building in release mode (optimized)..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        echo "✅ Release build successful!"
        echo "Binary location: target/release/p2p-game-server"
    fi
fi

# ARM 크로스 컴파일 옵션 (라즈베리 파이)
if [ "$1" == "--arm" ]; then
    echo ""
    echo "🍓 Building for Raspberry Pi (ARM)..."
    
    # 타겟 추가
    rustup target add armv7-unknown-linux-gnueabihf
    rustup target add aarch64-unknown-linux-gnu
    
    # 32-bit ARM (Pi 3)
    echo "Building for ARMv7..."
    cargo build --release --target=armv7-unknown-linux-gnueabihf
    
    # 64-bit ARM (Pi 4)
    echo "Building for ARM64..."
    cargo build --release --target=aarch64-unknown-linux-gnu
    
    echo "✅ ARM builds complete!"
    echo "Binaries:"
    echo "  32-bit: target/armv7-unknown-linux-gnueabihf/release/p2p-game-server"
    echo "  64-bit: target/aarch64-unknown-linux-gnu/release/p2p-game-server"
fi