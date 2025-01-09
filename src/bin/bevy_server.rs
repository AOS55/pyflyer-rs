use bevy::prelude::*;
use flyer::{
    plugins::{Id, ResetCompleteEvent, ResetRequestEvent, StepCompleteEvent, StepRequestEvent},
    resources::{AgentState, UpdateControl},
    systems::reset_env,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use pyflyer::gym::{setup_app, EnvConfig, ToControls, ToObservation};

/// Enum representing commands sent to the server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Command {
    /// Initialize the environment with a configuration.
    Initialize { config: serde_json::Value },
    /// Perform a simulation step with provided actions.
    Step {
        actions: HashMap<String, HashMap<String, f64>>,
    },
    /// Reset the environment with an optional random seed.
    Reset { seed: Option<u64> },
    /// Close the server connection.
    Close,
}

/// Struct representing the response from the server after handling a command.
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    /// Observation data from the environment (for each aircraft).
    obs: HashMap<String, HashMap<String, f64>>,
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
    // Initialize logging
    // setup_logging();

    println!("Starting Bevy server...");

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    println!("PORT={}", listener.local_addr().unwrap().port());

    // Accept one connection
    let (stream, _addr) = listener.accept().unwrap();
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
        let aircraft_info: Vec<_> = env_config
            .aircraft_configs
            .keys()
            .map(|name| {
                serde_json::json!({
                    "name": name,
                    "config": env_config.aircraft_configs.get(name).unwrap(),
                    "action_space": env_config.action_spaces.get(name).unwrap(),
                    "observation_space": env_config.observation_spaces.get(name).unwrap()
                })
            })
            .collect();

        let response = serde_json::json!({
            "status": "ready",
            "aircraft": aircraft_info
        });
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

    // Mark the server state as initialized
    app.world_mut()
        .get_resource_mut::<ServerState>()
        .unwrap()
        .initialized = true;

    // Add event and systems for handling step requests
    app.add_systems(FixedPreUpdate, handle_commands)
        .add_event::<StepRequestEvent>()
        .add_event::<StepCompleteEvent>()
        .add_systems(
            FixedUpdate,
            (handle_step_request, apply_deferred)
                .chain()
                .run_if(|control: Res<UpdateControl>| control.remaining_steps == 0),
        )
        .add_systems(
            FixedPostUpdate,
            (check_step_completion, handle_step_response).chain(),
        );

    // Add event for handling reset requests
    app.add_event::<ResetRequestEvent>()
        .add_event::<ResetCompleteEvent>()
        .add_systems(FixedUpdate, reset_env)
        .add_systems(FixedPostUpdate, handle_reset_response);

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
    info!("Attempting to receive initial config...");
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
            Ok(config)
        }
        _ => {
            let err = format!("Unexpected command received: {:?}", cmd);
            eprintln!("{}", err);
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        }
    }
}

fn handle_step_request(
    server: ResMut<ServerState>,
    mut step_requests: EventReader<StepRequestEvent>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
) {
    for request in step_requests.read() {
        if let Ok(mut action_queue) = agent_state.action_queue.lock() {
            // Apply actions
            for (aircraft_id, action) in &request.actions {
                if let Some(action_space) = server.config.action_spaces.get(aircraft_id) {
                    let controls = action_space.to_controls(action.clone());
                    let id = get_numeric_id(aircraft_id);
                    action_queue.insert(id, controls);
                }
            }
        }

        // Set steps to execute
        update_control.remaining_steps = server.config.steps_per_action;
    }
}

fn check_step_completion(
    agent_state: Res<AgentState>,
    mut step_complete: EventWriter<StepCompleteEvent>,
    server: Res<ServerState>,
) {
    if let Ok(state_buffer) = agent_state.state_buffer.lock() {
        // Add check for empty state buffer
        if state_buffer.is_empty() {
            warn!("State buffer empty, waiting for physics update");
            return;
        }

        let mut all_observations = HashMap::new();
        for (id, state) in state_buffer.iter() {
            let id_str = match id {
                Id::Named(name) => name.clone(),
                Id::Entity(entity) => entity.to_string(),
            };

            if let Some(obs_space) = server.config.observation_spaces.get(&id_str) {
                let obs = obs_space.to_observation(state);
                all_observations.insert(id_str, obs);
            }
        }

        // Only send event if we have observations
        if !all_observations.is_empty() {
            step_complete.send(StepCompleteEvent {
                observations: all_observations,
            });
        } else {
            warn!("No observations collected");
        }
    }
}

fn handle_step_response(
    mut step_completes: EventReader<StepCompleteEvent>,
    server: Res<ServerState>,
) {
    for event in step_completes.read() {
        if let Ok(guard) = server.conn.lock() {
            if let Ok(mut stream) = guard.try_clone() {
                let response = Response {
                    obs: event.observations.clone(),
                    reward: 0.0,
                    terminated: false,
                    truncated: false,
                    info: serde_json::json!({}),
                };

                let response_str = serde_json::to_string(&response).unwrap() + "\n";
                stream.write_all(response_str.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
    }
}

fn handle_reset_response(
    mut reset_complete: EventReader<ResetCompleteEvent>,
    agent_state: Res<AgentState>,
    server: Res<ServerState>,
) {
    for _ in reset_complete.read() {
        if let Ok(guard) = server.conn.lock() {
            if let Ok(mut stream) = guard.try_clone() {
                if let Ok(state_buffer) = agent_state.state_buffer.lock() {
                    let mut all_observations = HashMap::new();

                    for (id, state) in state_buffer.iter() {
                        let id_str = match id {
                            Id::Named(name) => name.clone(),
                            Id::Entity(entity) => entity.to_string(),
                        };

                        if let Some(obs_space) = server.config.observation_spaces.get(&id_str) {
                            let obs = obs_space.to_observation(state);
                            all_observations.insert(id_str, obs);
                        }
                    }

                    let response = Response {
                        obs: all_observations,
                        reward: 0.0,
                        terminated: false,
                        truncated: false,
                        info: serde_json::json!({}),
                    };

                    let response_str = serde_json::to_string(&response).unwrap() + "\n";
                    stream.write_all(response_str.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
            }
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
    agent_state: ResMut<AgentState>,
    mut step_events: EventWriter<StepRequestEvent>,
    mut reset_events: EventWriter<ResetRequestEvent>,
) {
    let cmd = {
        let guard = server.conn.lock().unwrap();
        let stream = guard.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        if reader.read_line(&mut line).is_ok() && !line.is_empty() {
            match serde_json::from_str::<Command>(&line) {
                Ok(cmd) => Some(cmd),
                Err(e) => {
                    error!("Failed to parse command: {}", e);
                    if let Ok(mut stream) = guard.try_clone() {
                        let error_response = serde_json::json!({
                            "error": format!("Invalid command format: {}", e)
                        });
                        let response_str = serde_json::to_string(&error_response).unwrap() + "\n";
                        stream.write_all(response_str.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                    None
                }
            }
        } else {
            None
        }
    };

    if let Some(cmd) = cmd {
        match cmd {
            Command::Initialize { .. } => {
                let debug_response = serde_json::json!({
                    "type": "Initialize",
                    "debug_info": format!("Command was matched as Initialize")
                });
                let response_str = serde_json::to_string(&debug_response).unwrap() + "\n";

                if let Ok(guard) = server.conn.lock() {
                    if let Ok(mut stream) = guard.try_clone() {
                        let response_str = serde_json::to_string(&response_str).unwrap() + "\n";
                        stream.write_all(response_str.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }

                // Ignore after initial setup
                println!("Server already initialized, ignoring command");
            }
            Command::Step { actions } => {
                info!("Step Command Received!");
                info!("actions: {:?}", actions);
                step_events.send(StepRequestEvent { actions });
            }
            Command::Reset { seed } => {
                info!("Reset Command Received with seed: {:?}", seed);

                // Rebuild EnvConfig with new seed if provided
                if let Some(seed_value) = seed {
                    match server.config.rebuild_with_seed(seed_value) {
                        Ok(new_config) => {
                            server.config = new_config;
                            info!("Successfully rebuilt EnvConfig with seed: {}", seed_value);
                        }
                        Err(e) => {
                            error!("Failed to rebuild EnvConfig: {}", e);
                            if let Ok(guard) = server.conn.lock() {
                                if let Ok(mut stream) = guard.try_clone() {
                                    let error_response = serde_json::json!({
                                        "error": format!("Failed to rebuild config with seed {}: {}", seed_value, e)
                                    });
                                    let response_str =
                                        serde_json::to_string(&error_response).unwrap() + "\n";
                                    stream.write_all(response_str.as_bytes()).unwrap();
                                    stream.flush().unwrap();
                                }
                            }
                            return;
                        }
                    }
                }

                // Send reset event with the seed
                info!("Sending ResetRequestEvent with seed: {:?}", seed);
                reset_events.send(ResetRequestEvent { seed });
            }
            Command::Close => {
                // Close Bevy App
                if let Ok(guard) = server.conn.lock() {
                    if let Ok(mut stream) = guard.try_clone() {
                        let response = "Close command called";
                        let response_str = serde_json::to_string(&response).unwrap() + "\n";
                        stream.write_all(response_str.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }

                // TODO: add a Bevy close system hook
            }
        }
    }
}
