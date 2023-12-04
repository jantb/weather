use std::fs;

use bevy::app::{App, Startup};
use bevy::asset::Handle;
use bevy::prelude::{AssetServer, Color, Commands, Component, Image, Plugin, Res, Sprite, SpriteBundle, Transform};
use bevy::utils::default;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_assets);
    }
}

fn spawn_assets(
    mut cmd: Commands,
    server: Res<AssetServer>,
) {
    let paths = fs::read_dir("assets/png/").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let filename = path.file_name().unwrap().to_string_lossy();
        let partlycloudy_night: Handle<Image> = server.load(format!("png/{}", filename));
        let filename: Vec<_> = filename.split(".").collect();
        cmd.spawn((ImageName { name: filename[0].to_string() }, SpriteBundle {
            sprite: Sprite {
                // Alpha channel of the color controls transparency.
                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            texture: partlycloudy_night,
            ..default()
        }));
    }
}


#[derive(Component)]
pub struct ImageName {
    pub name: String,
}
