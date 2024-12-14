use flyer::components::{
    AircraftConfig, AircraftGeometry, AircraftType, DubinsAircraftConfig, MassModel, PhysicsModel,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::extract_or_default;
use crate::gym::config::errors::ConfigError;
use crate::utils::WithRng;

#[derive(Default)]
pub struct AircraftConfigBuilder {
    ac_type: Option<AircraftType>,
    physics_model: Option<PhysicsModel>,
    mass: Option<MassModel>,
    geometry: Option<AircraftGeometry>,
    rng: Option<ChaCha8Rng>,
}

impl WithRng for AircraftConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.rng = Some(rng);
        self
    }
}

impl AircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn physics_model(mut self, physics_model: PhysicsModel) -> Self {
        self.physics_model = Some(physics_model);
        self
    }

    pub fn aircraft_type(mut self, ac_type: AircraftType) -> Self {
        self.ac_type = Some(ac_type);
        self
    }

    pub fn mass(mut self, mass: MassModel) -> Self {
        self.mass = Some(mass);
        self
    }

    pub fn geometry(mut self, geometry: AircraftGeometry) -> Self {
        self.geometry = Some(geometry);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Ok(Some(physics_model)) = dict.get_item("physics_model") {
            if let Ok(model_str) = physics_model.extract::<&str>() {
                builder = builder.physics_model(match model_str {
                    "full" => PhysicsModel::Full,
                    "simple" => PhysicsModel::Simple,
                    _ => {
                        eprintln!("Warning: Unsupported physics_model value '{}'. Defaulting to 'Simple'.", model_str);
                        PhysicsModel::Simple
                    },
                });
            } else {
                eprintln!("Warning: physics_model is not a valid string. Defaulting to 'Simple'.");
                builder = builder.physics_model(PhysicsModel::Simple);
            }
        } else {
            builder = builder.physics_model(PhysicsModel::Simple);
        }

        match builder.physics_model {
            Some(PhysicsModel::Simple) | None => {
                if let Some(aircraft_config) = dict.get_item("aircraft_config")? {
                    if let Ok(config_dict) = aircraft_config.downcast::<PyDict>() {
                        let default = DubinsAircraftConfig::default();
                        let dubins_config = DubinsAircraftConfig {
                            max_speed: extract_or_default!(
                                config_dict,
                                "max_speed",
                                default.max_speed
                            ),
                            min_speed: extract_or_default!(
                                config_dict,
                                "min_speed",
                                default.min_speed
                            ),
                            acceleration: extract_or_default!(
                                config_dict,
                                "acceleration",
                                default.acceleration
                            ),
                            max_bank_angle: extract_or_default!(
                                config_dict,
                                "max_bank_angle",
                                default.max_bank_angle
                            ),
                            max_turn_rate: extract_or_default!(
                                config_dict,
                                "max_turn_rate",
                                default.max_turn_rate
                            ),
                            max_climb_rate: extract_or_default!(
                                config_dict,
                                "max_climb_rate",
                                default.max_climb_rate
                            ),
                            max_descent_rate: extract_or_default!(
                                config_dict,
                                "max_descent_rate",
                                default.max_descent_rate
                            ),
                        };
                    }
                }
            }
            Some(PhysicsModel::Full) => {
                if let Some(type_str) = dict
                    .get_item("type")?
                    .and_then(|t| t.extract::<String>().ok())
                {
                    if let Some(type_str) = dict
                        .get_item("type")?
                        .and_then(|t| t.extract::<String>().ok())
                    {
                        builder = builder.aircraft_type(match type_str.as_str() {
                            "twin_otter" => AircraftType::TwinOtter,
                            "f4_phantom" => AircraftType::F4Phantom,
                            "generic_transport" => AircraftType::GenericTransport,
                            _ => AircraftType::Custom(type_str),
                        });
                    }
                }

                let ac_type = builder
                    .ac_type
                    .clone()
                    .unwrap_or(AircraftType::GenericTransport);

                let default_config = match ac_type {
                    AircraftType::TwinOtter => AircraftConfig::twin_otter(),
                    AircraftType::F4Phantom => AircraftConfig::f4_phantom(),
                    AircraftType::GenericTransport => AircraftConfig::generic_transport(),
                    AircraftType::Custom(name) => AircraftConfig {
                        ac_type: AircraftType::Custom(name),
                        ..AircraftConfig::default()
                    },
                };

                // Handle mass configuration
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
                            builder = builder.mass(MassModel::new(mass, ixx, iyy, izz, ixz));
                        }
                    }
                }

                // Handle geometry configuration
                if let Some(geom) = dict.get_item("geometry")? {
                    if let Ok(geom_dict) = geom.downcast::<PyDict>() {
                        let geometry = AircraftGeometry::new(
                            geom_dict
                                .get_item("wing_area")?
                                .and_then(|w| w.extract().ok())
                                .unwrap_or(default_config.geometry.wing_area),
                            geom_dict
                                .get_item("wing_span")?
                                .and_then(|w| w.extract().ok())
                                .unwrap_or(default_config.geometry.wing_span),
                            geom_dict
                                .get_item("mac")?
                                .and_then(|m| m.extract().ok())
                                .unwrap_or(default_config.geometry.mac),
                        );
                        builder = builder.geometry(geometry);
                    }
                }
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<AircraftConfig, ConfigError> {
        let _rng = self.rng.unwrap_or_else(|| ChaCha8Rng::from_entropy());

        let mut config = match self.ac_type.unwrap_or(AircraftType::GenericTransport) {
            AircraftType::TwinOtter => AircraftConfig::twin_otter(),
            AircraftType::F4Phantom => AircraftConfig::f4_phantom(),
            AircraftType::GenericTransport => AircraftConfig::generic_transport(),
            AircraftType::Custom(name) => AircraftConfig {
                ac_type: AircraftType::Custom(name),
                ..AircraftConfig::default()
            },
        };

        if let Some(mass) = self.mass {
            config.mass = mass;
        }

        if let Some(geometry) = self.geometry {
            config.geometry = geometry;
        }

        Ok(config)
    }
}
