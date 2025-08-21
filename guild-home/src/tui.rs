// Guild Home TUI Dashboard - 실시간 P2P 네트워크 모니터링
use crate::network::{Network, NetworkStats, PeerInfo};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, TableState,
    },
    Frame, Terminal,
};
use std::{
    io,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct TuiApp {
    network: Arc<Network>,
    start_time: Instant,
    table_state: TableState,
    last_refresh: Instant,
    // 캐시된 데이터 (주기적으로 업데이트)
    peer_count: usize,
    peers_info: Vec<(SocketAddr, PeerInfo)>,
    network_stats: NetworkStats,
    recent_logs: Vec<String>,
}

impl TuiApp {
    pub fn new(network: Arc<Network>) -> Self {
        Self {
            network,
            start_time: Instant::now(),
            table_state: TableState::default(),
            last_refresh: Instant::now(),
            peer_count: 0,
            peers_info: Vec::new(),
            network_stats: NetworkStats::default(),
            recent_logs: Vec::new(),
        }
    }
    
    // 네트워크 데이터 업데이트
    async fn update_data(&mut self) {
        self.peer_count = self.network.peer_count().await;
        self.peers_info = self.network.get_peers_info().await;
        self.network_stats = self.network.get_stats().await;
        
        // 최근 로그 가져오기
        let logger = guild_logger::get_logger();
        let logs = logger.get_recent_logs().await;
        self.recent_logs = logs.into_iter().take(15).collect();
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        // 초기 데이터 로드
        self.update_data().await;
        
        loop {
            // 매 500ms마다 데이터 업데이트
            if self.last_refresh.elapsed() >= Duration::from_millis(500) {
                self.update_data().await;
                self.last_refresh = Instant::now();
            }
            
            terminal.draw(|f| self.ui(f))?;

            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down => {
                            let peer_count = self.peers_info.len();
                            if peer_count > 0 {
                                self.table_state.select(Some(
                                    self.table_state
                                        .selected()
                                        .map(|i| (i + 1) % peer_count)
                                        .unwrap_or(0),
                                ));
                            }
                        }
                        KeyCode::Up => {
                            let peer_count = self.peers_info.len();
                            if peer_count > 0 {
                                self.table_state.select(Some(
                                    self.table_state
                                        .selected()
                                        .map(|i| if i == 0 { peer_count - 1 } else { i - 1 })
                                        .unwrap_or(0),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(8),     // Peers table
                Constraint::Length(5),  // Network activity
                Constraint::Min(6),     // Logs
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);
        self.render_peers_table(f, chunks[1]);
        self.render_network_activity(f, chunks[2]);
        self.render_logs(f, chunks[3]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let uptime = self.start_time.elapsed();
        let uptime_str = format!(
            "{:02}:{:02}:{:02}",
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        );

        // 실제 네트워크 정보 (캐시된 데이터 사용)
        let port = self.network.local_port();
        let header_text = format!(
            " Port: {} │ Peers: {} │ Uptime: {} ",
            port,
            self.peer_count,
            uptime_str
        );

        let header = Paragraph::new(header_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Guild Home Dashboard")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );

        f.render_widget(header, area);
    }

    fn render_peers_table(&mut self, f: &mut Frame, area: Rect) {
        let header_cells = ["IP Address", "Port", "Latency", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // 실제 peer 데이터 사용
        let rows = self.peers_info.iter().map(|(addr, peer_info)| {
            let ip = addr.ip().to_string();
            let port = addr.port().to_string();
            let latency = format!("{}ms", peer_info.latency_ms);
            let status = "✅ Connected";
            
            let status_style = Style::default().fg(Color::Green);
            
            Row::new(vec![
                Cell::from(ip),
                Cell::from(port),
                Cell::from(latency),
                Cell::from(status).style(status_style),
            ])
        });

        let peer_count = self.peers_info.len();
        let table_title = format!("Connected Peers ({})", peer_count);
        
        let table = Table::new(rows)
        .widths(&[
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(35),
        ])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(table_title)
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_network_activity(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(area.inner(&Margin::new(1, 1)));

        // 실제 Ping/Pong 성공률 계산
        let ping_success_rate = if self.network_stats.pings_sent > 0 {
            ((self.network_stats.pongs_received * 100) / self.network_stats.pings_sent).min(100)
        } else {
            0
        };
        
        let gauge_color = if ping_success_rate >= 90 {
            Color::Green
        } else if ping_success_rate >= 70 {
            Color::Yellow
        } else {
            Color::Red
        };
        
        let ping_gauge = Gauge::default()
            .block(Block::default().title("Ping/Pong Success Rate"))
            .gauge_style(Style::default().fg(gauge_color))
            .percent(ping_success_rate as u16)
            .label(format!("{}% ({}/{})", ping_success_rate, self.network_stats.pongs_received, self.network_stats.pings_sent));

        f.render_widget(ping_gauge, chunks[0]);

        // 실제 메시지 통계
        let messages_text = Text::from(vec![
            Line::from(vec![
                Span::raw("Messages: "),
                Span::styled(format!("{}", self.network_stats.messages_sent), Style::default().fg(Color::Green)),
                Span::raw(" sent, "),
                Span::styled(format!("{}", self.network_stats.messages_received), Style::default().fg(Color::Blue)),
                Span::raw(" received | Connections: "),
                Span::styled(format!("{}", self.network_stats.connections_established), Style::default().fg(Color::Cyan)),
                Span::raw("/"),
                Span::styled(format!("{}", self.network_stats.connections_lost), Style::default().fg(Color::Red)),
            ]),
        ]);

        let messages_paragraph = Paragraph::new(messages_text)
            .block(Block::default().title("Network Statistics"));

        f.render_widget(messages_paragraph, chunks[1]);

        let activity_block = Block::default()
            .borders(Borders::ALL)
            .title("Network Activity");
        f.render_widget(activity_block, area);
    }

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        // 캐시된 실제 로그 데이터 사용
        let display_logs: Vec<String> = self.recent_logs
            .iter()
            .rev()
            .take(15)
            .cloned()
            .collect();

        let log_items: Vec<ListItem> = if display_logs.is_empty() {
            // 로그가 비어있으면 기본 메시지 표시
            vec![
                ListItem::new(Text::from("📋 No logs available yet..."))
                    .style(Style::default().fg(Color::Gray)),
                ListItem::new(Text::from("🔄 Waiting for network activity..."))
                    .style(Style::default().fg(Color::Gray)),
            ]
        } else {
            display_logs
                .iter()
                .map(|log| {
                    let style = if log.contains("✅") {
                        Style::default().fg(Color::Green)
                    } else if log.contains("🏓") {
                        Style::default().fg(Color::Cyan)
                    } else if log.contains("📤") {
                        Style::default().fg(Color::Yellow)
                    } else if log.contains("🔍") {
                        Style::default().fg(Color::Blue)
                    } else if log.contains("📡") {
                        Style::default().fg(Color::Magenta)
                    } else if log.contains("⚠️") {
                        Style::default().fg(Color::Yellow)
                    } else if log.contains("❌") {
                        Style::default().fg(Color::Red)
                    } else if log.contains("🔎") {
                        Style::default().fg(Color::Blue)
                    } else if log.contains("🚀") {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Text::from(log.as_str())).style(style)
                })
                .collect()
        };

        let log_count = self.recent_logs.len();
        let logs_title = format!("Recent Logs ({})", log_count);
        
        let logs_list = List::new(log_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(logs_title)
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_widget(logs_list, area);
    }
}

pub async fn run_tui(network: Arc<Network>) -> Result<(), Box<dyn std::error::Error>> {
    // 터미널 초기화
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TUI 앱 실행
    let mut app = TuiApp::new(network);
    let res = app.run(&mut terminal).await;

    // 터미널 복원
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}