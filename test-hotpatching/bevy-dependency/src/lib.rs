use bevy::prelude::*;

#[derive(Component)]
pub struct TheCube;

pub fn move_the_cube(mut the_cube: Single<&mut Transform, With<TheCube>>) {
    the_cube.translation = Vec3::new(0.0, 0.0, 0.0);
}
