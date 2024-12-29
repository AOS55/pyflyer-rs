use bevy::{prelude::*, window::PrimaryWindow};
use flyer::{
    plugins::Id,
    resources::{AgentState, StepCommand, UpdateControl},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use pyflyer::gym::{setup_app, ActionSpace, ConfigError, EnvConfig, ToControls, ToObservation};

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Initialize { config: serde_json::Value },
    Step { actions: HashMap<String, Vec<f64>> },
    Reset { seed: Option<u64> },
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    obs: Vec<f64>,
    reward: f64,
    terminated: bool,
    truncated: bool,
    info: serde_json::Value,
}

#[derive(Resource)]
struct ServerState {
    conn: Arc<Mutex<TcpStream>>,
    initialized: bool,
    config: EnvConfig,
}

// #[derive(Resource)]
// struct StepState {
//     needs_step: bool,
//     step_count: usize,
// }

// fn handle_stepping(
//     mut state: ResMut<StepState>,
//     mut update_control: ResMut<UpdateControl>,
//     // Use window query to force main thread
//     _window: Query<&Window, With<PrimaryWindow>>,
// ) {
//     if state.needs_step {
//         update_control.remaining_steps = state.step_count;
//         state.needs_step = false;
//     }
// }

fn get_numeric_id(aircraft_id: &str) -> Id {
    // The aircraft ID is already in the correct format from the config builder
    // e.g. "aircraft_0", "aircraft_1", etc.
    Id::Named(aircraft_id.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Bevy server...");

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("PORT={}", port);

    // Accept one connection
    let (stream, addr) = listener.accept().unwrap();
    println!("Connection accepted from: {}", addr);
    let stream = Arc::new(Mutex::new(stream));

    // Wait for initial config
    println!("Waiting for initial config...");
    let config = receive_initial_config(&stream)?;
    println!("Initial config received successfully");

    // Convert config to EnvConfig
    println!("Converting config to EnvConfig...");
    let env_config = EnvConfig::from_json(&config)?;
    println!("Config converted successfully");

    // Send ready signal
    {
        let response = serde_json::json!({ "status": "ready" });
        let response_str = serde_json::to_string(&response)? + "\n";
        match stream.lock() {
            Ok(guard) => {
                if let Ok(mut clone) = guard.try_clone() {
                    clone.write_all(response_str.as_bytes())?;
                    clone.flush()?;
                    println!("Ready signal sent successfully: {}", response_str.trim());
                }
            }
            Err(e) => eprintln!("Failed to acquire stream lock: {}", e),
        }
    }

    // Create and configure bevy app
    println!("Initializing Bevy app...");
    let mut app = App::new();

    // Add server state resource
    app.insert_resource(ServerState {
        conn: stream.clone(),
        initialized: false,
        config: env_config.clone(),
    });

    // Get asset directorys in correct location
    let current_dir = env::current_dir().unwrap();
    let asset_path = current_dir
        .join("pyflyer-rs/flyer-rs/assets")
        .to_str()
        .unwrap()
        .to_string();

    app = setup_app(app, env_config.clone(), asset_path);

    // Add command handling system
    app.add_systems(Update, handle_commands);
    // app.add_systems(Update, handle_stepping);

    // Set server state to initilized
    app.world_mut()
        .get_resource_mut::<ServerState>()
        .unwrap()
        .initialized = true;

    // Run app
    println!("Starting Bevy app...");
    app.run();

    Ok(())
}

fn receive_initial_config(stream: &Arc<Mutex<TcpStream>>) -> std::io::Result<serde_json::Value> {
    println!("Attempting to receive initial config...");
    let guard = match stream.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to acquire stream lock: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire lock",
            ));
        }
    };

    let mut reader = BufReader::new(guard.try_clone()?);
    let mut line = String::new();

    println!("Reading line from stream...");
    match reader.read_line(&mut line) {
        Ok(n) => println!("Read {} bytes", n),
        Err(e) => eprintln!("Error reading line: {}", e),
    }
    println!("Received raw line: '{}'", line);

    let cmd: Command = match serde_json::from_str(&line) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    match cmd {
        Command::Initialize { config } => {
            println!("Got Initialize command with config");

            // Immediately send ready response
            let response = serde_json::json!({ "status": "ready" });
            let response_str = serde_json::to_string(&response)? + "\n";

            // Write response using the original guard to maintain lock
            match guard.try_clone()?.write_all(response_str.as_bytes()) {
                Ok(_) => println!("Wrote response bytes"),
                Err(e) => eprintln!("Failed to write response: {}", e),
            }

            match guard.try_clone()?.flush() {
                Ok(_) => println!("Flushed stream"),
                Err(e) => eprintln!("Failed to flush stream: {}", e),
            }

            println!("Sent ready response: {}", response_str);
            Ok(config)
        }
        _ => {
            let err = format!("Unexpected command received: {:?}", cmd);
            eprintln!("{}", err);
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        }
    }
}

fn handle_commands(
    mut server: ResMut<ServerState>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
) {
    if !server.initialized {
        // Ignore commands until initialized
        return;
    }

    let cmd = {
        let guard = server.conn.lock().unwrap();
        // Clone the stream first to get a mutable copy
        let stream = guard.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        if reader.read_line(&mut line).is_ok() && !line.is_empty() {
            serde_json::from_str::<Command>(&line).ok()
        } else {
            None
        }
    };

    // Non-blocking read
    if let Some(cmd) = cmd {
        match cmd {
            Command::Initialize { .. } => {
                // Ignore after initial setup
            }
            Command::Step { actions } => {
                // Process actions for each aircaft and add to queue
                if let Ok(mut action_queue) = agent_state.action_queue.lock() {
                    for (aircraft_id, action) in actions {
                        // Get the action space for this aircraft
                        if let Some(action_space) = server.config.action_spaces.get(&aircraft_id) {
                            // Convert normalized actions to actual controls
                            let controls = action_space.to_controls(action);

                            // You'll need a way to map string aircraft_id to numeric Id
                            // This could be stored in ServerState or handled by a mapping function
                            let id = get_numeric_id(&aircraft_id);

                            // Insert controls for this aircraft
                            action_queue.insert(id, controls);
                        } else {
                            warn!("Action space not found for aircraft: {}", aircraft_id);
                        }
                    }
                }

                // Set step flag, wait to execute action queue for steps before moving to next step
                update_control.remaining_steps = server.config.steps_per_action;

                // Wait for steps to complete before sending response
                while update_control.remaining_steps > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }

                // Get a new clone for writing response
                if let Ok(guard) = server.conn.lock() {
                    if let Ok(mut stream) = guard.try_clone() {
                        if let Ok(state_buffer) = agent_state.state_buffer.lock() {
                            // Collect Observations for all aircraft
                            let mut all_observations = Vec::new();

                            for (id, state) in state_buffer.iter() {
                                // Get the observation space for this aircraft
                                let id_str = match id {
                                    Id::Named(name) => name.clone(),
                                    Id::Entity(entity) => entity.to_string(),
                                };

                                // Get the observation space for this aircraft
                                if let Some(obs_space) =
                                    server.config.observation_spaces.get(&id_str)
                                {
                                    // Convert state to observation
                                    let obs = obs_space.to_observation(state);
                                    all_observations.extend(obs);
                                } else {
                                    warn!("Observation space not found for aircraft: {:?}", id);
                                }
                            }

                            let response = Response {
                                obs: all_observations,
                                reward: 0.0,
                                terminated: agent_state.terminated,
                                truncated: agent_state.truncated,
                                info: serde_json::json!({}),
                            };
                            writeln!(stream, "{}", serde_json::to_string(&response).unwrap())
                                .unwrap();
                        }
                    }
                }
            }
            Command::Reset { seed } => {
                // Handle reset command
            }
            Command::Close => {
                // Handle close command
            }
        }
    }
}
