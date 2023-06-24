use crate::vec3::*;
use crate::wasm4::*;

const CAMERA_ROTATION_SPEED: f32 = 0.05;
const CAMERA_MAX_X_ROTATION: f32 = std::f32::consts::FRAC_PI_2 - 0.1;

pub struct Camera {
    pub rotation: Vec3<f32>,
}

impl Camera {
    pub const fn new() -> Self {
        Self {
            rotation: Vec3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    pub fn update(&mut self, gamepad: u8) {
        let mut rotate_y = 0.0;
        let mut rotate_x = 0.0;

        if gamepad & BUTTON_LEFT != 0 {
            rotate_y -= 1.0;
        }

        if gamepad & BUTTON_RIGHT != 0 {
            rotate_y += 1.0;
        }

        if gamepad & BUTTON_UP != 0 {
            rotate_x += 1.0;
        }

        if gamepad & BUTTON_DOWN != 0 {
            rotate_x -= 1.0;
        }

        self.rotation.x = (self.rotation.x + rotate_x * CAMERA_ROTATION_SPEED).clamp(-CAMERA_MAX_X_ROTATION, CAMERA_MAX_X_ROTATION);

        self.rotation.y += rotate_y * CAMERA_ROTATION_SPEED;
    }
}
