use tokio::sync::broadcast;
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
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Frame, Terminal,
};
use std::{env, error::Error, io, time::{Duration, Instant}};
use pioneerfs::{Network, DebugLevel};
use std::sync::{Arc, Mutex};
use tokio::task;

mod webui;
use libp2p::PeerId;
use rand::Rng;

enum InputMode {
    Normal,
    Editing,
}

struct App {
    input: String,
    input_mode: InputMode,
    network: Arc<Mutex<Network>>,
    messages: Vec<String>,
    messages_state: ListState,
    scroll_offset: usize,
}

impl App {
    fn new(debug_level: DebugLevel, network: Arc<Mutex<Network>>) -> App {
        network.lock().unwrap().set_debug_level(debug_level);
        let (tx, _rx) = broadcast::channel(100);
        let app = App {
            input: String::new(),
            input_mode: InputMode::Normal,
            network,
            messages: Vec::new(),
            messages_state: ListState::default(),
            scroll_offset: 0,
        };

        let sender = tx.clone();
        app.network.lock().unwrap().message_sender = Some(sender.clone());
        app.network.lock().unwrap().message_sender = Some(sender);
        adjust_pricing(&mut network.lock().unwrap());
        app
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--test".to_string()) {
        // Run in test mode
        let mut network = Network::new();
        network.set_debug_level(DebugLevel::Low);
        let (tx, _rx) = broadcast::channel(100);
        run_replication_tests(&mut network, tx.clone());
    } else if args.contains(&"--advanced-tests".to_string()) {
        // Run advanced network tests
        let mut network = Network::new();
        network.set_debug_level(DebugLevel::Low);
        let (tx, _rx) = broadcast::channel(100);
        run_advanced_network_tests(&mut network, tx.clone());
    } else {
        // Run in normal mode
        let network = Arc::new(Mutex::new(Network::new()));

        let (tx, rx) = broadcast::channel(100);

        let webui_handle = {
            let network_clone = Arc::clone(&network);
            let tx_clone = tx.clone();
            task::spawn(async move {
                webui::start_webui(network_clone, tx_clone).await;
            })
        };

        let terminal_handle = {
            let network_clone = Arc::clone(&network);
            task::spawn_blocking(move || -> Result<(), Box<dyn Error + Send + Sync>> {
                enable_raw_mode()?;
                let mut stdout = io::stdout();
                execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
                let backend = CrosstermBackend::new(stdout);
                let mut terminal = Terminal::new(backend)?;

                let debug_level = if args.contains(&"--debug".to_string()) {
                    DebugLevel::High
                } else {
                    DebugLevel::None
                };

                let tick_rate = Duration::from_millis(250);
                let mut app = App::new(debug_level, network_clone);
                app.messages.push("WebUI is available at http://localhost:3030".to_string());
                let res = run_app(&mut terminal, app, tick_rate, rx);

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
            })
        };

        if let Err(e) = webui_handle.await {
            eprintln!("Error in webui_handle: {:?}", e);
        }

        if let Err(e) = terminal_handle.await {
            eprintln!("Error in terminal_handle: {:?}", e);
        }
    }

}
    Ok(())




    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    mut rx: broadcast::Receiver<String>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

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
                        KeyCode::Up => {
                            app.scroll_offset = app.scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            let max_scroll = app.messages.len().saturating_sub(1);
                            app.scroll_offset = (app.scroll_offset + 1).min(max_scroll);
                        }
                        KeyCode::PageUp => {
                            app.scroll_offset = app.scroll_offset.saturating_sub(10);
                        }
                        KeyCode::PageDown => {
                            let max_scroll = app.messages.len().saturating_sub(1);
                            app.scroll_offset = (app.scroll_offset + 10).min(max_scroll);
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
        if let Ok(message) = rx.try_recv() {
            app.messages.push(message);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

fn ui(f: &mut Frame, app: &mut App) {
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

    let messages_height = chunks[2].height as usize - 2; // Subtract 2 for the border
    let total_messages = app.messages.len();
    // Removed unused variable
    let start_index = (total_messages as i64 - messages_height as i64 - app.scroll_offset as i64).max(0) as usize;
    let end_index = (start_index + messages_height).min(total_messages);
    
    let messages: Vec<ListItem> = app.messages.iter()
        .skip(start_index)
        .take(end_index - start_index)
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Line::from(Span::raw(format!("{}: {}", start_index + i + 1, m)))];
            ListItem::new(content)
        })
        .collect();
    
    let messages = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title(format!("Messages ({}-{}/{})", start_index + 1, end_index, total_messages)))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(messages, chunks[2], &mut app.messages_state);
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
            app.network.lock().unwrap().add_client(peer_id);
            app.messages.push(format!("Added client with PeerId: {}", peer_id));
        }
        "add_sp" => {
            if parts.len() != 2 {
                app.messages.push("Usage: add_sp <price_per_gb>".to_string());
                return;
            }
            let peer_id = PeerId::random();
            let price_per_gb = parts[1].parse::<u64>().unwrap_or(0);
            app.network.lock().unwrap().add_storage_node(peer_id, price_per_gb);
            app.messages.push(format!("Added storage provider (SP) with PeerId: {} and price per GB: {}", peer_id, price_per_gb));
        }
        "list_clients" => {
            let clients = app.network.lock().unwrap().list_clients();
            app.messages.push("Clients:".to_string());
            for (i, client_id) in clients.iter().enumerate() {
                app.messages.push(format!("  {}: {}", i + 1, client_id));
            }
        }
        "list_sps" => {
            let sps = app.network.lock().unwrap().list_storage_nodes();
            app.messages.push("Storage Providers (SPs):".to_string());
            for (i, sp_id) in sps.iter().enumerate() {
                app.messages.push(format!("  {}: {}", i + 1, sp_id));
            }
        }
        "upload_file" => {
            if parts.len() != 6 {
                app.messages.push("Usage: upload_file <client_id> <sp_id> <filename> <content> <replication_factor>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let _sp_id = PeerId::from_bytes(&hex::decode(parts[2]).unwrap()).unwrap();
            let filename = parts[3].to_string();
            let content = parts[4].as_bytes().to_vec();
            let replication_factor = parts[5].parse::<usize>().unwrap_or(3); // Default to 3 if parsing fails

            match app.network.lock().unwrap().upload_file(&client_id, filename, content, replication_factor) {
                Ok(_) => app.messages.push(format!("File uploaded successfully with replication factor {}", replication_factor)),
                Err(e) => app.messages.push(format!("Failed to upload file: {}", e)),
            }
        }
        "download_file" => {
            if parts.len() != 4 {
                app.messages.push("Usage: download_file <client_id> <sp_id> <filename>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let _sp_id = PeerId::from_bytes(&hex::decode(parts[2]).unwrap()).unwrap();
            let filename = parts[3];

            match app.network.lock().unwrap().download_file(&client_id, filename) {
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

            match app.network.lock().unwrap().renew_deal(&client_id, &sp_id, filename) {
                Ok(_) => app.messages.push("Deal renewed successfully".to_string()),
                Err(e) => app.messages.push(format!("Failed to renew deal: {}", e)),
            }
        }
        "check_deals" => {
            app.network.lock().unwrap().check_deals();
            app.messages.push("Checked and removed expired deals".to_string());
        }
        "get_reputation" => {
            if parts.len() != 2 {
                app.messages.push("Usage: get_reputation <sp_id>".to_string());
                return;
            }
            let sp_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            if let Some(sp) = app.network.lock().unwrap().storage_nodes().get(&sp_id) {
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
            app.network.lock().unwrap().add_storage_offer(sp_id, price_per_gb, available_space);
            app.messages.push("Storage offer added to the marketplace".to_string());
        }
        "list_storage_offers" => {
            let network = app.network.lock().unwrap();
            let offers = network.get_storage_offers();
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
            match app.network.lock().unwrap().accept_storage_offer(&client_id, offer_index, file_size) {
                Ok(_) => app.messages.push("Storage offer accepted successfully".to_string()),
                Err(e) => app.messages.push(format!("Failed to accept storage offer: {}", e)),
            }
        }
        "increase_replication" => {
            if parts.len() != 4 {
                app.messages.push("Usage: increase_replication <client_id> <filename> <new_replication_factor>".to_string());
                return;
            }
            let client_id = PeerId::from_bytes(&hex::decode(parts[1]).unwrap()).unwrap();
            let filename = parts[2];
            let new_replication_factor = parts[3].parse::<usize>().unwrap_or(0);
            match app.network.lock().unwrap().request_higher_replication(&client_id, filename, new_replication_factor) {
                Ok(_) => app.messages.push(format!("Successfully increased replication for file {} to factor {}", filename, new_replication_factor)),
                Err(e) => app.messages.push(format!("Failed to increase replication: {}", e)),
            }
        }
        _ => {
            app.messages.push(format!("Unknown command: {}. Type 'help' for available commands.", command));
        }
    }

    app.input.clear();
}
fn adjust_pricing(network: &mut Network) {
    for sp in network.storage_nodes.values_mut() {
        let usage_ratio = sp.used_space() as f64 / sp.total_space() as f64;
        if usage_ratio > 0.8 {
            sp.set_price_per_gb(sp.price_per_gb() + 1); // Increase price if usage is above 80%
        } else if usage_ratio < 0.2 {
            sp.set_price_per_gb(sp.price_per_gb().saturating_sub(1)); // Decrease price if usage is below 20%
        }
    }
}

fn run_replication_tests(network: &mut Network, tx: broadcast::Sender<String>) {
    let mut rng = rand::thread_rng();
    let network = Arc::new(Mutex::new(Network::new()));
    let (tx, rx) = broadcast::channel(100);

    {
        let mut network = network.lock().unwrap();
        // Ensure we have at most 12 storage providers (SPs)
        for _ in 0..12 {
            let sp_id = PeerId::random();
            network.add_storage_node(sp_id.clone(), rng.gen_range(10..20));
            tx.send(format!("Added storage provider (SP) with PeerId: {}", sp_id)).unwrap();
        }

        for i in 0..100 {
            let client_id = PeerId::random();
            network.add_client(client_id.clone());
            tx.send(format!("Added client with PeerId: {}", client_id)).unwrap();
            
            let filename = format!("test_file_{}.txt", i);
            let data = vec![0u8; rng.gen_range(1000..10000)];
            let replication_factor = rng.gen_range(2..5);
            
            match network.upload_file(&client_id, filename.clone(), data, replication_factor) {
                Ok(_) => {
                    println!("Test {}: File uploaded successfully", i);
                    tx.send(format!("Test {}: File uploaded successfully", i)).unwrap();
                }
                Err(e) => {
                    println!("Test {}: Upload failed - {}", i, e);
                    tx.send(format!("Test {}: Upload failed - {}", i, e)).unwrap();
                }
            }
            
            // Display abstract network state
            display_abstract_network(&network);
        }
    }
    Ok(())
}
    for sp in network.storage_nodes.values_mut() {
        let usage_ratio = sp.used_space() as f64 / sp.total_space() as f64;
        if usage_ratio > 0.8 {
            sp.set_price_per_gb(sp.price_per_gb() + 1); // Increase price if usage is above 80%
        } else if usage_ratio < 0.2 {
            sp.set_price_per_gb(sp.price_per_gb().saturating_sub(1)); // Decrease price if usage is below 20%
        }
    }
}

fn display_abstract_network(network: &Network) {
    let status = network.get_network_status();
    println!("Network Abstract State:");
    println!("  Clients: {}", status.clients.len());
    println!("  Storage Nodes: {}", status.storage_nodes.len());
    println!("  Active Deals: {}", status.deals);
    println!("  Marketplace Offers: {}", status.marketplace);
    println!("----------------------------");
}
fn run_advanced_network_tests(network: &mut Network, tx: broadcast::Sender<String>) {
    let mut rng = rand::thread_rng();
    
    for i in 0..100 {
        let client_id = PeerId::random();
        network.add_client(client_id.clone());
        tx.send(format!("Added client with PeerId: {}", client_id)).unwrap();
        
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id.clone(), rng.gen_range(10..20));
        tx.send(format!("Added storage provider (SP) with PeerId: {}", sp_id)).unwrap();
        
        let filename = format!("test_file_{}.txt", i);
        let data = vec![0u8; rng.gen_range(1000..10000)];
        let replication_factor = rng.gen_range(2..5);
        
        match network.upload_file(&client_id, filename.clone(), data, replication_factor) {
            Ok(_) => {
                tx.send(format!("Test {}: File uploaded successfully", i)).unwrap();
            }
            Err(e) => {
                tx.send(format!("Test {}: Upload failed - {}", i, e)).unwrap();
            }
        }
        
        // Display abstract network state
        let status = network.get_network_status();
        tx.send(format!("Network status: {:?}", status)).unwrap();
    }
}
