#[cfg(feature = "buddy-alloc")]
mod alloc;
mod rng;
mod wasm4;
use wasm4::*;

#[derive(Clone, Copy)]
struct Vec3<T> {
    x: T,
    y: T,
    z: T,
}

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
const RAY_RANGE: f32 = 32.0;
const SHADOW_DISTANCE: f32 = 16.0;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 160;
const ASPECT_RATIO: f32 = 1.0;
const HEIGHT: f32 = 2.0;
const WIDTH: f32 = ASPECT_RATIO * HEIGHT;
const FOCAL_LENGTH: f32 = 1.0;
const HORIZONTAL: Vec3<f32> = Vec3::<f32> {
    x: WIDTH,
    y: 0.0,
    z: 0.0,
};
const VERTICAL: Vec3<f32> = Vec3::<f32> {
    x: 0.0,
    y: HEIGHT,
    z: 0.0,
};
const TEXTURE_SIZE: usize = 8;

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

struct Game {
    frame_count: u32,
    map: [u8; MAP_LENGTH],
    rng: rng::Rng,
}

impl Game {
    pub const fn new() -> Self {
        Self {
            frame_count: 0,
            map: [0; MAP_LENGTH],
            rng: rng::Rng::new(777),
        }
    }

    pub fn start(&mut self) {
        self.generate_map();
    }

    pub fn update(&mut self) {
        let time = self.frame_count as f32 * 0.05;

        let start = Vec3::<f32> {
            x: 15.5,
            y: 15.5,
            z: 15.5 + time,
        };
        let lower_left_corner = Vec3::<f32> {
            x: start.x - HORIZONTAL.x * 0.5 - VERTICAL.x * 0.5,
            y: start.y - HORIZONTAL.y * 0.5 - VERTICAL.y * 0.5,
            z: start.z - HORIZONTAL.z * 0.5 - VERTICAL.z * 0.5 - FOCAL_LENGTH,
        };

        let angle = time.sin() * 0.25;

        for y in 0..SCREEN_HEIGHT {
            let v = y as f32 / SCREEN_HEIGHT as f32;
            for x in 0..SCREEN_WIDTH {
                let u = x as f32 / SCREEN_WIDTH as f32;
                let direction = Vec3::<f32> {
                    x: lower_left_corner.x + u * HORIZONTAL.x + v * VERTICAL.x - start.x,
                    y: lower_left_corner.y + u * HORIZONTAL.y + v * VERTICAL.y - start.y,
                    z: -(lower_left_corner.z + u * HORIZONTAL.z + v * VERTICAL.z - start.z),
                };

                let rotated_direction = Vec3::<f32> {
                    x: direction.x * angle.cos() + direction.z * angle.sin(),
                    y: direction.y,
                    z: direction.x * -angle.sin() + direction.z * angle.cos(),
                };

                let is_dithered = (x + y) & 1 == 0;
                let color = self.raycast(start, rotated_direction, is_dithered);
                unsafe { *DRAW_COLORS = color }
                pixel(x, y);
            }
        }

        self.frame_count += 1;
    }

    fn generate_map(&mut self) {
        for i in 0..MAP_LENGTH {
            if self.rng.range(100) > 90 {
                self.map[i] = 1;
            }
        }
    }

    fn hit_map(&self, position: Vec3<i32>) -> bool {
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

    // Uses DDA Voxel traversal to find the first voxel hit by the ray.
    fn raycast(&self, mut start: Vec3<f32>, direction: Vec3<f32>, is_dithered: bool) -> u16 {
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
        let mut initial_step = Vec3::<f32> {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

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

        let mut hit_block = self.hit_map(block);
        let mut last_move = 0;
        let max_distance = if is_dithered {
            SHADOW_DISTANCE
        } else {
            RAY_RANGE
        };
        while !hit_block && last_dist_to_next < max_distance {
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

            hit_block = self.hit_map(block);
        }

        if !hit_block {
            return 4;
        }

        // Take the absolute value to keep the wrapping math
        // working even at negative coordinates.
        let hit_position = Vec3::<f32> {
            x: (start.x + last_dist_to_next * direction.x).abs(),
            y: (start.y + last_dist_to_next * direction.y).abs(),
            z: (start.z + last_dist_to_next * direction.z).abs(),
        };

        // Find texture uv, multiply by texture size then
        // wrap to stay within the texture's bounds.
        // "a ^ (x-1)" is the same as "a % x".
        // Vertical and horizontal texture mapping requires different math.
        unsafe {
            let u;
            let v;

            if last_move == 1 {
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

            last_move + 1 + (((SMILEY[v] & (1 << u)) != 0) as u16)
        }
    }
}

static mut GAME: Game = Game::new();

#[no_mangle]
unsafe fn start() {
    GAME.start();
}

#[no_mangle]
unsafe fn update() {
    GAME.update();
}
