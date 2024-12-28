use bevy::{prelude::*, window::PrimaryWindow};
use flyer::resources::{AgentState, StepCommand, UpdateControl};
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use pyflyer::gym::{setup_app, EnvConfig};

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Initialize { config: serde_json::Value },
    Step { action: Vec<f32> },
    Reset { seed: Option<u64> },
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    obs: Vec<f32>,
    reward: f32,
    terminated: bool,
    truncated: bool,
    info: serde_json::Value,
}

#[derive(Resource)]
struct ServerState {
    conn: Arc<Mutex<TcpStream>>,
    initialized: bool,
}

#[derive(Resource)]
struct StepState {
    needs_step: bool,
    step_count: usize,
}

fn handle_stepping(
    mut state: ResMut<StepState>,
    mut update_control: ResMut<UpdateControl>,
    // Use window query to force main thread
    _window: Query<&Window, With<PrimaryWindow>>,
) {
    if state.needs_step {
        update_control.remaining_steps = state.step_count;
        state.needs_step = false;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("PORT={}", port);

    // Accept one connection
    let (stream, _) = listener.accept().unwrap();
    let stream = Arc::new(Mutex::new(stream));

    // Wait for initial config
    let config = receive_initial_config(&stream)?;

    // Create and configure bevy app
    let mut app = App::new();

    // Convert config to EnvConfig
    let env_config = EnvConfig::from_pydict(&config)?;

    // Add server state resource
    app.insert_resource(ServerState {
        conn: stream.clone(),
        initialized: false,
    });

    // Get asset directorys in correct location
    let current_dir = env::current_dir().unwrap();
    let asset_path = current_dir
        .join("pyflyer-rs/flyer-rs/assets")
        .to_str()
        .unwrap()
        .to_string();

    app = setup_app(app, config.clone(), asset_path);

    // Add command handling system
    app.add_systems(Update, handle_commands);

    // Send ready signal
    let response = serde_json::json!({
        "status": "ready",
    });

    writeln!(
        stream.lock().unwrap(),
        "{}",
        serde_json::to_string(&response)?
    )?;

    // Run app
    app.run();

    Ok(())
}

fn receive_initial_config(stream: &Arc<Mutex<TcpStream>>) -> std::io::Result<serde_json::Value> {
    let guard = stream.lock().unwrap();
    let mut reader = BufReader::new(guard.try_clone()?);
    let mut line = String::new();

    reader.read_line(&mut line)?;

    let cmd: Command = serde_json::from_str(&line)?;

    match cmd {
        Command::Initialize { config } => Ok(config),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected Initialize command",
        )),
    }
}

fn handle_commands(
    mut server: ResMut<ServerState>,
    mut step_state: ResMut<StepState>,
    agent_state: Res<AgentState>,
) {
    let stream = server.conn.lock().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();

    // Non-blocking read
    if reader.read_line(&mut line).is_ok() && !line.is_empty() {
        if let Ok(cmd) = serde_json::from_str::<Command>(&line) {
            match cmd {
                Command::Step { action } => {
                    // Handle step command
                }
                Command::Reset { seed } => {
                    // Handle reset command
                }
                Command::Close => {
                    // Handle close command
                }
                Command::Initialize { .. } => {
                    // Ignore after initial setup
                }
            }
        }
    }
}
