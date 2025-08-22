#!/bin/bash
# 최소 블록체인 실행 스크립트

echo "🏗️ 빌드 중..."

# workspace 전체 빌드
cargo build --release

echo "✅ 빌드 완료!"
echo ""
echo "🚀 실행 방법:"
echo ""
echo "터미널 1 (Guild-Home):"
echo "  cd guild-home"
echo "  cargo run -- --enable-blockchain"
echo ""
echo "터미널 2 (Minimal Blockchain):"
echo "  cd minimal-blockchain"
echo "  cargo run"
echo ""
echo "또는 release 모드:"
echo "  ./target/release/minimal-blockchain"