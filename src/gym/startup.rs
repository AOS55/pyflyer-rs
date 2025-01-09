use bevy::prelude::*;
use flyer::{
    plugins::{
        add_aircraft_plugin, AgentPlugin, CameraPlugin, HeadlessPlugin, StartupSequencePlugin,
        TerrainPlugin, TransformationPlugin, UpdateSequencePlugin,
    },
    resources::{RenderMode, UpdateControlPlugin},
    systems::camera_follow_system,
};

use crate::gym::EnvConfig;

pub fn setup_app(mut app: App, config: EnvConfig, asset_path: String) -> App {
    app.add_plugins((StartupSequencePlugin, UpdateSequencePlugin));

    app.add_plugins((
        TransformationPlugin::new(1.0),
        AgentPlugin::new(config.agent_config),
        UpdateControlPlugin,
    ));

    for aircraft_config in config.aircraft_configs.iter() {
        add_aircraft_plugin(&mut app, aircraft_config.1.clone());
    }

    println!("mode: {:?}", config.agent_config.mode);
    // TODO: sort out camera and render setup
    match config.agent_config.mode {
        RenderMode::Human => {
            println!("Running Human Mode");
            app.add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "FlyerEnv".into(),
                            resolution: (
                                config.agent_config.render_width,
                                config.agent_config.render_height,
                            )
                                .into(),
                            ..default()
                        }),
                        ..default()
                    })
                    .set(AssetPlugin {
                        file_path: asset_path,
                        ..default()
                    }),
            );
            app.add_plugins(CameraPlugin);
        }
        RenderMode::RGBArray => {
            println!("Running RGBArray Mode");
            app.add_plugins(HeadlessPlugin::new(
                config.agent_config.render_width as u32,
                config.agent_config.render_height as u32,
                asset_path,
            ))
            .add_systems(FixedUpdate, camera_follow_system);
            // .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        }
    }

    // app.add_plugins(CameraPlugin);
    app.add_plugins(TerrainPlugin::new());
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

    app
}
