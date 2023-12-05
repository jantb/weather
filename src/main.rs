use std::thread::sleep;
use std::time::Duration;

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, RequestRedraw, WindowResolution};
use bevy::winit::WinitSettings;
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
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, (setup, hide_cursor))
        .add_systems(Update, (read_stream, spawn_text))
        .add_plugins(CameraPlugin)
        .add_plugins(RenderPlugin)
        .run();
}

fn hide_cursor(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = false;
}

fn setup(mut commands: Commands) {
    let (tx, rx) = bounded::<State>(10);
    std::thread::spawn(move || {
        loop {
            let client = match reqwest::blocking::Client::builder()
                .user_agent("Something unique to me")
                .build()
            {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("Error creating client: {}", err);
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            let resp = match client.get("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=59.88369&lon=10.80548&altitude=166").send() {
                Ok(ok) => ok,
                Err(err) => {
                    eprintln!("Request error: {}", err);
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            let string = match resp.text() {
                Ok(ok) => ok,
                Err(err) => {
                    eprintln!("Response text error: {}", err);
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            let p: Root = match serde_json::from_str(string.as_str()) {
                Ok(parsed) => parsed,
                Err(err) => {
                    eprintln!("JSON parsing error: {}", err);
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            let x = match p.properties.timeseries.first() {
                Some(x) => x,
                None => {
                    eprintln!("No timeseries found");
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            let temperature_now = x.data.instant.details.air_temperature as f32;

            let icon_now = match x.data.next_1_hours.clone().map(|data| data.summary.symbol_code) {
                Some(icon) => icon,
                None => {
                    eprintln!("Next 1-hour forecast not found");
                    sleep(Duration::from_secs(60));
                    continue;
                }
            };

            // Send the state through the channel
            if let Err(err) = tx.send(State { temperature_now, icon_now }) {
                eprintln!("Error sending state through channel: {}", err);
            }

            // Sleep for 60 seconds
            sleep(Duration::from_secs(60));
        }
    });
    let text_style = TextStyle {
        font_size: 200.0,
        color: Color::WHITE,
        ..default()
    };
    commands.spawn(Text2dBundle {
        text: Text::from_section("No Value", text_style.clone())
            .with_alignment(TextAlignment::Center),
        transform: Transform::from_xyz(0.0, 100.0, 0.0),
        ..default()
    });

    commands.insert_resource(StreamReceiver(rx));
}

fn read_stream(receiver: Res<StreamReceiver>, mut events: EventWriter<StreamEvent>, mut event: EventWriter<RequestRedraw>) {
    for from_stream in receiver.try_iter() {
        events.send(StreamEvent(from_stream));
        event.send(RequestRedraw)
    }
}

fn spawn_text(mut reader: EventReader<StreamEvent>, mut query: Query<(&mut Sprite, &ImageName), With<ImageName>>, mut query_text: Query<&mut Text, With<Text>>) {
    for event in reader.read() {
        query_text.single_mut().sections.first_mut().unwrap().value = event.0.temperature_now.to_string();
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