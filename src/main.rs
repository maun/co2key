use clap::{command, value_parser, Arg};
use gilrs::{Axis, Event, Gilrs};
use rdev::{simulate, EventType, Key};
use serde::Deserialize;
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Deserialize)]
struct AxisCfg {
    axis: gilrs::Axis,
    high_key: Key,
    low_key: Key,
    threshold: f32,
}

#[derive(Deserialize)]
struct ButtonCfg {
    button: gilrs::Button,
    key: Key,
}

#[derive(Deserialize)]
struct ControllerCfg {
    axes: Vec<AxisCfg>,
    buttons: Vec<ButtonCfg>,
}

#[derive(Deserialize)]
struct Config {
    controllers: Vec<ControllerCfg>,
}

impl AxisCfg {
    pub fn handle_event(
        &self,
        axis: Axis,
        axis_value: f32,
        key_state: &mut HashMap<Key, bool>,
        verbose: bool,
    ) {
        if axis != self.axis {
            return;
        }

        match axis_value {
            _ if axis_value < -self.threshold => key_press_once(key_state, self.low_key, verbose),
            _ if axis_value > self.threshold => key_press_once(key_state, self.high_key, verbose),
            _ => {
                key_release_once(key_state, self.low_key, verbose);
                key_release_once(key_state, self.high_key, verbose);
            }
        }
    }
}

impl ControllerCfg {
    pub fn handle_event(&self, event: Event, key_state: &mut HashMap<Key, bool>, verbose: bool) {
        match event.event {
            gilrs::EventType::AxisChanged(axis, axis_value, _) => {
                for axis_mapping in &self.axes {
                    axis_mapping.handle_event(axis, axis_value, key_state, verbose);
                }
            }
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(mapping) = self.buttons.iter().find(|m| m.button == button) {
                    let _ = simulate(&EventType::KeyPress(mapping.key));
                    if verbose {
                        println!("\nSimulated key press {:?}", mapping.key);
                    }
                }
            }
            //gilrs::EventType::ButtonRepeated(button, _) => todo!(),
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(mapping) = self.buttons.iter().find(|m| m.button == button) {
                    let _ = simulate(&EventType::KeyRelease(mapping.key));
                    if verbose {
                        println!("\nSimulated key release {:?}", mapping.key);
                    }
                }
            }
            //gilrs::EventType::ButtonChanged(_, _, _) => (),
            _ => (),
        }
    }
}

impl Config {
    pub fn handle_event(
        &self,
        event: Event,
        key_state: &mut HashMap<Key, bool>,
        gamepad_idx: usize,
        verbose: bool,
    ) {
        if let Some(mapping) = self.controllers.get(gamepad_idx) {
            mapping.handle_event(event, key_state, verbose);
        };
    }
}

fn read_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = from_reader(reader)?;
    Ok(config)
}

fn key_press_once(key_state: &mut HashMap<Key, bool>, key: Key, verbose: bool) {
    let is_down = key_state.entry(key).or_insert(false);
    if !*is_down {
        let _ = simulate(&EventType::KeyPress(key));
        if verbose {
            println!("\nSimulated key press {:?}", key);
        }
        *is_down = true
    }
}

fn key_release_once(key_state: &mut HashMap<Key, bool>, key: Key, verbose: bool) {
    let is_down = key_state.entry(key).or_insert(false);
    if *is_down {
        let _ = simulate(&EventType::KeyRelease(key));
        if verbose {
            println!("\nSimulated key release {:?}", key);
        }
        *is_down = false
    }
}

fn main() {
    let matches = command!()
        .arg(
            Arg::new("config")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
        .get_matches();

    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let config = match read_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Could not parse config file. Error:\n\n");
            println!("{e}");
            return;
        }
    };

    let verbose = matches.get_count("verbose");

    let mut key_state = HashMap::<Key, bool>::new();
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(event) = gilrs.next_event_blocking(None) {
            if verbose > 1 {
                println!(
                    "{:?} New event from {}: {:?}\n",
                    event.time, event.id, event.event
                );
            }

            // match gamepad_id with index of configured a
            let gamepad_idx = match gilrs.gamepads().zip(0usize..).find(|g| g.0 .0 == event.id) {
                Some((_, idx)) => idx,
                _ => continue,
            };

            config.handle_event(event, &mut key_state, gamepad_idx, verbose != 0);
        }
    }
}
