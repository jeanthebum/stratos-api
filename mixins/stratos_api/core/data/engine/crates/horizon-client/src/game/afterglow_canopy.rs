//! Client-only water beads: real leaf attachment and collision, shader-made optics.
//! Bounded CPU trajectories feed the native weather mesh. No Minecraft water
//! particle sprites, block mutations, server changes or gameplay RNG are used.
use super::{Dim, Game};
use glam::DVec3;
use horizon_client::blocks::Fluid;
use horizon_client::weather::Vertex;
use horizon_core::random::{LegacyRandom, RandomSource};

const MAX_DROPS: usize = 256;
struct Drop {
    position: [f64; 3],
    previous: [f64; 3],
    velocity: f64,
    age: u32,
    hold: u32,
    radius: f64,
    kind: u32,
}

#[derive(Default)]
pub(super) struct Canopy {
    pub water: bool,
    pub strength: f32,
    pub humidity: f32,
    wet: f32,
    tick: i64,
    drops: Vec<Drop>,
    emitted: u64,
    landed: u64,
    rain_impacts: u64,
    converted: u64,
    rain_budget: u8,
}
impl Canopy {
    pub fn impact(&mut self, mut position: [f64; 3]) {
        if self.rain_budget >= 8 || self.drops.len() >= MAX_DROPS {
            return;
        }
        self.rain_budget += 1;
        self.rain_impacts += 1;
        position[1] += 0.012;
        self.drops.push(Drop {
            position,
            previous: position,
            velocity: 0.0,
            age: 0,
            hold: 0,
            radius: 0.12,
            kind: 4,
        });
    }
    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({"emitted":self.emitted,"landed":self.landed,
            "rain_impacts":self.rain_impacts,"converted_native_water":self.converted,"active":self.drops.len(),
            "hanging":self.drops.iter().filter(|d|d.kind==2).count(),
            "falling":self.drops.iter().filter(|d|d.kind==3).count(),
            "rings":self.drops.iter().filter(|d|d.kind==4).count()})
    }
}

// Pure configuration/rate helpers also exercised by the release harness.
#[cfg(test)]
pub(super) fn settings<'a>(id: &str, values: impl Iterator<Item = (&'a str, f64)>) -> [f32; 2] {
    let (mut enabled, mut strength, mut humidity) = (true, 0.0, 1.0);
    for (name, value) in values {
        match name {
            "biome_atmosphere" => enabled = value > 0.5,
            "canopy_drips" => strength = value.clamp(0.0, 2.0) as f32,
            "jungle_moisture" => humidity = value.clamp(0.0, 2.0) as f32,
            _ => {}
        }
    }
    [
        if enabled && id == "afterglow_hdr" {
            strength
        } else {
            0.0
        },
        humidity,
    ]
}

fn moisture(previous: f32, rain: f32) -> f32 {
    let target = rain.clamp(0.0, 1.0);
    previous + (target - previous) * if target > previous { 0.008 } else { 0.00065 }
}

fn drip_chance(family: Option<usize>, wet: f32, humidity: f32, strength: f32) -> f32 {
    let condensation = match family {
        Some(0) => 0.42,
        Some(1) => 0.19,
        _ => 0.0,
    };
    ((condensation * humidity + wet * 0.55) * strength).clamp(0.0, 0.9)
}

impl Game<'_> {
    pub(super) fn tick_afterglow_canopy(&mut self) {
        self.native_shaders.canopy.rain_budget = 0;
        if self.dim != Dim::Overworld
            || !self.native_shaders.canopy.water
            || !self.settings.horizon.weather
            || self.settings.particles >= 2
        {
            self.native_shaders.canopy.drops.clear();
            return;
        }
        let Some(eye) = self.eye_position() else {
            return;
        };
        let block = eye.map(|p| p.floor() as i32);
        let rain = self.weather_levels(1.0).0;
        let native_water = self.particles.take_water_droplets();
        let state = &mut self.native_shaders.canopy;
        for (kind, position, velocity) in native_water {
            state.converted += 1;
            if state.drops.len() >= MAX_DROPS {
                continue;
            }
            state.drops.push(Drop {
                position,
                previous: position,
                velocity: -velocity[1],
                age: 0,
                hold: 18,
                radius: 0.025,
                kind,
            });
        }
        state.wet = moisture(state.wet, rain);
        state.tick = state.tick.wrapping_add(1);
        let (strength, humidity, wet, tick) =
            (state.strength, state.humidity, state.wet, state.tick);
        let facts = self.res.facts();
        let shapes = self.collision.cached_shapes();
        let world = &self.world;
        // Swept vertical collision catches thin blocks and fluid surfaces, even
        // when a falling bead travels across more than one block in a tick.
        for drop in &mut state.drops {
            drop.previous = drop.position;
            drop.age += 1;
            if drop.kind == 2 && drop.age >= drop.hold {
                drop.kind = 3;
                drop.age = 0;
            }
            if drop.kind != 3 {
                continue;
            }
            drop.velocity = (drop.velocity + 0.035).min(0.85);
            drop.position[1] -= drop.velocity;
            let [x, _, z] = drop.position.map(|v| v.floor() as i32);
            let bx = drop.position[0] - f64::from(x);
            let bz = drop.position[2] - f64::from(z);
            for y in (drop.position[1].floor() as i32..=drop.previous[1].floor() as i32).rev() {
                let block_state = world.block(x, y, z);
                let f = facts.get(block_state);
                let solid = match shapes.cached_shape(block_state) {
                    Ok(Some(shape)) if !shape.is_empty() => {
                        shape.max_at(horizon_core::geometry::Axis::Y, bx, bz)
                    }
                    _ => 0.0,
                };
                let fluid = if f.fluid == Fluid::None {
                    0.0
                } else {
                    f64::from(f.fluid_level) / 9.0
                };
                let top = solid.max(fluid);
                let surface = f64::from(y) + top;
                if top > 0.0 && drop.previous[1] >= surface && drop.position[1] <= surface {
                    drop.position[1] = surface + 0.012;
                    drop.previous = drop.position;
                    drop.kind = 4;
                    drop.age = 0;
                    drop.radius = 0.15;
                    state.landed += 1;
                    break;
                }
            }
        }
        state.drops.retain(|d| {
            d.age < if d.kind == 4 { 10 } else { 140 }
                && (d.position[0] - eye[0]).abs() < 28.0
                && (d.position[2] - eye[2]).abs() < 28.0
                && (d.position[1] - eye[1]).abs() < 32.0
        });
        if strength <= 0.0
            || facts.get(world.block(block[0], block[1], block[2])).fluid != Fluid::None
        {
            return;
        }
        let mut random = LegacyRandom::new(tick.wrapping_mul(718_427_651).wrapping_add(93_117));
        let biomes = world_biomes!(self);
        let attempts = if self.settings.particles == 1 { 4 } else { 8 };
        let mut spawned = 0;
        for _ in 0..attempts {
            if self.native_shaders.canopy.drops.len() >= MAX_DROPS {
                break;
            }
            let x = block[0] + random.next_int_bound(19).unwrap_or(9) - 9;
            let z = block[2] + random.next_int_bound(19).unwrap_or(9) - 9;
            for y in block[1]..=block[1] + 18 {
                let leaf = facts.get(world.block(x, y, z));
                if leaf.name.ends_with("_leaves") {
                    if !facts.is_air(world.block(x, y - 1, z)) || world.light(x, y - 1, z) >> 4 < 4
                    {
                        break;
                    }
                    let Some(chunk) = world.chunks.get(&(x >> 4, z >> 4)) else {
                        break;
                    };
                    let biome = biomes.get(chunk.biome(x, y, z));
                    if biome.snows_at(x, y, z) != Some(false) {
                        break;
                    }
                    let family = horizon_client::biome::native_atmosphere_family(&biome.name);
                    if random.next_float() < drip_chance(family, wet, humidity, strength) {
                        let position = [
                            f64::from(x) + 0.12 + random.next_double() * 0.76,
                            f64::from(y) - 0.025,
                            f64::from(z) + 0.12 + random.next_double() * 0.76,
                        ];
                        let state = &mut self.native_shaders.canopy;
                        state.drops.push(Drop {
                            position,
                            previous: position,
                            velocity: 0.0,
                            age: 0,
                            hold: 12 + random.next_int_bound(18).unwrap_or(8) as u32,
                            radius: 0.022 + random.next_double() * 0.012,
                            kind: 2,
                        });
                        state.emitted += 1;
                        spawned += 1;
                    }
                    break;
                }
                if leaf.full_collision || leaf.fluid != Fluid::None {
                    break;
                }
            }
            if spawned >= 2 {
                break;
            }
        }
    }

    pub(super) fn afterglow_canopy_geometry(
        &self,
        camera: DVec3,
        alpha: f64,
        out: &mut Vec<Vertex>,
    ) {
        let state = &self.native_shaders.canopy;
        if !state.water || !self.settings.horizon.weather || self.settings.particles >= 2 {
            return;
        }
        // Back-to-front sorting matters for the handful of translucent overlaps.
        let mut ordered: Vec<_> = state.drops.iter().collect();
        ordered.sort_by(|a, b| {
            (DVec3::from(b.position) - camera)
                .length_squared()
                .total_cmp(&(DVec3::from(a.position) - camera).length_squared())
        });
        for drop in ordered {
            let p =
                DVec3::from(drop.previous).lerp(DVec3::from(drop.position), alpha.clamp(0.0, 1.0));
            let delta = p - camera;
            let distance = delta.length();
            if distance < 0.15 || distance > 32.0 {
                continue;
            }
            let age = f64::from(drop.age) + alpha.clamp(0.0, 1.0);
            let progress = if drop.kind == 4 {
                age / 10.0
            } else if drop.kind == 2 {
                age / f64::from(drop.hold)
            } else {
                0.0
            };
            let (right, up) = if drop.kind == 4 {
                let radius = drop.radius * (0.35 + progress * 1.8);
                (DVec3::X * radius, DVec3::Z * radius)
            } else {
                let grow = if drop.kind == 2 {
                    0.45 + 0.55 * progress.min(1.0)
                } else {
                    1.0
                };
                let right =
                    DVec3::new(delta.z, 0.0, -delta.x).normalize_or_zero() * drop.radius * grow;
                let height = drop.radius * grow + drop.velocity * 0.11;
                (right, DVec3::Y * height)
            };
            let block = p.floor().as_ivec3();
            let light = self.world.light(block.x, block.y, block.z);
            let packed = u32::from((light >> 4) * 16) | (u32::from((light & 15) * 16) << 8);
            let opacity = ((1.0 - ((distance - 20.0) / 12.0).clamp(0.0, 1.0)) * 255.0) as u32;
            for (pos, uv) in [
                (delta - right + up, [0.0, 0.0]),
                (delta - right - up, [0.0, 1.0]),
                (delta + right - up, [1.0, 1.0]),
                (delta + right + up, [1.0, 0.0]),
            ] {
                out.push(Vertex {
                    pos: pos.as_vec3().to_array(),
                    uv,
                    rect: [0.0; 4],
                    animation: [progress as f32, 0.0, 0.0, 0.0],
                    bases: [0; 4],
                    color: 0x00ff_ffff | (opacity << 24),
                    light: packed,
                    snow: drop.kind,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{drip_chance, moisture, settings};
    #[test]
    fn options_disable_drips_and_other_packs_never_enable_them() {
        assert_eq!(
            settings("other", [("canopy_drips", 2.0)].into_iter())[0],
            0.0
        );
        assert_eq!(
            settings(
                "afterglow_hdr",
                [("canopy_drips", 1.0), ("biome_atmosphere", 0.0)].into_iter()
            )[0],
            0.0
        );
        assert_eq!(settings("afterglow_hdr", [].into_iter())[0], 0.0);
        assert_eq!(
            settings("afterglow_hdr", [("canopy_drips", 1.4)].into_iter())[0],
            1.4
        );
    }
    #[test]
    fn humidity_is_local_and_rain_retains_a_drying_tail() {
        assert_eq!(drip_chance(Some(4), 0.0, 1.0, 1.0), 0.0);
        assert!(drip_chance(Some(0), 0.0, 1.0, 1.0) > 0.3);
        assert_eq!(drip_chance(Some(0), 0.0, 0.0, 1.0), 0.0);
        let mut wet = 0.0;
        for _ in 0..200 {
            wet = moisture(wet, 1.0);
        }
        assert!(wet > 0.7);
        for _ in 0..200 {
            wet = moisture(wet, 0.0);
        }
        assert!(wet > 0.5 && wet < 0.8);
        assert!(drip_chance(Some(8), wet, 0.0, 1.0) > 0.2);
    }
}
