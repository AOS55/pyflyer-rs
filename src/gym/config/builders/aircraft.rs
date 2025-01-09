use bevy::prelude::*;
use flyer::{
    components::{
        AircraftAeroCoefficients, AircraftConfig, AircraftGeometry, AircraftType,
        DubinsAircraftConfig, FullAircraftConfig, MassModel,
    },
    utils::WithRng,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use rand_chacha::ChaCha8Rng;

use crate::gym::config::builders::{
    ActionSpaceBuilder, ObservationSpaceBuilder, RandomStartConfigBuilder,
};
use crate::gym::config::errors::ConfigError;
use crate::gym::obs::ContinuousObservationSpace;
use crate::gym::{ActionSpace, ObservationSpace};

pub struct AircraftAgentBuilder {
    pub aircraft_builder: AircraftBuilderEnum,
    pub observation_builder: ObservationSpaceBuilder,
    pub action_builder: ActionSpaceBuilder,
}

// Simplified to just one trait for building aircraft
pub trait AircraftBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError>;
}

#[derive(Debug, Clone)]
pub enum AircraftBuilderEnum {
    Dubins(DubinsAircraftConfigBuilder),
    Full(FullAircraftConfigBuilder),
}

impl AircraftBuilder for AircraftBuilderEnum {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        match self {
            AircraftBuilderEnum::Dubins(builder) => builder.build(),
            AircraftBuilderEnum::Full(builder) => builder.build(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftConfigBuilder {
    pub name: Option<String>,
    pub max_speed: Option<f64>,
    pub min_speed: Option<f64>,
    pub acceleration: Option<f64>,
    pub max_bank_angle: Option<f64>,
    pub max_turn_rate: Option<f64>,
    pub max_climb_rate: Option<f64>,
    pub max_descent_rate: Option<f64>,
    pub random_start_config: Option<RandomStartConfigBuilder>,
    pub seed: Option<u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FullAircraftConfigBuilder {
    pub name: Option<String>,
    pub ac_type: Option<AircraftType>,
    pub mass: Option<MassModel>,
    pub geometry: Option<AircraftGeometry>,
    pub aero_coef: Option<AircraftAeroCoefficients>,
    #[serde(skip)]
    pub rng: Option<ChaCha8Rng>,
}

impl DubinsAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value, seed: u64) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        builder.name = value.get("name").and_then(|v| v.as_str()).map(String::from);

        if let Some(config) = value.get("config") {
            builder.max_speed = config.get("max_speed").and_then(|v| v.as_f64());
            builder.min_speed = config.get("min_speed").and_then(|v| v.as_f64());
            builder.acceleration = config.get("acceleration").and_then(|v| v.as_f64());
            builder.max_bank_angle = config.get("max_bank_angle").and_then(|v| v.as_f64());
            builder.max_turn_rate = config.get("max_turn_rate").and_then(|v| v.as_f64());
            builder.max_climb_rate = config.get("max_climb_rate").and_then(|v| v.as_f64());
            builder.max_descent_rate = config.get("max_descent_rate").and_then(|v| v.as_f64());
        }

        // Create RandomStartConfigBuilder with either provided config or defaults
        let random_start_builder = if let Some(random_start) = value.get("random_start") {
            RandomStartConfigBuilder::from_json(random_start, seed)?
        } else {
            // Create default builder but with the provided seed
            let mut default_builder = RandomStartConfigBuilder::new();
            default_builder.seed = Some(seed);
            default_builder
        };

        builder.random_start_config = Some(random_start_builder);
        Ok(builder)
    }
}

impl AircraftBuilder for DubinsAircraftConfigBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        info!("Building DubinsAircraftConfig");
        let default_config = DubinsAircraftConfig::default();

        let random_start_config = match (&self.random_start_config, self.seed) {
            (Some(config), Some(seed)) => {
                info!("Using master seed {} for random_start_config", seed);
                Some(config.clone().build_with_seed(seed))
            }
            (Some(config), None) => {
                info!("No seed provided, using default");
                Some(config.clone().build())
            }
            (None, _) => {
                info!("No random start config provided");
                None
            }
        };

        Ok(AircraftConfig::Dubins(DubinsAircraftConfig {
            name: self
                .name
                .clone()
                .unwrap_or_else(|| "unnamed_dubins".to_string()),
            max_speed: self.max_speed.unwrap_or(default_config.max_speed),
            min_speed: self.min_speed.unwrap_or(default_config.min_speed),
            acceleration: self.acceleration.unwrap_or(default_config.acceleration),
            max_bank_angle: self.max_bank_angle.unwrap_or(default_config.max_bank_angle),
            max_turn_rate: self.max_turn_rate.unwrap_or(default_config.max_turn_rate),
            max_climb_rate: self.max_climb_rate.unwrap_or(default_config.max_climb_rate),
            max_descent_rate: self
                .max_descent_rate
                .unwrap_or(default_config.max_descent_rate),
            random_start_config,
        }))
    }
}

impl FullAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        builder.name = value.get("name").and_then(|v| v.as_str()).map(String::from);

        if let Some(config) = value.get("config") {
            // Aircraft type
            if let Some(type_str) = config.get("ac_type").and_then(|t| t.as_str()) {
                builder.ac_type = Some(match type_str {
                    "twin_otter" => AircraftType::TwinOtter,
                    "f4_phantom" => AircraftType::F4Phantom,
                    _ => AircraftType::GenericTransport,
                });
            }

            // Parse mass configuration
            if let Some(mass_config) = config.get("mass") {
                builder.mass = parse_mass_json(mass_config)?;
            }

            // Parse geometry configuration
            if let Some(geom_config) = config.get("geometry") {
                builder.geometry = parse_geometry_json(geom_config)?;
            }

            // Handle aero coefficients based on aircraft type
            builder.aero_coef = Some(
                match builder
                    .ac_type
                    .as_ref()
                    .unwrap_or(&AircraftType::GenericTransport)
                {
                    AircraftType::TwinOtter => AircraftAeroCoefficients::twin_otter(),
                    AircraftType::F4Phantom => AircraftAeroCoefficients::f4_phantom(),
                    _ => AircraftAeroCoefficients::generic_transport(),
                },
            );
        }

        Ok(builder)
    }
}

impl AircraftBuilder for FullAircraftConfigBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        let ac_type = self
            .ac_type
            .clone()
            .unwrap_or(AircraftType::GenericTransport);
        let name = self.name.clone().unwrap_or_else(|| {
            format!(
                "unnamed_{}",
                match ac_type {
                    AircraftType::TwinOtter => "twin_otter",
                    AircraftType::F4Phantom => "f4",
                    _ => "generic",
                }
            )
        });

        Ok(AircraftConfig::Full(FullAircraftConfig {
            name,
            ac_type: ac_type.clone(),
            mass: self.mass.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => MassModel::twin_otter(),
                AircraftType::F4Phantom => MassModel::f4_phantom(),
                _ => MassModel::generic_transport(),
            }),
            geometry: self.geometry.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => AircraftGeometry::twin_otter(),
                AircraftType::F4Phantom => AircraftGeometry::f4_phantom(),
                _ => AircraftGeometry::generic_transport(),
            }),
            aero_coef: self.aero_coef.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => AircraftAeroCoefficients::twin_otter(),
                AircraftType::F4Phantom => AircraftAeroCoefficients::f4_phantom(),
                _ => AircraftAeroCoefficients::generic_transport(),
            }),
        }))
    }
}

// Helper functions to parse configuration
fn parse_mass_json(value: &Value) -> Result<Option<MassModel>, ConfigError> {
    let mass = value.get("mass").and_then(|v| v.as_f64());
    let ixx = value.get("ixx").and_then(|v| v.as_f64());
    let iyy = value.get("iyy").and_then(|v| v.as_f64());
    let izz = value.get("izz").and_then(|v| v.as_f64());
    let ixz = value.get("ixz").and_then(|v| v.as_f64());

    match (mass, ixx, iyy, izz, ixz) {
        (Some(mass), Some(ixx), Some(iyy), Some(izz), Some(ixz)) => {
            Ok(Some(MassModel::new(mass, ixx, iyy, izz, ixz)))
        }
        _ => Ok(None),
    }
}

fn parse_geometry_json(value: &Value) -> Result<Option<AircraftGeometry>, ConfigError> {
    let wing_area = value.get("wing_area").and_then(|v| v.as_f64());
    let wing_span = value.get("wing_span").and_then(|v| v.as_f64());
    let mac = value.get("mac").and_then(|v| v.as_f64());

    match (wing_area, wing_span, mac) {
        (Some(wing_area), Some(wing_span), Some(mac)) => {
            Ok(Some(AircraftGeometry::new(wing_area, wing_span, mac)))
        }
        _ => Ok(None),
    }
}

impl WithRng for DubinsAircraftConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        info!("Setting seed for Dubins aircraft config from RNG");
        let new_seed = rng.get_seed()[0] as u64;
        self.seed = Some(new_seed);
        self
    }
}

impl WithRng for FullAircraftConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.rng = Some(rng);
        self
    }
}

impl WithRng for AircraftBuilderEnum {
    fn with_rng(self, rng: ChaCha8Rng) -> Self {
        match self {
            AircraftBuilderEnum::Dubins(builder) => {
                AircraftBuilderEnum::Dubins(builder.with_rng(rng))
            }
            AircraftBuilderEnum::Full(builder) => AircraftBuilderEnum::Full(builder.with_rng(rng)),
        }
    }
}

pub fn create_aircraft_builder(
    value: &Value,
    seed: u64,
) -> Result<AircraftAgentBuilder, ConfigError> {
    let aircraft_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("aircraft type".into()))?;

    let action_type = value
        .get("action_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("action type".into()))?;

    let observation_type = value
        .get("observation_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("observation type".into()))?;

    let aircraft_builder = match aircraft_type {
        "dubins" => {
            let builder = DubinsAircraftConfigBuilder::from_json(value, seed)?;
            AircraftBuilderEnum::Dubins(builder)
        }
        "full" => {
            let builder = FullAircraftConfigBuilder::from_json(value)?;
            AircraftBuilderEnum::Full(builder)
        }
        _ => return Err(ConfigError::InvalidAircraftType(aircraft_type.to_string())),
    };

    let action_builder = match action_type {
        // No differnce between Continuous and Discrete anymore, could simplify
        "Continuous" => match aircraft_type {
            "dubins" => ActionSpaceBuilder::new().act_space(ActionSpace::new_continuous_dubins()),
            "full" => ActionSpaceBuilder::new().act_space(ActionSpace::new_continuous_full()),
            _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
        },
        "Discrete" => match aircraft_type {
            "dubins" => ActionSpaceBuilder::new().act_space(ActionSpace::new_discrete_dubins()),
            "full" => ActionSpaceBuilder::new().act_space(ActionSpace::new_discrete_full()),
            _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
        },
        _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
    };

    let observation_builder = match observation_type {
        "Continuous" => match aircraft_type {
            "dubins" => ObservationSpaceBuilder::new().obs_space(ObservationSpace::Continuous(
                ContinuousObservationSpace::DubinsAircraft,
            )),
            "full" => ObservationSpaceBuilder::new().obs_space(ObservationSpace::Continuous(
                ContinuousObservationSpace::FullAircraft,
            )),
            _ => return Err(ConfigError::InvalidAircraftType(aircraft_type.to_string())),
        },
        _ => {
            return Err(ConfigError::InvalidObservationType(
                observation_type.to_string(),
            ))
        }
    };

    Ok(AircraftAgentBuilder {
        aircraft_builder,
        observation_builder,
        action_builder,
    })
}
