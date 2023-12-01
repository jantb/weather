use bevy::app::{App, Startup, Update};
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Commands, Input, KeyCode, Plugin, Query, Res, Window};
use bevy::window::WindowMode;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, print_monitor_size);
        app.add_systems(Update, toggle_fullscreen);
    }
}

fn print_monitor_size(
    mut cmd: Commands,
) {
    let mut camera2d_bundle = Camera2dBundle::default();
    cmd.spawn(camera2d_bundle);
}


fn toggle_fullscreen(input: Res<Input<KeyCode>>,
                     mut windows: Query<&mut Window>) {
    if input.just_pressed(KeyCode::F) {
        let mut window = windows.single_mut();
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen,
            _ => WindowMode::Windowed,
        };
    }
}
