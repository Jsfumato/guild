# 🏰 Guild Home - P2P 네트워킹 시스템

순수한 P2P 피어 탐색 및 연결 시스템

## 📦 프로젝트 구조

```
guild-workspace/
├── Cargo.toml           (workspace 설정)
└── guild-home/          (P2P 네트워킹 코어)
    ├── Cargo.toml
    └── src/
        ├── lib.rs       (라이브러리 모듈)
        ├── main.rs      (실행 파일)
        ├── config.rs    (설정 관리)
        ├── guild_home.rs (메인 구조체)
        ├── network.rs   (P2P 네트워크)
        ├── discovery.rs (피어 탐색)
        └── help.rs      (도움말)
```

## 🚀 빠른 시작

### 빌드
```bash
# 프로젝트 빌드
cargo build

# 릴리즈 빌드
cargo build --release
```

### 실행
```bash
# 기본 실행 (자동 포트 할당, 피어 탐색)
./target/release/guild-home

# 특정 포트로 실행
./target/release/guild-home --port 8000

# 부트스트랩 피어와 함께 실행
./target/release/guild-home --bootstrap 192.168.1.10:8000,192.168.1.11:8000
```

## 🔧 기능 설명

### 🏠 Guild Home 코어
- **피어 탐색**: mDNS 기반 자동 피어 발견
- **P2P 연결**: QUIC 프로토콜 기반 안전한 연결
- **하트비트**: 주기적인 피어 상태 확인
- **설정**: 명령줄 인수 및 환경변수 지원

### 📡 네트워킹
- **자동 포트 할당**: 포트 0 지정시 자동 할당
- **부트스트랩**: 초기 피어 목록으로 네트워크 참여
- **피어 상태 모니터링**: 실시간 연결된 피어 수 표시

## ⚙️ 설정 옵션

### 명령줄 인수
```bash
guild-home [OPTIONS]

OPTIONS:
    -p, --port <PORT>             포트 지정 (0 = 자동)
    -b, --bootstrap <PEERS>       부트스트랩 피어 (콤마 구분)
    -d, --data-dir <DIR>          데이터 디렉토리 (기본: ./data)
    -i, --interval <SECONDS>      하트비트 간격 (기본: 5초)
    -l, --log <LEVEL>             로그 레벨 (error/warn/info/debug)
    -h, --help                    도움말 표시
```

### 환경변수
```bash
export GUILD_PORT=8000
export GUILD_BOOTSTRAP=192.168.1.10:8000,192.168.1.11:8000
export GUILD_HEARTBEAT_INTERVAL=10
export GUILD_LOG_LEVEL=debug
```

## 🎯 사용 시나리오

### 시나리오 1: 로컬 테스트
```bash
# 첫 번째 노드 (터미널 1)
./target/release/guild-home --port 8000

# 두 번째 노드 (터미널 2)
./target/release/guild-home --port 8001 --bootstrap 127.0.0.1:8000

# 세 번째 노드 (터미널 3)
./target/release/guild-home --port 8002 --bootstrap 127.0.0.1:8000
```

### 시나리오 2: 분산 네트워크
```bash
# 서버 A (192.168.1.10)
./target/release/guild-home --port 8000

# 서버 B (192.168.1.11)
./target/release/guild-home --port 8000 --bootstrap 192.168.1.10:8000

# 서버 C (192.168.1.12)
./target/release/guild-home --port 8000 --bootstrap 192.168.1.10:8000,192.168.1.11:8000
```

## 🐳 Docker Compose

3노드 P2P 네트워크 테스트:

```bash
# 전체 네트워크 시작
docker-compose up

# 개별 노드 시작
docker-compose up guild-home
docker-compose up guild-node-2
```

컨테이너 간 네트워크에서 자동으로 피어를 찾고 연결합니다.

## 📈 성능 최적화

### 릴리즈 프로파일
```toml
[profile.release]
opt-level = "z"        # 최대 크기 최적화
lto = true            # Link Time Optimization
codegen-units = 1     # 단일 코드 생성 유닛
strip = true          # 디버그 정보 제거
```

### 바이너리 크기
- **guild-home**: ~3MB (QUIC + mDNS만 포함)

## 🔮 확장 가능성

### 현재 기능
- [x] P2P 피어 탐색 및 연결
- [x] QUIC 기반 안전한 통신
- [x] mDNS 자동 발견
- [x] 하트비트 메커니즘

### 향후 확장 가능
- [ ] 메시지 라우팅 시스템
- [ ] 분산 파일 공유
- [ ] 채팅/메시징 서비스
- [ ] 분산 컴퓨팅 플랫폼
- [ ] 블록체인/DLT 레이어

## 🤝 기여하기

```bash
# 프로젝트 클론
git clone https://github.com/guild-home/guild-workspace
cd guild-workspace

# 새 기능 브랜치
git checkout -b feature/amazing-feature

# 테스트
cargo test

# 포맷팅
cargo fmt

# 린팅
cargo clippy -- -D warnings
```

---

**"순수한 P2P, 무한한 연결"** 🚀