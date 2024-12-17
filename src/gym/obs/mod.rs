mod builder;
pub mod dubins;

pub use builder::ObservationSpaceBuilder;

#[derive(Copy, Debug, Clone)]
pub enum ObservationSpace {
    ContinuousDubinsObs,
}

impl Default for ObservationSpace {
    fn default() -> Self {
        ObservationSpace::ContinuousDubinsObs
    }
}
