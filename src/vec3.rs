#[derive(Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

impl Vec3<f32> {
    pub fn rotated(&self, rotation: &Vec3<f32>) -> Vec3<f32> {
        let mut self_rotated = *self;
        self_rotated.rotate_by(rotation);
        self_rotated
    }

    pub fn rotate_by(&mut self, rotation: &Vec3<f32>) {
        // let a = self.camera.rotation.z;
        let b = rotation.y;
        let c = rotation.x;
        let b_cos = b.cos();
        let b_sin = b.sin();
        let c_cos = c.cos();
        let c_sin = c.sin();

        // This formula includes all axes, but currently z isn't used:
        // Vec3::<f32> {
        //     x: direction.x * (a.cos() * b.cos()) + direction.y * (a.cos() * b.sin() * c.sin() - a.sin() * c.cos()) + direction.z * (a.cos() * b.sin() * c.cos() + a.sin() * c.sin()),
        //     y: direction.x * (a.sin() * b.cos()) + direction.y * (a.sin() * b.sin() * c.sin() + a.cos() * c.cos()) + direction.z * (a.sin() * b.sin() * c.cos() - a.cos() * c.sin()),
        //     z: direction.x * (-b.sin()) + direction.y * (b.cos() * c.sin()) + direction.z * (b.cos() * c.cos()),
        // }

        // This formula skips z:
        let x = self.x;
        let y = self.y;
        let z = self.z;

        self.x = x * b_cos + y * b_sin * c_sin + z * b_sin * c_cos;
        self.y = y * c_cos + z * -c_sin;
        self.z = x * -b_sin + y * b_cos * c_sin + z * b_cos * c_cos;
    }

    pub fn rotate_by_precalculated(&mut self, x_sin: f32, x_cos: f32, y_sin: f32, y_cos: f32) {
        let x = self.x;
        let y = self.y;
        let z = self.z;

        self.x = x * y_cos + y * y_sin * x_sin + z * y_sin * x_cos;
        self.y = y * x_cos + z * -x_sin;
        self.z = x * -y_sin + y * y_cos * x_sin + z * y_cos * x_cos;
    }
}