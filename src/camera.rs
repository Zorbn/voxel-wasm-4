use crate::vec3::*;
use crate::wasm4::*;

const CAMERA_ROTATION_SPEED: f32 = 0.03;
const CAMERA_MAX_X_ROTATION: f32 = std::f32::consts::FRAC_PI_2 - 0.1;
const CAMERA_MOVE_SPEED: f32 = 0.05;
const CAMERA_DEFAULT_FORWARD: Vec3<f32> = Vec3::new(0.0, 0.0, 1.0);
const CAMERA_DEFAULT_RIGHT: Vec3<f32> = Vec3::new(1.0, 0.0, 0.0);

pub struct Camera {
    pub position: Vec3<f32>,
    pub rotation: Vec3<f32>,
    pub forward: Vec3<f32>,
    pub right: Vec3<f32>,
}

impl Camera {
    pub const fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 15.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            forward: CAMERA_DEFAULT_FORWARD,
            right: CAMERA_DEFAULT_RIGHT,
        }
    }

    pub fn update(&mut self, gamepad1: u8, gamepad2: u8) {
        self.rotate(gamepad1);
        self.step(gamepad2);
    }

    fn rotate(&mut self, gamepad: u8) {
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

        if rotate_x == 0.0 && rotate_y == 0.0 {
            return;
        }

        self.rotation.x = (self.rotation.x + rotate_x * CAMERA_ROTATION_SPEED)
            .clamp(-CAMERA_MAX_X_ROTATION, CAMERA_MAX_X_ROTATION);
        self.rotation.y += rotate_y * CAMERA_ROTATION_SPEED;
        self.forward = CAMERA_DEFAULT_FORWARD.rotate_by(self.rotation);
        self.right = CAMERA_DEFAULT_RIGHT.rotate_by(self.rotation);
    }

    fn step(&mut self, gamepad: u8) {
        let mut move_x = 0.0;
        let mut move_z = 0.0;

        if gamepad & BUTTON_LEFT != 0 {
            move_x -= 1.0;
        }

        if gamepad & BUTTON_RIGHT != 0 {
            move_x += 1.0;
        }

        if gamepad & BUTTON_UP != 0 {
            move_z += 1.0;
        }

        if gamepad & BUTTON_DOWN != 0 {
            move_z -= 1.0;
        }

        if move_x == 0.0 && move_z == 0.0 {
            return;
        }

        let move_squared: f32 = move_x * move_x + move_z * move_z;
        let move_magnitude: f32 = move_squared.sqrt();
        move_x = move_x / move_magnitude * CAMERA_MOVE_SPEED;
        move_z = move_z / move_magnitude * CAMERA_MOVE_SPEED;

        self.position.x += self.forward.x * move_z + self.right.x * move_x;
        self.position.y += self.forward.y * move_z + self.right.y * move_x;
        self.position.z += self.forward.z * move_z + self.right.z * move_x;
    }
}
