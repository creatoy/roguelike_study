use bevy::prelude::{ButtonInput, KeyCode, Res};

pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
}

impl GameControl {
    pub fn just_pressed(&self, keyboard_input: &Res<ButtonInput<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_pressed(KeyCode::KeyW)
                    || keyboard_input.just_pressed(KeyCode::ArrowUp)
            }
            GameControl::Down => {
                keyboard_input.just_pressed(KeyCode::KeyS)
                    || keyboard_input.just_pressed(KeyCode::ArrowDown)
            }
            GameControl::Left => {
                keyboard_input.just_pressed(KeyCode::KeyA)
                    || keyboard_input.just_pressed(KeyCode::ArrowLeft)
            }
            GameControl::Right => {
                keyboard_input.just_pressed(KeyCode::KeyD)
                    || keyboard_input.just_pressed(KeyCode::ArrowRight)
            }
        }
    }
}

pub fn get_movement(control: GameControl, input: &Res<ButtonInput<KeyCode>>) -> i32 {
    if control.just_pressed(input) {
        1
    } else {
        0
    }
}
