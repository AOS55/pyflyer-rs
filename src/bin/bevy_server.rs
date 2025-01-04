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

/// Enum representing commands sent to the server.
#[derive(Debug, Serialize, Deserialize)]
enum Command {
    /// Initialize the environment with a configuration.
    Initialize { config: serde_json::Value },
    /// Perform a simulation step with provided actions.
    Step { actions: HashMap<String, Vec<f64>> },
    /// Reset the environment with an optional random seed.
    Reset { seed: Option<u64> },
    /// Close the server connection.
    Close,
}

/// Struct representing the response from the server after handling a command.
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    /// Observation data from the environment.
    obs: Vec<f64>,
    /// Reward for the current step.
    reward: f64,
    /// Whether the episode is terminated.
    terminated: bool,
    /// Whether the episode is truncated.
    truncated: bool,
    /// Additional info about the step or environment state.
    info: serde_json::Value,
}

/// Resource representing the server state.
#[derive(Resource)]
struct ServerState {
    /// Connection to the client.
    conn: Arc<Mutex<TcpStream>>,
    /// Whether the server is initialized.
    initialized: bool,
    /// Configuration of the environment.
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

/// Function to generate an ID object from an aircraft string ID.
///
/// # Arguments
/// * `aircraft_id` - A string representing the aircraft ID.
///
/// # Returns
/// * `Id` - The ID corresponding to the string ID.
fn get_numeric_id(aircraft_id: &str) -> Id {
    Id::Named(aircraft_id.to_string())
}

/// The main function initializes the server and starts the Bevy app.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if the server starts successfully, an error otherwise.
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

    // Configure asset directory
    let current_dir = env::current_dir().unwrap();
    let asset_path = current_dir
        .join("pyflyer-rs/flyer-rs/assets")
        .to_str()
        .unwrap()
        .to_string();

    app = setup_app(app, env_config.clone(), asset_path);

    // Add systems for handling commands
    app.add_systems(Update, handle_commands);
    // app.add_systems(Update, handle_stepping);

    // Mark the server state as initialized
    app.world_mut()
        .get_resource_mut::<ServerState>()
        .unwrap()
        .initialized = true;

    // Run app
    println!("Starting Bevy app...");
    app.run();

    Ok(())
}

/// Function to receive the initial configuration from the client.
///
/// # Arguments
/// * `stream` - The stream to receive data from.
///
/// # Returns
/// * `Result<serde_json::Value, std::io::Error>` - The configuration data or an error.
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

/// System to handle commands received from the client.
///
/// # Arguments
/// * `server` - The server state resource.
/// * `update_control` - The update control resource.
/// * `agent_state` - The agent state resource.
fn handle_commands(
    mut server: ResMut<ServerState>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
) {
    if !server.initialized {
        // Ignore commands until initialized
        println!("Server not initialized yet, ignoring command");
        return;
    }

    println!("Handle commands called");

    let cmd = {
        let guard = server.conn.lock().unwrap();
        // Clone the stream first to get a mutable copy
        let stream = guard.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        if reader.read_line(&mut line).is_ok() && !line.is_empty() {
            println!("Received command: {}", line.trim());
            match serde_json::from_str::<Command>(&line) {
                Ok(cmd) => {
                    println!("Parsed command: {:?}", cmd); // Show parsed command
                    Some(cmd)
                }
                Err(e) => {
                    println!("Failed to parse command: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    // Non-blocking read
    if let Some(cmd) = cmd {
        println!("Processing command: {:?}", cmd);
        match cmd {
            Command::Initialize { .. } => {
                // Ignore after initial setup
            }
            Command::Step { actions } => {
                // TODO: Consider using std::sync::condvar here

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
                println!("Handling Reset command");

                // Clear action queue
                if let Ok(mut action_queue) = agent_state.action_queue.lock() {
                    action_queue.clear();
                }

                update_control.remaining_steps = server.config.steps_per_action;
                while update_control.remaining_steps > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }

                // Create response with proper error handling
                let result = match (server.conn.lock(), agent_state.state_buffer.lock()) {
                    (Ok(guard), Ok(state_buffer)) => {
                        if let Ok(mut stream) = guard.try_clone() {
                            let mut all_observations = Vec::new();

                            for (id, state) in state_buffer.iter() {
                                let id_str = match id {
                                    Id::Named(name) => name.clone(),
                                    Id::Entity(entity) => entity.to_string(),
                                };

                                if let Some(obs_space) =
                                    server.config.observation_spaces.get(&id_str)
                                {
                                    let obs = obs_space.to_observation(state);
                                    all_observations.extend(obs);
                                } else {
                                    warn!("Observation space not found for aircraft: {:?}", id);
                                }
                            }

                            let response = Response {
                                obs: all_observations,
                                reward: 0.0,
                                terminated: false,
                                truncated: false,
                                info: serde_json::json!({}),
                            };

                            // Log response for debugging
                            println!("Sending reset response: {:?}", response);
                            let response = "test";

                            if let Err(e) =
                                writeln!(stream, "{}", serde_json::to_string(&response).unwrap())
                            {
                                error!("Failed to write response: {}", e);
                            }
                        }
                    }
                    (Err(e1), _) => error!("Failed to lock connection: {}", e1),
                    (_, Err(e2)) => error!("Failed to lock state buffer: {}", e2),
                };
                result
            }
            Command::Close => {
                // Handle close command
            }
        }
    }
}
