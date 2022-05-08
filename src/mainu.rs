extern crate bevy;
use bevy::prelude::*;
extern crate fastrand;
use fastrand::Rng;
use std::f32::consts::TAU;

mod orthographic_rotate_panning;

mod materia;
use materia::*;

#[derive(Component)]
struct Resident;
#[derive(Component)]
struct Position(Vec2);
struct PixPerMRad(f32);
struct TextScale(f32);


const RANDOM_DUDE_PLACEMENT_RADIUS:f32 = 40.0;

const PIP_RADIUS:f32 = 2.0;
const LABEL_LINEI:f32 = 11.0;
const OUTER_PADDING_MRADI:f32 = 5.0;
const DARKER_COLOR:Color = Color::rgb(0.4,0.4,0.4);
const BACKGROUND_COLOR: Color = Color::rgb(0.68, 0.68, 0.68);

fn setup_example_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ppr: Res<PixPerMRad>,
    text_scale: Res<TextScale>,
){
    let r = Rng::with_seed(80);
    let count = 50;
    let circle = asset_server.load("circle.png");
    let pip_scale = Vec3::new(
        PIP_RADIUS,
        PIP_RADIUS,
        1.0,
    );
    let ts = text_scale.0*LABEL_LINEI;
    
    // commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());
    
    for _ in 0..count {
        commands.spawn()
            .insert(Resident)
            .insert_bundle(SpriteBundle{
                texture: circle.clone(),
                transform: Transform{
                    translation: flat3(from_angle(r.f32()*TAU)*sq(r.f32())*RANDOM_DUDE_PLACEMENT_RADIUS),
                    scale: pip_scale,
                    ..default()
                },
                sprite: Sprite{
                    color: DARKER_COLOR,
                    anchor: bevy::sprite::Anchor::Center,
                    custom_size: Some(Vec2::new(2.0, 2.0)),
                    ..default()
                },
                ..default()
            })
        ;
    }
    
    let rubik = asset_server.load("Rubik-Medium.ttf");
    let tpad = Val::Px(OUTER_PADDING_MRADI*ppr.0);
    commands.spawn_bundle(TextBundle{
        text: Text{
            sections: vec![
                TextSection {
                    value: "PROPINQUITY TABLE".into(),
                    style: TextStyle {
                        font: rubik,
                        font_size: ts,
                        color: DARKER_COLOR,
                    },
                },
            ],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect{
                left: tpad,
                bottom: tpad,
                ..default()
            },
            ..default()
        },
        ..default()
    });
    
}

// pub struct BasicLevel;
// impl Plugin for BasicLevel {
//     fn build(&self, app: &mut App) {
//         app.add_startup_system(setup_example_level);    
//     }
// }



fn main() {
    let makos_screen_ppmr:f32 = ((1920.0/2.0)/((700.0/2.0)/632.0f32).atan()) / 1000.0;
    println!("makos_screen_ppmr {}", makos_screen_ppmr);
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Msaa{ samples: 4 })
        .insert_resource(PixPerMRad(makos_screen_ppmr))
        .insert_resource(TextScale(makos_screen_ppmr))
        .add_plugin(orthographic_rotate_panning::OrthographicRotatePanningZooming)
        .add_startup_system(setup_example_level)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
