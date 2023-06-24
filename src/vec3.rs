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
    pub fn rotate_by(&self, rotation: &Vec3<f32>) -> Vec3<f32> {
        // let a = self.camera.rotation.z;
        let b = rotation.y;
        let c = rotation.x;

        // This formula includes all axes, but currently z isn't used:
        // #[rustfmt::skip]
        // Vec3::<f32> {
        //     x: direction.x * (a.cos() * b.cos()) + direction.y * (a.cos() * b.sin() * c.sin() - a.sin() * c.cos()) + direction.z * (a.cos() * b.sin() * c.cos() + a.sin() * c.sin()),
        //     y: direction.x * (a.sin() * b.cos()) + direction.y * (a.sin() * b.sin() * c.sin() + a.cos() * c.cos()) + direction.z * (a.sin() * b.sin() * c.cos() - a.cos() * c.sin()),
        //     z: direction.x * (-b.sin()) + direction.y * (b.cos() * c.sin()) + direction.z * (b.cos() * c.cos()),
        // }

        // This formula skips z:
        Vec3::<f32> {
            x: self.x * b.cos() + self.y * b.sin() * c.sin() + self.z * b.sin() * c.cos(),
            y: self.y * c.cos() + self.z * -c.sin(),
            z: self.x * -b.sin() + self.y * b.cos() * c.sin() + self.z * b.cos() * c.cos(),
        }
    }
}