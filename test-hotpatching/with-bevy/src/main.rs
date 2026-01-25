use bevy::prelude::*;
use bevy_dependency::{TheCube, move_the_cube};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_the_cube, rotate_the_cube))
        .add_systems(Update, control_transform)
        .run();
}

pub fn rotate_the_cube(mut the_cube: Single<&mut Transform, With<TheCube>>) {
    the_cube.rotation = Quat::from_rotation_z(0.0);
}

fn transform_equals(a: &Transform, b: &Transform) -> bool {
    (a.translation - b.translation).length() < 0.01
        && (a.rotation.to_scaled_axis() - b.rotation.to_scaled_axis()).length() < 0.01
}

pub fn control_transform(
    the_cube: Single<&Transform, With<TheCube>>,
    mut original: Local<Option<Transform>>,
    mut changed: Local<bool>,
) {
    if let Some(original) = *original {
        if !transform_equals(&original, *the_cube) {
            *changed = true;
        } else if *changed {
            panic!("library changes are ignored")
        }
    } else {
        *original = Some(the_cube.clone());
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        TheCube,
    ));
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
