use clap::{command, value_parser, Arg};
use gilrs::{Event, Gamepad, Gilrs};
use rdev::{simulate, EventType, Key};
use serde::Deserialize;
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Deserialize)]
struct AxisMapping {
    axis: gilrs::Axis,
    high_key: Key,
    low_key: Key,
    threshold: f32,
}

#[derive(Deserialize)]
struct ButtonMapping {
    button: gilrs::Button,
    key: Key,
}

#[derive(Deserialize)]
struct ControllerMapping {
    controller_id: u32,
    axis_mappings: Vec<AxisMapping>,
    button_mappings: Vec<ButtonMapping>,
}

#[derive(Deserialize)]
struct Config {
    controller_mappings: Vec<ControllerMapping>,
}

impl AxisMapping {
    pub fn apply_mapping(
        &self,
        key_state: &mut HashMap<Key, bool>,
        gamepad: &Gamepad,
        verbose: bool,
    ) {
        let v = gamepad
            .axis_data(self.axis)
            .map_or(0.0, |data| data.value());

        match v {
            _ if v < -self.threshold => key_press_once(key_state, self.low_key, verbose),
            _ if v > self.threshold => key_press_once(key_state, self.high_key, verbose),
            _ => {
                key_release_once(key_state, self.low_key, verbose);
                key_release_once(key_state, self.high_key, verbose);
            }
        }
    }
}

impl ButtonMapping {
    pub fn apply_mapping(
        &self,
        key_state: &mut HashMap<Key, bool>,
        gamepad: &Gamepad,
        verbose: bool,
    ) {
        let button_pressed = gamepad
            .button_data(self.button)
            .map_or(0.0, |data| data.value());

        if button_pressed > 0.5 {
            key_press_once(key_state, self.key, verbose)
        } else {
            key_release_once(key_state, self.key, verbose);
        }
    }
}

impl ControllerMapping {
    pub fn apply_mapping(
        &self,
        key_state: &mut HashMap<Key, bool>,
        gamepad: &Gamepad,
        verbose: bool,
    ) {
        let id: usize = gamepad.id().into();
        if self.controller_id != id as u32 {
            return;
        }

        for axis_mapping in &self.axis_mappings {
            axis_mapping.apply_mapping(key_state, gamepad, verbose);
        }

        for button_mapping in &self.button_mappings {
            button_mapping.apply_mapping(key_state, gamepad, verbose);
        }
    }
}

impl Config {
    pub fn apply_mapping(
        &self,
        key_state: &mut HashMap<Key, bool>,
        gamepad: &Gamepad,
        verbose: bool,
    ) {
        for controller_mapping in &self.controller_mappings {
            controller_mapping.apply_mapping(key_state, gamepad, verbose);
        }
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
        while let Some(Event { id, event, time }) = gilrs.next_event_blocking(None) {
            if verbose > 1 {
                println!("{:?} New event from {}: {:?}\n", time, id, event);
            }

            for (_id, gamepad) in gilrs.gamepads() {
                config.apply_mapping(&mut key_state, &gamepad, verbose != 0);
            }
        }
    }
}
