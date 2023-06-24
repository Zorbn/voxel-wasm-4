mod alloc;
mod camera;
mod rng;
mod vec3;
#[cfg(feature = "buddy-alloc")]
mod wasm4;

use crate::vec3::*;
use wasm4::*;

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b00100100,
    0b00100100,
    0b00000000,
    0b00100100,
    0b10011001,
    0b11000011,
];

const MAP_SIZE: usize = 32;
const MAP_LENGTH: usize = MAP_SIZE * MAP_SIZE * MAP_SIZE;
const RAY_RANGE: f32 = 24.0;
const SHADOW_DISTANCE: f32 = 16.0;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 160;
const ASPECT_RATIO: f32 = 1.0;
const HEIGHT: f32 = 2.0;
const WIDTH: f32 = ASPECT_RATIO * HEIGHT;
const FOCAL_LENGTH: f32 = 1.0;
const HORIZONTAL: Vec3<f32> = Vec3::new(WIDTH, 0.0, 0.0);
const VERTICAL: Vec3<f32> = Vec3::new(0.0, HEIGHT, 0.0);
const TEXTURE_SIZE: usize = 8;
const INTERACT_DISTANCE: f32 = 6.0;

// From the WASM-4 documentation:
fn pixel(x: usize, y: usize) {
    // The byte index into the framebuffer that contains (x, y)
    let idx = (y * 160 + x) >> 2;

    // Calculate the bits within the byte that corresponds to our position
    let shift = (x as u8 & 0b11) << 1;
    let mask = 0b11 << shift;

    unsafe {
        let palette_color: u8 = (*DRAW_COLORS & 0xf) as u8;
        if palette_color == 0 {
            // Transparent
            return;
        }
        let color = (palette_color - 1) & 0b11;

        let framebuffer = &mut *FRAMEBUFFER;

        framebuffer[idx] = (color << shift) | (framebuffer[idx] & !mask);
    }
}

struct RayHit {
    distance: f32,
    // TODO: Convert to enum?
    // 0 = x, 1 = y, 2 = z
    hit_side: u16,
    block: Option<Vec3<i32>>,
}

struct Game {
    frame_count: u32,
    map: [u8; MAP_LENGTH],
    rng: rng::Rng,
    camera: camera::Camera,
    previous_gamepad1: u8,
}

impl Game {
    const fn new() -> Self {
        Self {
            frame_count: 0,
            map: [0; MAP_LENGTH],
            rng: rng::Rng::new(777),
            camera: camera::Camera::new(),
            previous_gamepad1: 0,
        }
    }

    fn start(&mut self) {
        self.generate_map();
    }

    fn update(&mut self, gamepad1: u8, gamepad2: u8) {
        self.camera.update(gamepad1, gamepad2);

        let pressed_this_frame = gamepad1 & (gamepad1 ^ self.previous_gamepad1);

        if pressed_this_frame & BUTTON_1 != 0 {
            let ray_hit = self.raycast(
                self.camera.position,
                &self.camera.forward,
                INTERACT_DISTANCE,
                true,
            );
            if let Some(ray_hit) = ray_hit {
                let hit_block = ray_hit.block.unwrap();
                self.set_map(&hit_block, 0);
            }
        } else if pressed_this_frame & BUTTON_2 != 0 {
            let ray_hit = self.raycast(
                self.camera.position,
                &self.camera.forward,
                INTERACT_DISTANCE,
                true,
            );
            if let Some(ray_hit) = ray_hit {
                let mut target_block = ray_hit.block.unwrap();

                match ray_hit.hit_side {
                    0 => target_block.x -= self.camera.forward.x.signum() as i32,
                    1 => target_block.y -= self.camera.forward.y.signum() as i32,
                    2 => target_block.z -= self.camera.forward.z.signum() as i32,
                    _ => {}
                }

                self.set_map(&target_block, 1);
            }
        }

        let lower_left_corner = Vec3::<f32> {
            x: self.camera.position.x - HORIZONTAL.x * 0.5 - VERTICAL.x * 0.5,
            y: self.camera.position.y - HORIZONTAL.y * 0.5 - VERTICAL.y * 0.5,
            z: self.camera.position.z - HORIZONTAL.z * 0.5 - VERTICAL.z * 0.5 - FOCAL_LENGTH,
        };

        for y in 0..SCREEN_HEIGHT {
            let v = y as f32 / SCREEN_HEIGHT as f32;
            for x in 0..SCREEN_WIDTH {
                let u = x as f32 / SCREEN_WIDTH as f32;
                let direction = Vec3::<f32> {
                    x: lower_left_corner.x + u * HORIZONTAL.x - self.camera.position.x,
                    y: lower_left_corner.y + v * VERTICAL.y - self.camera.position.y,
                    z: -lower_left_corner.z + self.camera.position.z,
                };

                let rotated_direction = direction.rotate_by(&self.camera.rotation);

                // Modify range to add dithering effect.
                let range = if (x + y) & 1 == 0 {
                    SHADOW_DISTANCE
                } else {
                    RAY_RANGE
                };
                let ray_hit = self.raycast(self.camera.position, &rotated_direction, range, false);
                let color = Self::hit_to_color(ray_hit, &self.camera.position, &rotated_direction);
                unsafe { *DRAW_COLORS = color }
                pixel(x, y);
            }
        }

        // Draw the crosshair:
        unsafe { *DRAW_COLORS = 0x41 }
        rect(78, 78, 4, 4);

        self.frame_count += 1;
        self.previous_gamepad1 = gamepad1;
    }

    fn generate_map(&mut self) {
        let min_y = MAP_SIZE / 2;

        for z in 0..MAP_SIZE {
            for y in min_y..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    self.map[x + y * MAP_SIZE + z * MAP_SIZE * MAP_SIZE] = 1;
                }
            }
        }
    }

    fn hit_map(&self, position: &Vec3<i32>) -> bool {
        let wrapped_position = Vec3::<usize> {
            x: position.x as usize & (MAP_SIZE - 1),
            y: position.y as usize & (MAP_SIZE - 1),
            z: position.z as usize & (MAP_SIZE - 1),
        };

        unsafe {
            *self.map.get_unchecked(
                wrapped_position.x
                    + wrapped_position.y * MAP_SIZE
                    + wrapped_position.z * MAP_SIZE * MAP_SIZE,
            ) == 1
        }
    }

    fn set_map(&mut self, position: &Vec3<i32>, voxel: u8) {
        unsafe {
            let wrapped_position = Vec3::<usize> {
                x: position.x as usize & (MAP_SIZE - 1),
                y: position.y as usize & (MAP_SIZE - 1),
                z: position.z as usize & (MAP_SIZE - 1),
            };

            *self.map.get_unchecked_mut(
                wrapped_position.x
                    + wrapped_position.y * MAP_SIZE
                    + wrapped_position.z * MAP_SIZE * MAP_SIZE,
            ) = voxel;
        }
    }

    fn hit_to_color(ray_hit: Option<RayHit>, start: &Vec3<f32>, direction: &Vec3<f32>) -> u16 {
        // Check if the ray hit nothing:
        if ray_hit.is_none() {
            return 4;
        }

        let ray_hit = ray_hit.unwrap();

        // Take the absolute value to keep the wrapping math
        // working even at negative coordinates.
        let hit_position = Vec3::<f32> {
            x: (start.x + ray_hit.distance * direction.x).abs(),
            y: (start.y + ray_hit.distance * direction.y).abs(),
            z: (start.z + ray_hit.distance * direction.z).abs(),
        };

        // Find texture uv, multiply by texture size then
        // wrap to stay within the texture's bounds.
        // "a ^ (x-1)" is the same as "a % x".
        // Vertical and horizontal texture mapping requires different math.
        unsafe {
            let u;
            let v;

            if ray_hit.hit_side == 1 {
                u = (hit_position.x * TEXTURE_SIZE as f32).to_int_unchecked::<usize>()
                    & (TEXTURE_SIZE - 1);
                v = (hit_position.z * TEXTURE_SIZE as f32).to_int_unchecked::<usize>()
                    & (TEXTURE_SIZE - 1);
            } else {
                u = ((hit_position.x + hit_position.z) * TEXTURE_SIZE as f32)
                    .to_int_unchecked::<usize>()
                    & (TEXTURE_SIZE - 1);
                v = (hit_position.y * TEXTURE_SIZE as f32).to_int_unchecked::<usize>()
                    & (TEXTURE_SIZE - 1);
            }

            ray_hit.hit_side + 1 + (((SMILEY[v] & (1 << u)) != 0) as u16)
        }
    }

    // Uses DDA Voxel traversal to find the first voxel hit by the ray.
    fn raycast(
        &self,
        mut start: Vec3<f32>,
        direction: &Vec3<f32>,
        range: f32,
        with_block: bool,
    ) -> Option<RayHit> {
        // Add a small bias to prevent landing perfectly on block boundaries,
        // otherwise there will be visual glitches in that case.
        start.x += 1e-4;
        start.y += 1e-4;
        start.z += 1e-4;

        let tile_dir = unsafe {
            Vec3::<i32> {
                x: direction.x.signum().to_int_unchecked(),
                y: direction.y.signum().to_int_unchecked(),
                z: direction.z.signum().to_int_unchecked(),
            }
        };
        let ray_step = Vec3::<f32> {
            x: (1.0 / direction.x).abs(),
            y: (1.0 / direction.y).abs(),
            z: (1.0 / direction.z).abs(),
        };
        let mut initial_step = Vec3::new(0.0, 0.0, 0.0);

        if direction.x > 0.0 {
            initial_step.x = (start.x.ceil() - start.x) * ray_step.x;
        } else {
            initial_step.x = (start.x - start.x.floor()) * ray_step.x;
        }

        if direction.y > 0.0 {
            initial_step.y = (start.y.ceil() - start.y) * ray_step.y;
        } else {
            initial_step.y = (start.y - start.y.floor()) * ray_step.y;
        }

        if direction.z > 0.0 {
            initial_step.z = (start.z.ceil() - start.z) * ray_step.z;
        } else {
            initial_step.z = (start.z - start.z.floor()) * ray_step.z;
        }

        let mut dist_to_next = initial_step;
        let mut block = unsafe {
            Vec3::<i32> {
                x: start.x.floor().to_int_unchecked(),
                y: start.y.floor().to_int_unchecked(),
                z: start.z.floor().to_int_unchecked(),
            }
        };
        let mut last_dist_to_next = 0.0;

        let mut hit_block = self.hit_map(&block);
        let mut last_move = 0;
        while !hit_block && last_dist_to_next < range {
            if dist_to_next.x < dist_to_next.y && dist_to_next.x < dist_to_next.z {
                last_dist_to_next = dist_to_next.x;
                dist_to_next.x += ray_step.x;
                block.x += tile_dir.x;
                last_move = 0;
            } else if dist_to_next.y < dist_to_next.z {
                last_dist_to_next = dist_to_next.y;
                dist_to_next.y += ray_step.y;
                block.y += tile_dir.y;
                last_move = 1;
            } else {
                last_dist_to_next = dist_to_next.z;
                dist_to_next.z += ray_step.z;
                block.z += tile_dir.z;
                last_move = 2;
            }

            hit_block = self.hit_map(&block);
        }

        if !hit_block {
            return None;
        }

        Some(RayHit {
            distance: last_dist_to_next,
            hit_side: last_move,
            block: if with_block { Some(block) } else { None },
        })
    }
}

static mut GAME: Game = Game::new();

#[no_mangle]
unsafe fn start() {
    GAME.start();
}

#[no_mangle]
unsafe fn update() {
    GAME.update(*GAMEPAD1, *GAMEPAD2);
}
