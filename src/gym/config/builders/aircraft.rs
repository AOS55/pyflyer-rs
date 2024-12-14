use flyer::components::{AircraftConfig, AircraftGeometry, AircraftType, MassModel};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::gym::config::errors::ConfigError;
use crate::utils::WithRng;

#[derive(Default)]
pub struct AircraftConfigBuilder {
    ac_type: Option<AircraftType>,
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
