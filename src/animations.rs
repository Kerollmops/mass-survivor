use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;

pub fn space_invader_animation(
    mut start: Vec3,
    steps: usize,
    width: f32,
    height: f32,
) -> Sequence<Transform> {
    let mut moves = Vec::with_capacity(steps * 4);
    for _ in 0..steps {
        // left_to_right 0..steps {
        moves.push(Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs(4),
            OneAxisTransformPositionLens {
                slide_on: Axis::X,
                start,
                end: start + Vec3::new(width, 0., 0.),
            },
        ));

        // first_top_to_bottom
        moves.push(Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_millis(50),
            OneAxisTransformPositionLens {
                slide_on: Axis::Y,
                start: start + Vec3::new(width, 0., 0.),
                end: start + Vec3::new(width, -height, 0.),
            },
        ));

        // right_to_left
        moves.push(Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs(4),
            OneAxisTransformPositionLens {
                slide_on: Axis::X,
                start: start + Vec3::new(width, -height, 0.),
                end: start + Vec3::new(0., -height, 0.),
            },
        ));

        // second_top_to_bottom
        let end = start + Vec3::new(0., -(height * 2.), 0.);
        moves.push(Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_millis(50),
            OneAxisTransformPositionLens {
                slide_on: Axis::Y,
                start: start + Vec3::new(0., -height, 0.),
                end,
            },
        ));

        start = end;
    }

    Sequence::new(moves)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OneAxisTransformPositionLens {
    slide_on: Axis,
    start: Vec3,
    end: Vec3,
}

impl Lens<Transform> for OneAxisTransformPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;

        let axis = match self.slide_on {
            Axis::X => target.translation[1],
            Axis::Y => target.translation[0],
        };

        target.translation = value;

        match self.slide_on {
            Axis::X => target.translation[1] = axis,
            Axis::Y => target.translation[0] = axis,
        }
    }
}
