# Minimal Blockchain

지갑과 스토리지 없는 최소 블록체인 구현

## 특징

- ✅ **지갑 없음**: 단순 노드 ID만 사용
- ✅ **스토리지 없음**: 메모리에 최근 100개 블록만 유지
- ✅ **네트워크 격리**: Guild-Home을 통해서만 통신
- ✅ **간단한 컨센서스**: PBFT (2/3+1 투표)
- ✅ **1초 블록 생성**: 라운드 로빈 제안자

## 아키텍처

```
외부 네트워크
     ↑
Guild-Home (네트워크 게이트웨이)
     ↑ IPC (TCP localhost:9000)
     ↓
Minimal Blockchain (별도 프로세스)
```

## 빌드

```bash
# workspace 루트에서
cargo build --workspace

# 또는 개별 빌드
cd minimal-blockchain
cargo build --release
```

## 실행

### 방법 1: 개별 실행

```bash
# 터미널 1
cd guild-home
cargo run

# 터미널 2
cd minimal-blockchain
cargo run
```

### 방법 2: 스크립트 사용

```bash
./run_minimal_blockchain.sh
```

## 컨센서스 흐름

1. **블록 제안** (1초마다)
   - 라운드 로빈으로 제안자 선택
   - 새 블록 생성 및 브로드캐스트

2. **투표**
   - 블록 검증
   - 유효한 경우 투표 전송

3. **확정**
   - 2/3+1 투표 수집시 블록 확정
   - 다음 라운드 진행

## 메시지 타입

```rust
enum ConsensusMessage {
    Propose(MinimalBlock),  // 블록 제안
    Vote(BlockVote),       // 투표
    Commit(MinimalBlock),  // 확정
}
```

## 환경 변수

- `IPC_PORT`: Guild-Home과 통신할 포트 (기본: 9000)

## 개발 상태

- [x] 기본 블록 구조
- [x] 간단한 PBFT 컨센서스
- [x] IPC 통신
- [x] Guild-Home 브리지
- [ ] 다중 노드 테스트
- [ ] 성능 최적화