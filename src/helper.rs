use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

pub fn random_in_radius<R: Rng>(rng: &mut R, center: Vec3, radius: f32) -> Vec2 {
    let [x0, y0, _] = center.to_array();
    let t = 2.0 * PI * rng.gen_range(0.0..=1.0);
    let r = radius * rng.gen_range(0.0..=1.0f32).sqrt();
    let x = x0 + r * t.cos();
    let y = y0 + r * t.sin();
    Vec2::new(x, y)
}

pub fn move_from_deadzone(origin: Vec2, deadzone: f32) -> Vec2 {
    let [x, y] = origin.to_array();
    let x = if x.is_sign_positive() { x + deadzone } else { x - deadzone };
    let y = if y.is_sign_positive() { y + deadzone } else { y - deadzone };
    Vec2::new(x, y)
}

/// Returns the angle between 2 points in radians
pub fn angle_between(a: Vec2, b: Vec2) -> f32 {
    let [ax, ay] = a.to_array();
    let [bx, by] = b.to_array();
    (by - ay).atan2(bx - ax)
}
