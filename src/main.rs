use std::thread::sleep;
use std::time::Duration;

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use crossbeam::channel::{bounded, Receiver};

use crate::camera_plugin::CameraPlugin;
use crate::render_plugin::{ImageName, RenderPlugin};
use crate::serde::Root;

mod serde;
mod camera_plugin;
mod render_plugin;

#[derive(Resource)]
struct State {
    temperature_now: f32,
    icon_now: String,
}

fn main() {
//https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=59.88369&lon=10.80548&altitude=166

    App::new()
        //  .insert_resource(state)
        .add_event::<StreamEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "".into(),
                resolution: WindowResolution::new(1920., 1200.),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (read_stream, spawn_text))
        .add_plugins(CameraPlugin)
        .add_plugins(RenderPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    let (tx, rx) = bounded::<State>(10);
    std::thread::spawn(move || {
        loop {
            let client = reqwest::blocking::Client::builder()
                .user_agent("Something unique to me")
                .build().unwrap();

            let resp = client.get("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=59.88369&lon=10.80548&altitude=166").send().unwrap();
            let string = resp.text().unwrap();
            let p: Root = serde_json::from_str(string.as_str()).unwrap();

            let x = p.properties.timeseries.first().unwrap();

            tx.send(State { temperature_now: x.data.instant.details.air_temperature as f32, icon_now: x.data.next_1_hours.clone().unwrap().summary.symbol_code }).unwrap();
            sleep(Duration::from_secs(60))
        }
    });

    commands.insert_resource(StreamReceiver(rx));
}

fn read_stream(receiver: Res<StreamReceiver>, mut events: EventWriter<StreamEvent>) {
    for from_stream in receiver.try_iter() {
        events.send(StreamEvent(from_stream));
    }
}

fn spawn_text(mut commands: Commands, mut reader: EventReader<StreamEvent>, mut query: Query<(&mut Sprite, &ImageName), With<ImageName>>) {
    let text_style = TextStyle {
        font_size: 200.0,
        color: Color::WHITE,
        ..default()
    };

    for (per_frame, event) in reader.read().enumerate() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(event.0.temperature_now.to_string(), text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::from_xyz(per_frame as f32 * 100.0, 300.0, 0.0),
            ..default()
        });
        for (mut sprite, image_name) in query.iter_mut() {
            if image_name.name == event.0.icon_now {
                sprite.color = sprite.color.with_a(1.0).clone();
            } else {
                sprite.color = sprite.color.with_a(0.0).clone();
            }
        }
    }
}

#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<State>);

#[derive(Event)]
struct StreamEvent(State);