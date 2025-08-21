#!/bin/bash

# 기존 프로세스 종료
pkill -f guild-home

echo "🚀 Building..."
cargo build --release

echo "🔵 Starting node 1 on port 8000..."
RUST_LOG=debug ./target/release/guild-home --port 8000 --log debug &
PID1=$!

sleep 2

echo "🟢 Starting node 2 on port 8001..."
RUST_LOG=debug ./target/release/guild-home --port 8001 --bootstrap 127.0.0.1:8000 --log debug &
PID2=$!

echo "⏳ Running for 30 seconds to observe ping/pong..."
sleep 30

echo "🛑 Stopping nodes..."
kill $PID1 $PID2

echo "✅ Test complete"