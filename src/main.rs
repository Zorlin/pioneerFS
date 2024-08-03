use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}};
use pioneerfs::Network;
use libp2p::PeerId;

enum InputMode {
    Normal,
    Editing,
}

struct App {
    input: String,
    input_mode: InputMode,
    network: Network,
    messages: Vec<String>,
}

impl App {
    fn new() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            network: Network::new(),
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            execute_command(&mut app);
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let help_message = match app.input_mode {
        InputMode::Normal => Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ]),
            Line::from("Type 'help' for available commands."),
        ]).style(Style::default().add_modifier(Modifier::RAPID_BLINK)),
        InputMode::Editing => Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute command"),
            ]),
            Line::from("Type 'help' for available commands."),
        ]),
    };
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Command Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor at the end of the input text
                chunks[1].x + app.input.len() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Line::from(format!("{}: {}", i, m))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[2]);
}

fn execute_command(app: &mut App) {
    let command = app.input.trim();
    app.messages.push(format!("Executing: {}", command));

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => {
            app.messages.push("Available commands:".to_string());
            app.messages.push("  help - Display this help message".to_string());
            app.messages.push("  add_client - Add a new client".to_string());
            app.messages.push("  add_sp <price_per_gb> - Add a new storage provider (SP)".to_string());
            app.messages.push("  list_clients - List all clients".to_string());
            app.messages.push("  list_sps - List all storage providers".to_string());
            app.messages.push("  upload_file <client_id> <sp_id> <filename> <content> - Upload a file".to_string());
            app.messages.push("  download_file <client_id> <sp_id> <filename> - Download a file".to_string());
            app.messages.push("  renew_deal <client_id> <sp_id> <filename> - Renew a storage deal".to_string());
            app.messages.push("  check_deals - Check and remove expired deals".to_string());
            app.messages.push("  get_reputation <sp_id> - Get the reputation of a storage provider".to_string());
            app.messages.push("  add_storage_offer <sp_id> <price_per_gb> <available_space> - Add a storage offer to the marketplace".to_string());
            app.messages.push("  list_storage_offers - List all storage offers in the marketplace".to_string());
            app.messages.push("  accept_storage_offer <client_id> <offer_index> <file_size> - Accept a storage offer".to_string());
        }
        "add_client" => {
            let peer_id = PeerId::random();
            app.network.add_client(peer_id);
            app.messages.push(format!("Added client with PeerId: {}", peer_id));
        }
        "add_sp" => {
            let peer_id = PeerId::random();
            app.network.add_storage_node(peer_id);
            app.messages.push(format!("Added storage provider (SP) with PeerId: {}", peer_id));
        }
        "list_clients" => {
            let clients = app.network.list_clients();
            app.messages.push("Clients:".to_string());
            for (i, client_id) in clients.iter().enumerate() {
                app.messages.push(format!("  {}: {}", i + 1, client_id));
            }
        }
        "list_sps" => {
            let sps = app.network.list_storage_nodes();
            app.messages.push("Storage Providers (SPs):".to_string());
            for (i, sp_id) in sps.iter().enumerate() {
                app.messages.push(format!("  {}: {}", i + 1, sp_id));
            }
        }
        "upload_file" => {
            if parts.len() != 5 {
                app.messages.push("Usage: upload_file <client_id> <sp_id> <filename> <content>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let sp_id = PeerId::from_bytes(&hex::decode(parts[2]).unwrap()).unwrap();
            let filename = parts[3].to_string();
            let content = parts[4].as_bytes().to_vec();

            match app.network.upload_file(&client_id, &sp_id, filename, content) {
                Ok(_) => app.messages.push("File uploaded successfully".to_string()),
                Err(e) => app.messages.push(format!("Failed to upload file: {}", e)),
            }
        }
        "download_file" => {
            if parts.len() != 4 {
                app.messages.push("Usage: download_file <client_id> <sp_id> <filename>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let sp_id = PeerId::from_bytes(&hex::decode(parts[2]).unwrap()).unwrap();
            let filename = parts[3];

            match app.network.download_file(&client_id, &sp_id, filename) {
                Ok(data) => app.messages.push(format!("Downloaded file content: {:?}", String::from_utf8_lossy(&data))),
                Err(e) => app.messages.push(format!("Failed to download file: {}", e)),
            }
        }
        "renew_deal" => {
            if parts.len() != 4 {
                app.messages.push("Usage: renew_deal <client_id> <sp_id> <filename>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let sp_id = PeerId::from_bytes(&hex::decode(parts[2]).unwrap()).unwrap();
            let filename = parts[3];

            match app.network.renew_deal(&client_id, &sp_id, filename) {
                Ok(_) => app.messages.push("Deal renewed successfully".to_string()),
                Err(e) => app.messages.push(format!("Failed to renew deal: {}", e)),
            }
        }
        "check_deals" => {
            app.network.check_deals();
            app.messages.push("Checked and removed expired deals".to_string());
        }
        "get_reputation" => {
            if parts.len() != 2 {
                app.messages.push("Usage: get_reputation <sp_id>".to_string());
                return;
            }
            let sp_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            if let Some(sp) = app.network.storage_nodes().get(&sp_id) {
                app.messages.push(format!("Reputation of SP {}: {}", sp_id, sp.reputation()));
            } else {
                app.messages.push(format!("Storage provider with ID {} not found", sp_id));
            }
        }
        "add_storage_offer" => {
            if parts.len() != 4 {
                app.messages.push("Usage: add_storage_offer <sp_id> <price_per_gb> <available_space>".to_string());
                return;
            }
            let sp_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let price_per_gb = parts[2].parse::<u64>().unwrap();
            let available_space = parts[3].parse::<usize>().unwrap();
            app.network.add_storage_offer(sp_id, price_per_gb, available_space);
            app.messages.push("Storage offer added to the marketplace".to_string());
        }
        "list_storage_offers" => {
            let offers = app.network.get_storage_offers();
            app.messages.push("Storage Offers:".to_string());
            for (i, offer) in offers.iter().enumerate() {
                app.messages.push(format!("  {}: SP: {}, Price per GB: {}, Available Space: {} bytes", 
                    i, offer.storage_node_id, offer.price_per_gb, offer.available_space));
            }
        }
        "accept_storage_offer" => {
            if parts.len() != 4 {
                app.messages.push("Usage: accept_storage_offer <client_id> <offer_index> <file_size>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let offer_index = parts[2].parse::<usize>().unwrap();
            let file_size = parts[3].parse::<usize>().unwrap();
            match app.network.accept_storage_offer(&client_id, offer_index, file_size) {
                Ok(_) => app.messages.push("Storage offer accepted successfully".to_string()),
                Err(e) => app.messages.push(format!("Failed to accept storage offer: {}", e)),
            }
        }
        _ => {
            app.messages.push(format!("Unknown command: {}. Type 'help' for available commands.", command));
        }
    }

    app.input.clear();
}
