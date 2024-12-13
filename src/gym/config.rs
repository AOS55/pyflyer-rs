use flyer::{
    components::{AircraftConfig, AircraftGeometry, AircraftType, MassModel, PhysicsModel},
    resources::{
        AtmosphereConfig, AtmosphereType, BiomeConfig, BiomeThresholds, EnvironmentConfig,
        FeatureConfig, HeightNoiseConfig, MoistureNoiseConfig, NoiseConfig, PhysicsConfig,
        TerrainConfig, WindConfig,
    },
    systems::terrain::noise::NoiseLayer,
};
use nalgebra::Vector3;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::*;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    // Time Configuration
    pub max_episode_steps: u32, // Max steps before truncation
    pub steps_per_action: u32,  // Number of simulation steps per action
    pub time_step: f64,         // Size of time step update

    // Aircraft Configuration
    pub aircraft_config: AircraftConfig,
    pub physics_model: PhysicsModel,
    pub physics_config: PhysicsConfig,
    pub environment_config: EnvironmentConfig,

    // Terrain Configuration
    pub terrain_config: TerrainConfig,

    // Terminal conditions
    pub terminal_conditions: TerminalConditions,

    // Reward configuration
    pub reward_weights: Option<RewardWeights>,
}

// Placeholder for now
#[derive(Clone, Debug)]
pub struct RewardWeights {
    pub altitude: f64,
    pub velocity: f64,
    pub attitude: f64,
    pub control_smoothness: f64,
    pub goal_distance: f64,
}

// Placeholder for now
#[derive(Clone, Debug)]
pub struct TerminalConditions {
    pub min_altitude: f64,
    pub max_altitude: f64,
    pub max_velocity: f64,
    pub max_angle: f64,
    pub terrain_collision: bool,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            max_episode_steps: 1000,
            steps_per_action: 4,
            time_step: 1.0 / 60.0,

            aircraft_config: AircraftConfig::default(),
            physics_model: PhysicsModel::Full,
            physics_config: PhysicsConfig::default(),

            terrain_config: TerrainConfig::default(),
            environment_config: EnvironmentConfig::default(),

            reward_weights: None,
            terminal_conditions: TerminalConditions::default(),
        }
    }
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            altitude: 1.0,
            velocity: 1.0,
            attitude: 1.0,
            control_smoothness: 0.5,
            goal_distance: 2.0,
        }
    }
}

impl Default for TerminalConditions {
    fn default() -> Self {
        Self {
            min_altitude: -100.0,  // meters
            max_altitude: 10000.0, // meters
            max_velocity: 200.0,   // m/s
            max_angle: 60.0,       // degrees
            terrain_collision: true,
        }
    }
}

impl EnvConfig {
    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut config = Self::default();
        // Get keys as a Python list and iterate over them

        if let Some(steps) = dict.get_item("max_episode_steps")? {
            config.max_episode_steps = steps.extract()?;
        }
        if let Some(steps_per_action) = dict.get_item("steps_per_action")? {
            config.steps_per_action = steps_per_action.extract()?;
        }
        if let Some(time_step) = dict.get_item("time_step")? {
            config.time_step = time_step.extract()?;
        }

        if let Some(physics) = dict.get_item("physics_model")? {
            config.physics_model = match physics.extract::<&str>()? {
                "simple" => PhysicsModel::Simple,
                "full" => PhysicsModel::Full,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Invalid physics_model. Must be 'simple' or 'full'.",
                    ))
                }
            };
        }

        if let Some(aircraft_dict) = dict.get_item("aircraft_config")? {
            if let Ok(dict) = aircraft_dict.downcast::<PyDict>() {
                config.aircraft_config = parse_aircraft_config(&dict)?;
            }
        }

        if let Some(physics_dict) = dict.get_item("physics_config")? {
            if let Ok(dict) = physics_dict.downcast::<PyDict>() {
                config.physics_config = parse_physics_config(&dict)?;
            }
        }

        if let Some(env_dict) = dict.get_item("environment_config")? {
            if let Ok(dict) = env_dict.downcast::<PyDict>() {
                config.environment_config = parse_environment_config(&dict)?;
            }
        }

        if let Some(terrain_dict) = dict.get_item("terrain_config")? {
            if let Ok(dict) = terrain_dict.downcast::<PyDict>() {
                config.terrain_config = parse_terrain_config(&dict)?;
            }
        }

        if let Some(terminal_dict) = dict.get_item("terminal_conditions")? {
            if let Ok(dict) = terminal_dict.downcast::<PyDict>() {
                config.terminal_conditions = parse_terminal_conditions(&dict)?;
            }
        }

        if let Some(reward_dict) = dict.get_item("reward_weights")? {
            if let Ok(dict) = reward_dict.downcast::<PyDict>() {
                config.reward_weights = Some(parse_reward_weights(&dict)?);
            }
        }

        Ok(config)
    }
}

fn parse_physics_config(dict: &Bound<'_, PyDict>) -> PyResult<PhysicsConfig> {
    let mut config = PhysicsConfig::default();

    if let Some(max_velocity) = dict.get_item("max_velocity")? {
        config.max_velocity = max_velocity.extract()?;
    }
    if let Some(max_angular_velocity) = dict.get_item("max_angular_velocity")? {
        config.max_angular_velocity = max_angular_velocity.extract()?;
    }
    if let Some(timestep) = dict.get_item("timestep")? {
        config.timestep = timestep.extract()?;
    }

    Ok(config)
}

fn parse_aircraft_config(dict: &Bound<'_, PyDict>) -> PyResult<AircraftConfig> {
    let mut config = match dict
        .get_item("type")?
        .and_then(|t| t.extract::<String>().ok())
    {
        Some(type_str) => match type_str.as_str() {
            "twin_otter" => AircraftConfig::twin_otter(),
            "f4_phantom" => AircraftConfig::f4_phantom(),
            "generic_transport" => AircraftConfig::generic_transport(),
            _ => AircraftConfig {
                ac_type: AircraftType::Custom(type_str),
                ..AircraftConfig::default()
            },
        },
        None => AircraftConfig::default(),
    };

    // Only override mass properties if specified
    if let Some(mass) = dict.get_item("mass")? {
        if let Ok(mass_dict) = mass.downcast::<PyDict>() {
            if let (Some(mass), Some(ixx), Some(iyy), Some(izz), Some(ixz)) = (
                mass_dict
                    .get_item("mass")?
                    .and_then(|m| m.extract::<f64>().ok()),
                mass_dict
                    .get_item("ixx")?
                    .and_then(|i| i.extract::<f64>().ok()),
                mass_dict
                    .get_item("iyy")?
                    .and_then(|i| i.extract::<f64>().ok()),
                mass_dict
                    .get_item("izz")?
                    .and_then(|i| i.extract::<f64>().ok()),
                mass_dict
                    .get_item("ixz")?
                    .and_then(|i| i.extract::<f64>().ok()),
            ) {
                config.mass = MassModel::new(mass, ixx, iyy, izz, ixz);
            }
        }
    }

    // Only override geometry if specified
    if let Some(geom) = dict.get_item("geometry")? {
        if let Ok(geom_dict) = geom.downcast::<PyDict>() {
            config.geometry = AircraftGeometry::new(
                geom_dict
                    .get_item("wing_area")?
                    .and_then(|w| w.extract().ok())
                    .unwrap_or(config.geometry.wing_area),
                geom_dict
                    .get_item("wing_span")?
                    .and_then(|w| w.extract().ok())
                    .unwrap_or(config.geometry.wing_span),
                geom_dict
                    .get_item("mac")?
                    .and_then(|m| m.extract().ok())
                    .unwrap_or(config.geometry.mac),
            );
        }
    }

    Ok(config)
}

fn parse_environment_config(dict: &Bound<'_, PyDict>) -> PyResult<EnvironmentConfig> {
    let mut config = EnvironmentConfig::default();

    // Parse wind configuration
    if let Some(wind_dict) = dict.get_item("wind_model_config")? {
        if let Ok(dict) = wind_dict.downcast::<PyDict>() {
            config.wind_model_config = parse_wind_config(&dict)?;
        }
    }

    // Parse atmosphere configuration
    if let Some(atmosphere_dict) = dict.get_item("atmosphere_config")? {
        if let Ok(dict) = atmosphere_dict.downcast::<PyDict>() {
            config.atmosphere_config = parse_atmosphere_config(&dict)?;
        }
    }

    Ok(config)
}

fn parse_wind_config(dict: &Bound<'_, PyDict>) -> PyResult<WindConfig> {
    let wind_type = dict
        .get_item("type")?
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Missing 'type' key in wind_model_config",
            )
        })?
        .extract::<String>()?;

    match wind_type.as_str() {
        "Constant" => {
            let velocity = dict
                .get_item("velocity")?
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Missing 'velocity' for Constant wind",
                    )
                })?
                .extract::<(f64, f64, f64)>()?;
            Ok(WindConfig::Constant {
                velocity: Vector3::new(velocity.0, velocity.1, velocity.2),
            })
        }
        "Logarithmic" => {
            let d = dict
                .get_item("d")?
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let z0 = dict
                .get_item("z0")?
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let u_star = dict
                .get_item("u_star")?
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let bearing = dict
                .get_item("bearing")?
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            Ok(WindConfig::Logarithmic {
                d,
                z0,
                u_star,
                bearing,
            })
        }
        "PowerLaw" => {
            let u_r = dict
                .get_item("u_r")? // Add ? here
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let z_r = dict
                .get_item("z_r")? // Add ? here
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let bearing = dict
                .get_item("bearing")? // Add ? here
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            let alpha = dict
                .get_item("alpha")? // Add ? here
                .and_then(|item| item.extract::<f64>().ok())
                .unwrap_or(0.0);
            Ok(WindConfig::PowerLaw {
                u_r,
                z_r,
                bearing,
                alpha,
            })
        }
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Invalid wind_model_config type",
        )),
    }
}

fn parse_atmosphere_config(dict: &Bound<'_, PyDict>) -> PyResult<AtmosphereConfig> {
    let model_type = dict
        .get_item("model_type")?
        .and_then(|item| item.extract::<String>().ok())
        .map(|s| match s.as_str() {
            "Constant" => AtmosphereType::Constant,
            "Standard" => AtmosphereType::Standard,
            _ => AtmosphereType::Standard,
        })
        .unwrap_or(AtmosphereType::Standard);

    let sea_level_density = dict
        .get_item("sea_level_density")?
        .and_then(|item| item.extract::<f64>().ok())
        .unwrap_or(1.225);

    let sea_level_temperature = dict
        .get_item("sea_level_temperature")?
        .and_then(|item| item.extract::<f64>().ok())
        .unwrap_or(288.15);

    Ok(AtmosphereConfig {
        model_type,
        sea_level_density,
        sea_level_temperature,
    })
}

fn parse_terrain_config(dict: &Bound<'_, PyDict>) -> PyResult<TerrainConfig> {
    let mut config = TerrainConfig::default();

    if let Some(noise_dict) = dict.get_item("noise")? {
        if let Ok(dict) = noise_dict.downcast::<PyDict>() {
            config.noise = parse_noise_config(&dict)?;
        }
    }

    if let Some(biome_dict) = dict.get_item("biome")? {
        if let Ok(dict) = biome_dict.downcast::<PyDict>() {
            config.biome = parse_biome_config(&dict)?;
        }
    }

    if let Some(feature_dict) = dict.get_item("feature")? {
        if let Ok(dict) = feature_dict.downcast::<PyDict>() {
            config.feature = parse_feature_config(&dict)?;
        }
    }

    // if let Some(render_dict) = dict.get_item("render")? {
    //     if let Ok(dict) = render_dict.downcast::<PyDict>() {
    //         config.render = parse_render_config(&dict)?;
    //     }
    // }

    Ok(config)
}

fn parse_noise_config(dict: &Bound<'_, PyDict>) -> PyResult<NoiseConfig> {
    // Changed to reference
    let mut config = NoiseConfig::default();

    if let Some(height_dict) = dict.get_item("height")? {
        if let Ok(dict) = height_dict.downcast::<PyDict>() {
            config.height = parse_height_noise_config(&dict)?;
        }
    }

    if let Some(moisture_dict) = dict.get_item("moisture")? {
        if let Ok(dict) = moisture_dict.downcast::<PyDict>() {
            config.moisture = parse_moisture_noise_config(&dict)?;
        }
    }

    // if let Some(river_dict) = dict
    //     .get_item("river")?
    //     .and_then(|item| item.downcast::<PyDict>().ok())
    // {
    //     config.river = parse_river_noise_config(&river_dict)?;
    // }

    Ok(config)
}

fn parse_height_noise_config(dict: &Bound<'_, PyDict>) -> PyResult<HeightNoiseConfig> {
    // Changed to reference
    let scale = dict
        .get_item("scale")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(800.0);

    let octaves = dict
        .get_item("octaves")?
        .and_then(|item| item.extract::<u32>().ok())
        .unwrap_or(4);

    let persistence = dict
        .get_item("persistence")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.5);

    let lacunarity = dict
        .get_item("lacunarity")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(2.0);

    // This needs to be fixed
    let layers = if let Some(layers_item) = dict.get_item("layers")? {
        if let Ok(layers_list) = layers_item.downcast::<PyList>() {
            layers_list
                .iter()
                .filter_map(|layer| {
                    if let Ok(layer_dict) = layer.downcast::<PyDict>() {
                        parse_noise_layer(&layer_dict).ok()
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Ok(HeightNoiseConfig {
        scale,
        octaves,
        persistence,
        lacunarity,
        layers,
    })
}

// TODO: Make this neater
fn parse_moisture_noise_config(dict: &Bound<'_, PyDict>) -> PyResult<MoistureNoiseConfig> {
    let scale = dict
        .get_item("scale")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(250.0);

    let layers = if let Some(layers_item) = dict.get_item("layers")? {
        if let Ok(layers_list) = layers_item.downcast::<PyList>() {
            layers_list
                .iter()
                .filter_map(|layer| {
                    if let Ok(layer_dict) = layer.downcast::<PyDict>() {
                        parse_noise_layer(&layer_dict).ok()
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Ok(MoistureNoiseConfig { scale, layers })
}

fn parse_noise_layer(dict: &Bound<'_, PyDict>) -> PyResult<NoiseLayer> {
    let scale = dict
        .get_item("scale")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(1.0);

    let amplitude = dict
        .get_item("amplitude")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(1.0);

    let frequency = dict
        .get_item("frequency")?
        .and_then(|item| item.extract::<u32>().ok())
        .unwrap_or(1);

    let weight = dict
        .get_item("weight")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(1.0);

    Ok(NoiseLayer::new(scale, amplitude, frequency).with_weight(weight))
}

fn parse_biome_config(dict: &Bound<'_, PyDict>) -> PyResult<BiomeConfig> {
    let thresholds = match dict.get_item("thresholds")? {
        Some(thresholds_dict) if thresholds_dict.downcast::<PyDict>().is_ok() => {
            parse_biome_thresholds(thresholds_dict.downcast::<PyDict>().unwrap())?
        }
        _ => BiomeThresholds::default(),
    };

    Ok(BiomeConfig { thresholds })
}

fn parse_biome_thresholds(dict: &Bound<'_, PyDict>) -> PyResult<BiomeThresholds> {
    let mut config = BiomeThresholds::default(); // ToDo: Change to use this

    let water = dict
        .get_item("water")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.48);

    let mountain_start = dict
        .get_item("mountain_start")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.75);

    let mountain_width = dict
        .get_item("mountain_width")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.1);

    let beach_width = dict
        .get_item("beach_width")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.025);

    let forest_moisture = dict
        .get_item("forest_moisture")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.95);

    let desert_moisture = dict
        .get_item("desert_moisture")?
        .and_then(|item| item.extract::<f32>().ok())
        .unwrap_or(0.2);

    let field_sizes = dict
        .get_item("field_sizes")?
        .and_then(|item| item.extract::<Vec<f32>>().ok())
        .unwrap_or_else(|| vec![96.0, 128.0, 256.0, 512.0])
        .try_into()
        .unwrap_or([96.0, 128.0, 256.0, 512.0]);

    Ok(BiomeThresholds {
        water,
        mountain_start,
        mountain_width,
        beach_width,
        forest_moisture,
        desert_moisture,
        field_sizes,
    })
}

fn parse_feature_config(dict: &Bound<'_, PyDict>) -> PyResult<FeatureConfig> {
    let mut config = FeatureConfig::default();

    Ok(config)
}

fn parse_terminal_conditions(dict: &Bound<'_, PyDict>) -> PyResult<TerminalConditions> {
    let mut conditions = TerminalConditions::default();

    if let Some(min_alt) = dict.get_item("min_altitude")? {
        conditions.min_altitude = min_alt.extract()?;
    }
    if let Some(max_alt) = dict.get_item("max_altitude")? {
        conditions.max_altitude = max_alt.extract()?;
    }
    if let Some(max_vel) = dict.get_item("max_velocity")? {
        conditions.max_velocity = max_vel.extract()?;
    }
    if let Some(max_angle) = dict.get_item("max_angle")? {
        conditions.max_angle = max_angle.extract()?;
    }
    if let Some(terrain_collision) = dict.get_item("terrain_collision")? {
        conditions.terrain_collision = terrain_collision.extract()?;
    }

    Ok(conditions)
}

fn parse_reward_weights(dict: &Bound<'_, PyDict>) -> PyResult<RewardWeights> {
    let mut weights = RewardWeights::default();

    if let Some(altitude) = dict.get_item("altitude")? {
        weights.altitude = altitude.extract()?;
    }
    if let Some(velocity) = dict.get_item("velocity")? {
        weights.velocity = velocity.extract()?;
    }
    if let Some(attitude) = dict.get_item("attitude")? {
        weights.attitude = attitude.extract()?;
    }
    if let Some(control_smoothness) = dict.get_item("control_smoothness")? {
        weights.control_smoothness = control_smoothness.extract()?;
    }
    if let Some(goal_distance) = dict.get_item("goal_distance")? {
        weights.goal_distance = goal_distance.extract()?;
    }

    Ok(weights)
}
