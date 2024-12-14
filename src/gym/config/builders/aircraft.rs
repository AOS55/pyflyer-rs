use flyer::components::{
    AircraftAeroCoefficients, AircraftConfig, AircraftGeometry, AircraftType, DubinsAircraftConfig,
    FullAircraftConfig, MassModel,
};

use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand_chacha::ChaCha8Rng;

use crate::gym::config::builders::RandomStartPosConfigBuilder;
use crate::gym::config::errors::ConfigError;
use crate::utils::WithRng;

// Simplified to just one trait for building aircraft
pub trait AircraftBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError>;
}

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

#[derive(Default, Debug)]
pub struct DubinsAircraftConfigBuilder {
    name: Option<String>,
    max_speed: Option<f64>,
    min_speed: Option<f64>,
    acceleration: Option<f64>,
    max_bank_angle: Option<f64>,
    max_turn_rate: Option<f64>,
    max_climb_rate: Option<f64>,
    max_descent_rate: Option<f64>,
    random_start_config: RandomStartPosConfigBuilder,
    rng: Option<ChaCha8Rng>,
}

#[derive(Default, Debug)]
pub struct FullAircraftConfigBuilder {
    pub name: Option<String>,
    pub ac_type: Option<AircraftType>,
    pub mass: Option<MassModel>,
    pub geometry: Option<AircraftGeometry>,
    pub aero_coef: Option<AircraftAeroCoefficients>,
    pub rng: Option<ChaCha8Rng>,
}

impl DubinsAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();
        builder.name = dict.get_item("name")?.and_then(|v| v.extract().ok());

        if let Ok(Some(config)) = dict.get_item("config") {
            if let Ok(config_dict) = config.downcast::<PyDict>() {
                builder.max_speed = config_dict
                    .get_item("max_speed")?
                    .and_then(|v| v.extract().ok());
                builder.min_speed = config_dict
                    .get_item("min_speed")?
                    .and_then(|v| v.extract().ok());
                builder.acceleration = config_dict
                    .get_item("acceleration")?
                    .and_then(|v| v.extract().ok());
                builder.max_bank_angle = config_dict
                    .get_item("max_bank_angle")?
                    .and_then(|v| v.extract().ok());
                builder.max_turn_rate = config_dict
                    .get_item("max_turn_rate")?
                    .and_then(|v| v.extract().ok());
                builder.max_climb_rate = config_dict
                    .get_item("max_climb_rate")?
                    .and_then(|v| v.extract().ok());
                builder.max_descent_rate = config_dict
                    .get_item("max_descent_rate")?
                    .and_then(|v| v.extract().ok());
                builder.random_start_config = RandomStartPosConfigBuilder::from_pydict(dict)?;
            }
        }

        Ok(builder)
    }
}

impl AircraftBuilder for DubinsAircraftConfigBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        let default_config = DubinsAircraftConfig::default();

        let random_start_config = if let Some(rng) = &self.rng {
            self.random_start_config
                .clone()
                .with_rng(rng.clone())
                .build()
        } else {
            self.random_start_config.clone().build()
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
            random_start_config: Some(random_start_config),
        }))
    }
}

impl WithRng for DubinsAircraftConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.rng = Some(rng);
        self
    }
}

impl FullAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();
        builder.name = dict
            .get_item("name")?
            .and_then(|v| v.extract::<String>().ok());

        if let Ok(Some(config)) = dict.get_item("config") {
            if let Ok(config_dict) = config.downcast::<PyDict>() {
                // Aircraft type
                if let Some(type_str) = config_dict
                    .get_item("ac_type")?
                    .and_then(|t| t.extract::<String>().ok())
                {
                    builder.ac_type = Some(match type_str.as_str() {
                        "twin_otter" => AircraftType::TwinOtter,
                        "f4_phantom" => AircraftType::F4Phantom,
                        _ => AircraftType::GenericTransport,
                    });
                }

                // Parse mass and geometry configurations
                if let Ok(Some(mass_dict)) = config_dict.get_item("mass") {
                    if let Ok(mass_dict) = mass_dict.downcast::<PyDict>() {
                        builder.mass = parse_mass_dict(&mass_dict)?;
                    }
                }

                if let Ok(Some(geom_dict)) = config_dict.get_item("geometry") {
                    if let Ok(geom_dict) = geom_dict.downcast::<PyDict>() {
                        builder.geometry = parse_geometry_dict(&geom_dict)?;
                    }
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

        let config = FullAircraftConfig {
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
        };

        Ok(AircraftConfig::Full(config))
    }
}

impl WithRng for FullAircraftConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.rng = Some(rng);
        self
    }
}

// Helper functions to parse configuration
fn parse_mass_dict(mass_dict: &Bound<'_, PyDict>) -> PyResult<Option<MassModel>> {
    let mass = mass_dict.get_item("mass")?.and_then(|v| v.extract().ok());
    let ixx = mass_dict.get_item("ixx")?.and_then(|v| v.extract().ok());
    let iyy = mass_dict.get_item("iyy")?.and_then(|v| v.extract().ok());
    let izz = mass_dict.get_item("izz")?.and_then(|v| v.extract().ok());
    let ixz = mass_dict.get_item("ixz")?.and_then(|v| v.extract().ok());

    match (mass, ixx, iyy, izz, ixz) {
        (Some(mass), Some(ixx), Some(iyy), Some(izz), Some(ixz)) => {
            Ok(Some(MassModel::new(mass, ixx, iyy, izz, ixz)))
        }
        _ => Ok(None),
    }
}

fn parse_geometry_dict(geom_dict: &Bound<'_, PyDict>) -> PyResult<Option<AircraftGeometry>> {
    let wing_area = geom_dict
        .get_item("wing_area")?
        .and_then(|v| v.extract().ok());
    let wing_span = geom_dict
        .get_item("wing_span")?
        .and_then(|v| v.extract().ok());
    let mac = geom_dict.get_item("mac")?.and_then(|v| v.extract().ok());

    match (wing_area, wing_span, mac) {
        (Some(wing_area), Some(wing_span), Some(mac)) => {
            Ok(Some(AircraftGeometry::new(wing_area, wing_span, mac)))
        }
        _ => Ok(None),
    }
}

fn get_default_full_config(ac_type: &AircraftType, name: Option<String>) -> FullAircraftConfig {
    match ac_type {
        AircraftType::TwinOtter => FullAircraftConfig {
            name: name.unwrap_or_else(|| "unnamed_twin_otter".to_string()),
            ac_type: AircraftType::TwinOtter,
            mass: MassModel::twin_otter(),
            geometry: AircraftGeometry::twin_otter(),
            aero_coef: AircraftAeroCoefficients::twin_otter(),
        },
        AircraftType::F4Phantom => FullAircraftConfig {
            name: name.unwrap_or_else(|| "unnamed_f4".to_string()),
            ac_type: AircraftType::F4Phantom,
            mass: MassModel::f4_phantom(),
            geometry: AircraftGeometry::f4_phantom(),
            aero_coef: AircraftAeroCoefficients::f4_phantom(),
        },
        _ => FullAircraftConfig {
            name: name.unwrap_or_else(|| "unnamed_generic".to_string()),
            ac_type: AircraftType::GenericTransport,
            mass: MassModel::generic_transport(),
            geometry: AircraftGeometry::generic_transport(),
            aero_coef: AircraftAeroCoefficients::generic_transport(),
        },
    }
}

pub fn create_aircraft_builder(
    dict: &Bound<'_, PyDict>,
) -> Result<AircraftBuilderEnum, ConfigError> {
    let aircraft_type: String = dict
        .get_item("type")
        .map_err(|_| ConfigError::MissingRequired("aircraft type".into()))?
        .ok_or_else(|| ConfigError::MissingRequired("aircraft type".into()))?
        .extract()?;

    match aircraft_type.as_str() {
        "dubins" => Ok(AircraftBuilderEnum::Dubins(
            DubinsAircraftConfigBuilder::from_pydict(dict)?,
        )),
        "full" => Ok(AircraftBuilderEnum::Full(
            FullAircraftConfigBuilder::from_pydict(dict)?,
        )),
        _ => Err(ConfigError::InvalidAircraftType(aircraft_type)),
    }
}
