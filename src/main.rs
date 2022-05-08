extern crate bevy;
use bevy::prelude::*;
extern crate fastrand;
use fastrand::Rng;

mod orthographic_rotate_panning;

mod materia;
use materia::*;

const CUBE_INTERVAL: f32 = 1.0;
const CUBE_GAPR: f32 = 0.1;
const CUBE_RADIUS: f32 = CUBE_INTERVAL / 2.0 - CUBE_GAPR;
const CUBE_COUNT_R: isize = 30;
const CUBE_HEIGHT_VARIANCE: f32 = 2.5;
const CUBE_HEIGHT_PARK: f32 = 0.07;

struct CubeSpec {
    height: f32,
    lowx: isize,
    lowy: isize,
    highx: isize,
    highy: isize,
}
impl CubeSpec {
    fn is_small(&self) -> bool {
        self.lowx == self.highx && self.lowy == self.highy
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // cubes
    let rng = Rng::with_seed(12);
    let row_length = CUBE_COUNT_R * 2 + 1;
    let n_cells = (row_length * row_length) as usize;
    let mut cubespecs = Vec::with_capacity(n_cells);
    let mut cubespec_index = Vec::new();
    let inclusion_rad = CUBE_COUNT_R as f32 + 0.5;
    for iy in -CUBE_COUNT_R..=CUBE_COUNT_R {
        for ix in -CUBE_COUNT_R..=CUBE_COUNT_R {
            let dist = ((ix * ix) as f32 + (iy * iy) as f32).sqrt();
            cubespecs.push(if dist < inclusion_rad {
                let address = cubespec_index.len();
                cubespec_index.push(Some(CubeSpec {
                    height:
                        if rng.f32() < 0.1 { CUBE_HEIGHT_PARK }
                        else if rng.f32() < 0.3 { 1.0 }
                        else { 1.0 + sq(rng.f32())*CUBE_HEIGHT_VARIANCE },
                    lowx: ix,
                    lowy: iy,
                    highx: ix,
                    highy: iy,
                }));
                Some(address)
            } else {
                None
            });
        }
    }
    //spread some random ones
    //this could be done a lot better if I had an int rect API but I simply can't right now
    //It might also be fine to just not prevent them from overlapping, well it would look fine.. I really should have..
    let to_probe = (n_cells as f32 * 0.1) as usize;
    println!("to_probe: {}", to_probe);
    for _ in 0..to_probe {
        let want_to_delete = rng.f32() < 0.1;
        for _ in 0..10 {
            //retry any given poke only 4 times. This will usually be enough and it will prevent it from crashing if it somehow gets completely stuck
            let position = rng.usize(0..n_cells);
            if let Some(&Some(incumbent_index)) = cubespecs.get(position) {
                if want_to_delete {
                    let csn = &mut cubespec_index[incumbent_index];
                    if let &mut Some(ref cs) = csn {
                        if !cs.is_small() {
                            continue;
                        }
                        *csn = None;
                        cubespecs[position] = None;
                        println!("deleted {}", incumbent_index);
                    } else {
                        continue;
                    }
                } else {
                    //extend it if you can
                    let cs = cubespec_index[incumbent_index].as_ref().unwrap();
                    if !cs.is_small() {
                        continue;
                    }
                    if rng.bool() {
                    let nx = cs.highx + 1;
                    let other_position =
                        ((cs.lowy + CUBE_COUNT_R) * row_length + nx + CUBE_COUNT_R) as usize;
                    let ncsp = &mut cubespecs[other_position];
                    match ncsp {
                        &mut Some(ncsi) => {
                            let ncii = &mut cubespec_index[ncsi];
                            if !ncii.as_ref().unwrap().is_small() {
                                continue;
                            }
                            *ncii = None;
                            *ncsp = Some(incumbent_index);
                            cubespec_index[incumbent_index].as_mut().unwrap().highx = nx;
                        }
                        &mut None => {
                            continue;
                        }
                    }
                    }else{
                        let ny = cs.highy + 1;
                        let other_position =
                            ((ny + CUBE_COUNT_R) * row_length + cs.lowx + CUBE_COUNT_R) as usize;
                        let ncsp = &mut cubespecs[other_position];
                        match ncsp {
                            &mut Some(ncsi) => {
                                let ncii = &mut cubespec_index[ncsi];
                                if !ncii.as_ref().unwrap().is_small() {
                                    continue;
                                }
                                *ncii = None;
                                *ncsp = Some(incumbent_index);
                                cubespec_index[incumbent_index].as_mut().unwrap().highy = ny;
                            }
                            &mut None => {
                                continue;
                            }
                        }
                    }
                }
            } else {
                continue;
            }
            break;
        }
    }

    let cube_material = materials.add(Color::hsl(124.0, 16.0 / 100.0, 43.0 / 100.0).into());
    for cr in cubespec_index.iter() {
        if let Some(ref cubespec) = *cr {
            let boxu = shape::Box {
                min_x: cubespec.lowx as f32 * CUBE_INTERVAL - CUBE_RADIUS,
                max_x: cubespec.highx as f32 * CUBE_INTERVAL + CUBE_RADIUS,
                min_z: cubespec.lowy as f32 * CUBE_INTERVAL - CUBE_RADIUS,
                max_z: cubespec.highy as f32 * CUBE_INTERVAL + CUBE_RADIUS,
                min_y: 0.0,
                max_y: cubespec.height,
            };
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(boxu)),
                material: cube_material.clone(),
                ..default()
            });
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::hsl(
            124.0 / 360.0,
            0.0 / 100.0,
            87.0 / 100.0,
        )))
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(orthographic_rotate_panning::OrthographicRotatePanningZooming)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
