use std::default;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin};

const SEED: u32 = 2137;
const CHUNK_SIZE: usize = 16;
const WORLD_SCALE: f64 = 0.1;
const NOISE_THRESHOLD: f64 = 0.3;

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct CustomUV;

#[derive(Component)]
struct Chunk;

#[derive(PartialEq, Debug, Copy, Clone)]
enum BlockType {
    Air,
    Dirt,
}

#[derive(Clone, Copy, Debug)]
enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

#[derive(Clone, Debug)]
struct Block {
    block_type: BlockType,
    block_intersections: Vec<BlockFace>,
    block_position: IVec3,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(WorldInspectorPlugin::new())
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Import the custom texture.
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/dirt.png");
    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_chunk_mesh(IVec3::new(0, 0, 0)));

    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh_handle,
            material: materials.add(StandardMaterial {
                base_color_texture: Some(custom_texture_handle),
                ..default()
            }),
            ..default()
        },
        CustomUV,
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(40., 40., 40.).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            ..default()
        },
        transform: camera_and_light_transform,
        ..default()
    });

    // Text to describe the controls.
    commands.spawn(
        TextBundle::from_section(
            "Controls:\nSpace: Change UVs\nX/Y/Z: Rotate\nR: Reset orientation",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<Input<KeyCode>>,
    // mesh_query: Query<&Handle<Mesh>, With<CustomUV>>,
    // mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Transform, With<CustomUV>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::X) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::Y) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::Z) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::R) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

fn create_cube_mesh() -> Mesh {
    let mut cube_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5], // vertex with index 1
            [0.5, 0.5, 0.5], // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    );

    // Set-up UV coordinates
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the right side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the left side. 
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the back side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the forward side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ],
    );

    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    );

    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    #[rustfmt::skip]
    cube_mesh.set_indices(Some(Indices::U32(vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y) 
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ])));

    cube_mesh
}

/// Creates a 32x32x32 chunk mesh using 3D Perlin noise.
///
/// The mesh is created by sampling the noise function at each vertex position.
/// If the noise value is above a certain threshold, a cube is created at that position.
/// The chunk_position vec3 is the position of the chunk in the world. It is scaled down by the chunk size.
fn create_chunk_mesh(chunk_position: IVec3) -> Mesh {
    // Create a new mesh.
    let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Generate a vector of Blocks, representing whether a cube should be created at that position.
    let mut chunk_data: Vec<Block> = Vec::new();

    // Create a 3D Perlin noise function with a random seed.
    let perlin = Perlin::new(SEED);

    // Loop over each block position in the chunk.
    // Remember to offset the position by the chunk position.
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Scale the position down by the chunk size.
                let scaled_x = x as i32 + (chunk_position.x * CHUNK_SIZE as i32);
                let scaled_y = y as i32 + (chunk_position.y * CHUNK_SIZE as i32);
                let scaled_z = z as i32 + (chunk_position.z * CHUNK_SIZE as i32);
                // info!("Scaled position: {}, {}, {}", scaled_x, scaled_y, scaled_z);

                // Sample the noise function at the scaled position.
                // The perlin noise needs a float value, so we need to cast the scaled position to a float.
                let noise_value = perlin.get([
                    scaled_x as f64 * WORLD_SCALE,
                    scaled_y as f64 * WORLD_SCALE,
                    scaled_z as f64 * WORLD_SCALE,
                ]);

                // If the noise value is above the threshold, create a cube at that position.
                if noise_value > NOISE_THRESHOLD {
                    chunk_data.push(Block {
                        block_type: BlockType::Dirt,
                        block_intersections: Vec::new(),
                        block_position: IVec3::new(scaled_x, scaled_y, scaled_z),
                    });
                } else {
                    chunk_data.push(Block {
                        block_type: BlockType::Air,
                        block_intersections: Vec::new(),
                        block_position: IVec3::new(scaled_x, scaled_y, scaled_z),
                    });
                }
            }
        }
    }
    // Now that the chunk data is generated, check the neighbouring blocks to see if we need to create faces.
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Get the index of the current block.
                let index = x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE);

                // Get the block at the current index.
                let current_block = &chunk_data[index];

                if current_block.block_type == BlockType::Air {
                    continue;
                }

                // #[derive(PartialEq, Debug, Copy, Clone)]
                // enum BlockType {
                //     Air,
                //     Dirt,
                // }

                // #[derive(Clone, Copy, Debug)]
                // enum BlockFace {
                //     Top,
                //     Bottom,
                //     Left,
                //     Right,
                //     Front,
                //     Back,
                // }

                // #[derive(Clone, Debug)]
                // struct Block {
                //     block_type: BlockType,
                //     block_intersections: Vec<BlockFace>,
                //     block_position: IVec3,
                // }

                // Check the block above the current block.
                if y < CHUNK_SIZE - 1 {
                    let above_block = &chunk_data[index + CHUNK_SIZE];
                    if above_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index].block_intersections.push(BlockFace::Top);
                    }
                }

                // Check the block below the current block.
                if y > 0 {
                    let below_block = &chunk_data[index - CHUNK_SIZE];
                    if below_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index]
                            .block_intersections
                            .push(BlockFace::Bottom);
                    }
                }

                // Check the block to the left of the current block.
                if x > 0 {
                    let left_block = &chunk_data[index - 1];
                    if left_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index].block_intersections.push(BlockFace::Left);
                    }
                }

                // Check the block to the right of the current block.
                if x < CHUNK_SIZE - 1 {
                    let right_block = &chunk_data[index + 1];
                    if right_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index].block_intersections.push(BlockFace::Right);
                    }
                }

                // Check the block in front of the current block.
                if z < CHUNK_SIZE - 1 {
                    let front_block = &chunk_data[index + CHUNK_SIZE * CHUNK_SIZE];
                    if front_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index].block_intersections.push(BlockFace::Front);
                    }
                }

                // Check the block behind the current block.
                if z > 0 {
                    let back_block = &chunk_data[index - CHUNK_SIZE * CHUNK_SIZE];
                    if back_block.block_type != BlockType::Air {
                        // Update the block_intersections of the current block.
                        // Update, don't overwrite, because there might be multiple blocks intersecting.
                        chunk_data[index].block_intersections.push(BlockFace::Back);
                    }
                }
            }
        }
    }

    // We now have all the information we need to create the mesh.
    // We need to create a face for each block that intersects with another block.
    // We can do this by iterating over the chunk data and checking the block_intersections of each block.
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for block in &chunk_data {
        if block.block_intersections.is_empty() {
            // The block is not intersecting with any other blocks, so we need to create a face for it.
            let x = block.block_position.x as f32;
            let y = block.block_position.y as f32;
            let z = block.block_position.z as f32;

            // Create the vertices for the face.
            vertices.push([x - 0.5, y - 0.5, z - 0.5]);
            vertices.push([x + 0.5, y - 0.5, z - 0.5]);
            vertices.push([x + 0.5, y + 0.5, z - 0.5]);
            vertices.push([x - 0.5, y + 0.5, z - 0.5]);

            // Create the indices for the face.
            let i = vertices.len() as u32 - 4;
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + 2);
            indices.push(i);
            indices.push(i + 2);
            indices.push(i + 3);

            // Create the normals for the face. The normal should be facing towards air. If there are multiple air blocks, the normal should be facing up.
            // The above is TODO, for now we just assume the normal is facing up.
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);

            // Create the UVs for the face.
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([1.0, 1.0]);
            uvs.push([0.0, 1.0]);
        }
    }

    // Convert the vectors to VertexAttributeValues and add them to the mesh.
    chunk_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(vertices),
    );
    chunk_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    chunk_mesh.set_indices(Some(Indices::U32(indices)));

    // Count the amount of Air and Dirt blocks.
    let mut air_blocks = 0;
    let mut dirt_blocks = 0;
    for block in &chunk_data {
        match block.block_type {
            BlockType::Air => air_blocks += 1,
            BlockType::Dirt => dirt_blocks += 1,
        }
    }
    info!("Air blocks: {}", air_blocks);
    info!("Dirt blocks: {}", dirt_blocks);
    // Count average intersections.
    let mut total_intersections = 0;
    for block in &chunk_data {
        total_intersections += block.block_intersections.len();
    }
    info!(
        "Average intersections: {}",
        total_intersections as f64 / chunk_data.len() as f64
    );

    chunk_mesh
}
