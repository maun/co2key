use gilrs::{Event, Gamepad, Gilrs};
use rdev::{simulate, EventType, Key};
use std::collections::HashMap;

struct AxisMapping {
    axis: gilrs::Axis,
    high_key: Key,
    low_key: Key,
    threshold: f32,
}

struct ButtonMapping {
    button: gilrs::Button,
    key: Key,
}

struct ControllerMapping {
    controller_id: u32,
    axis_mappings: Vec<AxisMapping>,
    button_mappings: Vec<ButtonMapping>,
}

struct Config {
    controller_mappings: Vec<ControllerMapping>,
}

impl AxisMapping {
    pub fn apply_mapping(&self, key_state: &mut HashMap<Key, bool>, gamepad: &Gamepad) {
        let v = gamepad
            .axis_data(self.axis)
            .map_or(0.0, |data| data.value());

        match v {
            _ if v < -self.threshold => key_press_once(key_state, self.low_key),
            _ if v > self.threshold => key_press_once(key_state, self.high_key),
            _ => {
                key_release_once(key_state, self.low_key);
                key_release_once(key_state, self.high_key);
            }
        }
    }
}

impl ButtonMapping {
    pub fn apply_mapping(&self, key_state: &mut HashMap<Key, bool>, gamepad: &Gamepad) {
        let button_pressed = gamepad
            .button_data(self.button)
            .map_or(0.0, |data| data.value());

        if button_pressed > 0.5 {
            key_press_once(key_state, self.key)
        } else {
            key_release_once(key_state, self.key);
        }
    }
}

impl ControllerMapping {
    pub fn apply_mapping(&self, key_state: &mut HashMap<Key, bool>, gamepad: &Gamepad) {
        let id: usize = gamepad.id().into();
        if self.controller_id != id as u32 {
            return;
        }

        for axis_mapping in &self.axis_mappings {
            axis_mapping.apply_mapping(key_state, gamepad);
        }

        for button_mapping in &self.button_mappings {
            button_mapping.apply_mapping(key_state, gamepad);
        }
    }
}

impl Config {
    pub fn apply_mapping(&self, key_state: &mut HashMap<Key, bool>, gamepad: &Gamepad) {
        for controller_mapping in &self.controller_mappings {
            controller_mapping.apply_mapping(key_state, gamepad);
        }
    }
}

fn main() {
    let config = Config {
        controller_mappings: vec![
            ControllerMapping {
                controller_id: 0,
                axis_mappings: vec![
                    AxisMapping {
                        threshold: 0.3,
                        axis: gilrs::Axis::LeftStickX,
                        high_key: Key::KeyD,
                        low_key: Key::KeyA,
                    },
                    AxisMapping {
                        threshold: 0.3,
                        axis: gilrs::Axis::LeftStickY,
                        high_key: Key::KeyW,
                        low_key: Key::KeyS,
                    },
                ],
                button_mappings: vec![
                    ButtonMapping {
                        button: gilrs::Button::South,
                        key: Key::KeyE,
                    },
                    ButtonMapping {
                        button: gilrs::Button::East,
                        key: Key::KeyK,
                    },
                    ButtonMapping {
                        button: gilrs::Button::West,
                        key: Key::KeyJ,
                    },
                    ButtonMapping {
                        button: gilrs::Button::RightTrigger,
                        key: Key::KeyL,
                    },
                    ButtonMapping {
                        button: gilrs::Button::LeftTrigger,
                        key: Key::KeyH,
                    },
                ],
            },
            ControllerMapping {
                controller_id: 1,
                axis_mappings: vec![
                    AxisMapping {
                        threshold: 0.3,
                        axis: gilrs::Axis::LeftStickX,
                        high_key: Key::RightArrow,
                        low_key: Key::LeftArrow,
                    },
                    AxisMapping {
                        threshold: 0.3,
                        axis: gilrs::Axis::LeftStickY,
                        high_key: Key::KeyM,
                        low_key: Key::DownArrow,
                    },
                ],
                button_mappings: vec![
                    ButtonMapping {
                        button: gilrs::Button::South,
                        key: Key::Space,
                    },
                    ButtonMapping {
                        button: gilrs::Button::East,
                        key: Key::KeyX,
                    },
                    ButtonMapping {
                        button: gilrs::Button::West,
                        key: Key::KeyC,
                    },
                    ButtonMapping {
                        button: gilrs::Button::RightTrigger,
                        key: Key::KeyV,
                    },
                    ButtonMapping {
                        button: gilrs::Button::LeftTrigger,
                        key: Key::KeyZ,
                    },
                ],
            },
        ],
    };

    let mut key_state = HashMap::<Key, bool>::new();
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event_blocking(None) {
            println!("{:?} New event from {}: {:?}", time, id, event);

            for (_id, gamepad) in gilrs.gamepads() {
                config.apply_mapping(&mut key_state, &gamepad);
            }
        }
    }
}

fn key_press_once(key_state: &mut HashMap<Key, bool>, key: Key) {
    let is_down = key_state.entry(key).or_insert(false);
    if !*is_down {
        let _ = simulate(&EventType::KeyPress(key));
        *is_down = true
    }
}

fn key_release_once(key_state: &mut HashMap<Key, bool>, key: Key) {
    let is_down = key_state.entry(key).or_insert(false);
    if *is_down {
        let _ = simulate(&EventType::KeyRelease(key));
        *is_down = false
    }
}
