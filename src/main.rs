use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::window::{PrimaryWindow, Window, WindowPlugin, WindowResolution};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rustopia - Magic Dungeon Crawler".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(Material2dPlugin::<MagicMaterial>::default())
        .add_plugins(Material2dPlugin::<SunRaysMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_physics_system,
            camera_follow_system,
            mouse_fire_system,
            radial_menu_system,
            projectile_system,
            projectile_enemy_collision_system.after(projectile_system),
            enemy_movement_system.after(projectile_enemy_collision_system),
            cooldown_system,
            magic_input_system,
            update_magic_material_system,
            update_sun_rays_system,
            update_light_mask_system,
            lobby_dungeon_transition_system,
        ))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct EnemyVelocity(Vec3);

#[derive(Component)]
struct ShadowCaster {
    size: Vec2,
}

#[derive(Component)]
struct LightMask;

#[derive(Resource)]
struct SunLight {
    position: Vec2,
}

#[derive(Component)]
struct Platform {
    size: Vec2,
    kind: PlatformKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlatformKind {
    Solid,
    OneWay,
}

#[derive(Component)]
struct Projectile {
    direction: Vec2,
    speed: f32,
    lifetime: f32,
}

#[derive(Component)]
struct Enemy {
    health: f32,
    speed: f32,
    damage: f32,
    depth: f32,
}

#[derive(Component)]
struct FollowCamera;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameScene {
    Lobby,
    Dungeon,
}

#[derive(Resource)]
struct SceneState(GameScene);

#[derive(Component)]
struct LobbyMarker;

#[derive(Component)]
struct DungeonMarker;

#[derive(Resource)]
struct MagicState {
    combo: Vec<MagicInput>,
    cast_result: Option<MagicCast>,
    shoot_cooldown: f32,
    shoot_timer: f32,
    selected_element: MagicElement,
}

#[derive(Resource)]
struct DashState {
    cooldown: f32,
    cooldown_timer: f32,
    duration: f32,
    timer: f32,
    active: bool,
}

impl Default for DashState {
    fn default() -> Self {
        DashState {
            cooldown: 0.6,
            cooldown_timer: 0.6,
            duration: 0.16,
            timer: 0.0,
            active: false,
        }
    }
}

impl Default for MagicState {
    fn default() -> Self {
        MagicState {
            combo: Vec::new(),
            cast_result: None,
            shoot_cooldown: 0.25,
            shoot_timer: 0.25,
            selected_element: MagicElement::Fire,
        }
    }
}

#[derive(Resource, Default)]
struct RadialMenuState {
    menu_entity: Option<Entity>,
    cursor_position: Vec2,
    selected_element: MagicElement,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MagicInput {
    Fire,
    Frost,
    Wind,
    Arcane,
}

impl MagicInput {
    fn label(self) -> &'static str {
        match self {
            MagicInput::Fire => "Feu",
            MagicInput::Frost => "Givre",
            MagicInput::Wind => "Vent",
            MagicInput::Arcane => "Arcane",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MagicElement {
    Fire,
    Frost,
    Wind,
    Arcane,
}

impl Default for MagicElement {
    fn default() -> Self {
        MagicElement::Fire
    }
}

impl MagicElement {
    fn label(self) -> &'static str {
        match self {
            MagicElement::Fire => "Feu",
            MagicElement::Frost => "Givre",
            MagicElement::Wind => "Vent",
            MagicElement::Arcane => "Arcane",
        }
    }

    fn color(self) -> Color {
        match self {
            MagicElement::Fire => Color::rgb(1.0, 0.4, 0.1),
            MagicElement::Frost => Color::rgb(0.2, 0.7, 1.0),
            MagicElement::Wind => Color::rgb(0.6, 1.0, 0.3),
            MagicElement::Arcane => Color::rgb(0.7, 0.2, 1.0),
        }
    }
}

#[derive(Debug)]
enum MagicCast {
    FlameWave,
    CrystalShield,
    StormOrb,
    ChaosBolt,
    WildSpark,
}

impl MagicCast {
    fn label(&self) -> &'static str {
        match self {
            MagicCast::FlameWave => "Vague de Flammes",
            MagicCast::CrystalShield => "Bouclier de Cristal",
            MagicCast::StormOrb => "Orbe de Tempête",
            MagicCast::ChaosBolt => "Projectile du Chaos",
            MagicCast::WildSpark => "Étincelle Sauvage",
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct MagicMaterial {
    #[uniform(0)]
    color: Vec4,
    #[uniform(1)]
    time: f32,
    #[uniform(2)]
    light_position: Vec4,
    #[uniform(3)]
    light_color: Vec4,
    #[uniform(4)]
    light_radius: f32,
}

impl Material2d for MagicMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/magic.wgsl".into()
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct SunRaysMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(1)]
    sun_position: Vec4,
    #[uniform(2)]
    sun_color: Vec4,
    #[uniform(3)]
    sun_radius: f32,
    #[uniform(4)]
    sun_intensity: f32,
}

impl Material2d for SunRaysMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sun_rays.wgsl".into()
    }
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MagicMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), FollowCamera));
    commands.insert_resource(SunLight {
        position: Vec2::new(-1100.0, 420.0),
    });

    let player_material = materials.add(MagicMaterial {
        color: Vec4::new(0.82, 0.36, 0.12, 1.0),
        time: 0.0,
        light_position: Vec4::new(-200.0, -150.0, 0.0, 0.0),
        light_color: Vec4::new(1.0, 0.72, 0.36, 0.0),
        light_radius: 260.0,
    });

    commands.spawn(
        (MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(64.0, 96.0)))).into(),
            material: player_material,
            transform: Transform::from_translation(Vec3::new(-200.0, -150.0, 0.0)),
            ..default()
        }, Player, Velocity(Vec3::ZERO), Name::new("Player"), ShadowCaster { size: Vec2::new(64.0, 96.0) })
    );
    commands.insert_resource(SceneState(GameScene::Lobby));

    let floor_material = color_materials.add(ColorMaterial::from(Color::rgb(0.2, 0.2, 0.25)));
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(1280.0, 40.0)))).into(),
            material: floor_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
            ..default()
        }
    )
    .insert(Platform { size: Vec2::new(1280.0, 40.0), kind: PlatformKind::Solid })
    .insert(LobbyMarker);

    commands.spawn(
        (
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(300.0, 24.0)))).into(),
                material: floor_material,
                transform: Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
                ..default()
            },
            Platform { size: Vec2::new(300.0, 24.0), kind: PlatformKind::OneWay },
            LobbyMarker,
            ShadowCaster { size: Vec2::new(300.0, 24.0) },
        ),
    );

    commands.spawn(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        text: Text::from_section(
            "Lobby : appuie sur Entrée pour lancer le donjon.",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    })
    .insert(LobbyMarker);

    commands.insert_resource(MagicState::default());
    commands.insert_resource(DashState::default());
    commands.insert_resource(RadialMenuState::default());
}

fn lobby_dungeon_transition_system(
    keyboard: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut scene_state: ResMut<SceneState>,
    mut sun_light: ResMut<SunLight>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut sun_rays_materials: ResMut<Assets<SunRaysMaterial>>,
    lobby_query: Query<Entity, With<LobbyMarker>>,
    dungeon_query: Query<Entity, With<DungeonMarker>>,
) {
    if scene_state.0 == GameScene::Lobby && keyboard.just_pressed(KeyCode::Return) {
        for entity in lobby_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let entrance = spawn_dungeon_scene(&mut commands, &mut meshes, &mut color_materials, &mut sun_rays_materials);
        sun_light.position = Vec2::new(-1100.0, 420.0);
        if let Ok((mut transform, mut velocity)) = player_query.get_single_mut() {
            transform.translation = entrance;
            velocity.0 = Vec3::ZERO;
        }
        scene_state.0 = GameScene::Dungeon;
        return;
    }

    if scene_state.0 == GameScene::Dungeon && keyboard.just_pressed(KeyCode::Escape) {
        for entity in dungeon_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if let Ok((mut transform, mut velocity)) = player_query.get_single_mut() {
            transform.translation = Vec3::new(-200.0, -150.0, 0.0);
            velocity.0 = Vec3::ZERO;
        }
        sun_light.position = Vec2::new(-200.0, -150.0);
        spawn_lobby_scene(&mut commands, &mut meshes, &mut color_materials);
        scene_state.0 = GameScene::Lobby;
    }
}

fn spawn_lobby_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let floor_material = color_materials.add(ColorMaterial::from(Color::rgb(0.2, 0.2, 0.25)));
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(1280.0, 40.0)))).into(),
            material: floor_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
            ..default()
        }
    )
    .insert(Platform { size: Vec2::new(1280.0, 40.0), kind: PlatformKind::Solid })
    .insert(LobbyMarker);

    commands.spawn(
        (
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(300.0, 24.0)))).into(),
                material: floor_material,
                transform: Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
                ..default()
            },
            Platform { size: Vec2::new(300.0, 24.0), kind: PlatformKind::OneWay },
            LobbyMarker,
            ShadowCaster { size: Vec2::new(300.0, 24.0) },
        ),
    );

    commands.spawn(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        text: Text::from_section(
            "Lobby : appuie sur Entrée pour lancer le donjon.",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    })
    .insert(LobbyMarker);
}

fn spawn_dungeon_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    sun_rays_materials: &mut ResMut<Assets<SunRaysMaterial>>,
) -> Vec3 {
    let background_material = color_materials.add(ColorMaterial::from(Color::rgb(0.04, 0.05, 0.08)));
    let ground_material = color_materials.add(ColorMaterial::from(Color::rgb(0.16, 0.14, 0.10)));
    let enemy_material = color_materials.add(ColorMaterial::from(Color::rgb(0.9, 0.2, 0.2)));

    let level_width = 2400.0;
    let floor_height = 40.0;

    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(level_width + 500.0, 1000.0)))).into(),
            material: background_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        }
    )
    .insert(DungeonMarker);

    let sun_overlay_material = sun_rays_materials.add(SunRaysMaterial {
        time: 0.0,
        sun_position: Vec4::new(-1100.0, 420.0, 0.0, 0.0),
        sun_color: Vec4::new(1.0, 0.74, 0.30, 1.0),
        sun_radius: 1800.0,
        sun_intensity: 0.85,
    });
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(3800.0, 3800.0)))).into(),
        material: sun_overlay_material,
        transform: Transform::from_translation(Vec3::new(-1100.0, 420.0, -4.0)),
        ..default()
    })
    .insert(DungeonMarker);

    let light_mesh = meshes.add(create_light_mesh());
    let light_material = color_materials.add(ColorMaterial::from(Color::rgba(1.0, 0.88, 0.64, 0.12)));
    commands.spawn(MaterialMesh2dBundle {
        mesh: light_mesh.clone().into(),
        material: light_material,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)),
        ..default()
    })
    .insert(DungeonMarker)
    .insert(LightMask);

    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(level_width, floor_height)))).into(),
            material: ground_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -320.0, 0.0)),
            ..default()
        }
    )
    .insert(Platform { size: Vec2::new(level_width, floor_height), kind: PlatformKind::Solid })
    .insert(DungeonMarker);

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(48.0, 72.0)))).into(),
        material: enemy_material.clone(),
        transform: Transform::from_translation(Vec3::new(220.0, -260.0, 2.0)),
        ..default()
    })
    .insert(Enemy { health: 18.0, speed: 160.0, damage: 8.0, depth: 0.0 })
    .insert(EnemyVelocity(Vec3::ZERO))
    .insert(DungeonMarker)
    .insert(ShadowCaster { size: Vec2::new(48.0, 72.0) });

    commands.spawn(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        text: Text::from_section(
            "Test terrain plat : appuie sur Échap pour revenir au lobby.",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    })
    .insert(DungeonMarker);

    Vec3::new(-level_width / 2.0 + 150.0, -260.0, 0.0)
}

fn mouse_fire_system(
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    mut magic_state: ResMut<MagicState>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    if magic_state.shoot_timer < magic_state.shoot_cooldown {
        return;
    }

    let window = if let Ok(window) = windows.get_single() {
        window
    } else {
        return;
    };

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(camera_pair) => camera_pair,
        Err(_) => return,
    };

    let cursor_position = match window.cursor_position() {
        Some(position) => position,
        None => return,
    };

    let world_pos = if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        world_pos
    } else {
        return;
    };

    magic_state.shoot_timer = 0.0;

    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();
        let mut direction = world_pos - player_pos;
        if direction.length_squared() <= 0.0 {
            return;
        }
        direction = direction.normalize();

        let projectile_material = color_materials.add(ColorMaterial::from(magic_state.selected_element.color()));
        commands.spawn(
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(12.0, 12.0)))).into(),
                material: projectile_material,
                transform: Transform::from_translation(player_transform.translation + direction.extend(0.0) * 48.0 + Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            }
        )
        .insert(Projectile {
            direction,
            speed: 900.0,
            lifetime: 2.5,
        });
    }
}

fn radial_menu_system(
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut radial_state: ResMut<RadialMenuState>,
    mut magic_state: ResMut<MagicState>,
    mut ui_query: Query<(&mut BackgroundColor, &RadialMenuItem)>,
) {
    let window = if let Ok(window) = windows.get_single() {
        window
    } else {
        return;
    };

    let cursor_position = if let Some(position) = window.cursor_position() {
        position
    } else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Right) {
        radial_state.cursor_position = cursor_position;
        radial_state.selected_element = MagicElement::Fire;
        let menu = commands.spawn(
            (NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(cursor_position.x - 60.0),
                    top: Val::Px(cursor_position.y - 60.0),
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            RadialMenuRoot)).with_children(|parent| {
                parent.spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(40.0),
                        top: Val::Px(0.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(1.0, 0.4, 0.1, 0.75)),
                    ..default()
                }, RadialMenuItem(MagicElement::Fire)));
                parent.spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(80.0),
                        top: Val::Px(40.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.6, 1.0, 0.3, 0.75)),
                    ..default()
                }, RadialMenuItem(MagicElement::Wind)));
                parent.spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(40.0),
                        top: Val::Px(80.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.7, 0.2, 1.0, 0.75)),
                    ..default()
                }, RadialMenuItem(MagicElement::Arcane)));
                parent.spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(40.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.2, 0.7, 1.0, 0.75)),
                    ..default()
                }, RadialMenuItem(MagicElement::Frost)));
            }).id();
        radial_state.menu_entity = Some(menu);
        return;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        if radial_state.menu_entity.is_some() {
            let selection = pick_radial_element(cursor_position, radial_state.cursor_position);
            radial_state.selected_element = selection;
            for (mut color, item) in ui_query.iter_mut() {
                if item.0 == selection {
                    color.0.set_a(1.0);
                } else {
                    color.0.set_a(0.4);
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Right) {
        magic_state.selected_element = radial_state.selected_element;
        if let Some(menu_entity) = radial_state.menu_entity {
            commands.entity(menu_entity).despawn_recursive();
            radial_state.menu_entity = None;
        }
    }
}

#[derive(Component)]
struct RadialMenuRoot;

#[derive(Component)]
struct RadialMenuItem(MagicElement);

fn pick_radial_element(cursor: Vec2, center: Vec2) -> MagicElement {
    let delta = Vec2::new(cursor.x - center.x, center.y - cursor.y);
    let angle = delta.y.atan2(delta.x);
    match angle {
        a if a >= -std::f32::consts::FRAC_PI_4 && a < std::f32::consts::FRAC_PI_4 => MagicElement::Wind,
        a if a >= std::f32::consts::FRAC_PI_4 && a < 3.0 * std::f32::consts::FRAC_PI_4 => MagicElement::Fire,
        a if a >= -3.0 * std::f32::consts::FRAC_PI_4 && a < -std::f32::consts::FRAC_PI_4 => MagicElement::Arcane,
        _ => MagicElement::Frost,
    }
}

fn aabb_overlap(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    let a_min = a_center - a_half;
    let a_max = a_center + a_half;
    let b_min = b_center - b_half;
    let b_max = b_center + b_half;

    a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
}

fn projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    platforms: Query<(&GlobalTransform, &Platform), Without<Player>>,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        transform.translation += projectile.direction.extend(0.0) * projectile.speed * time.delta_seconds();
        projectile.lifetime -= time.delta_seconds();

        let pos = transform.translation.truncate();
        let projectile_half = Vec2::splat(6.0);
        let mut hit_wall = false;

        for (platform_transform, platform) in platforms.iter() {
            let platform_center = platform_transform.translation().truncate();
            let platform_half = platform.size / 2.0;
            if aabb_overlap(pos, projectile_half, platform_center, platform_half) {
                hit_wall = true;
                break;
            }
        }

        let needs_despawn = projectile.lifetime <= 0.0
            || pos.x.abs() > 2000.0
            || pos.y.abs() > 2000.0
            || hit_wall;

        if needs_despawn {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_enemy_collision_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    for (projectile_entity, projectile_transform, _) in projectiles.iter() {
        let pos = projectile_transform.translation.truncate();
        let projectile_half = Vec2::splat(6.0);
        let mut hit_enemy = None;

        for (enemy_entity, enemy_transform, mut enemy) in enemies.iter_mut() {
            let enemy_center = enemy_transform.translation.truncate();
            let enemy_scale = enemy_transform.scale.truncate();
            let enemy_half = Vec2::new(24.0, 36.0) * enemy_scale;
            if aabb_overlap(pos, projectile_half, enemy_center, enemy_half) {
                enemy.health -= 12.0;
                if enemy.health <= 0.0 {
                    hit_enemy = Some(enemy_entity);
                }
                break;
            }
        }

        if let Some(enemy_entity) = hit_enemy {
            commands.entity(enemy_entity).despawn();
            commands.entity(projectile_entity).despawn();
        }
    }
}

fn enemy_movement_system(
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&mut Transform, &mut Enemy, &mut EnemyVelocity), Without<Player>>,
    )>,
    platforms: Query<(&Transform, &Platform), (Without<Player>, Without<Enemy>)>,
) {
    let player_query = query_set.p0();
    let player_transform = if let Ok(transform) = player_query.get_single() {
        transform
    } else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut enemy_transform, enemy, mut velocity) in query_set.p1().iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);
        let chase_radius = 300.0 + enemy.damage * 6.0;
        if distance < chase_radius {
            let direction = (player_pos - enemy_pos).normalize_or_zero();
            velocity.0.x = direction.x * enemy.speed;
        } else {
            velocity.0.x = 0.0;
        }

        velocity.0.y -= 1400.0 * time.delta_seconds();

        let enemy_half = Vec2::new(24.0, 36.0);
        let previous_bottom = enemy_transform.translation.y - enemy_half.y;
        let mut next_translation = enemy_transform.translation + velocity.0 * time.delta_seconds();
        let mut next_bottom = next_translation.y - enemy_half.y;

        for (platform_transform, platform) in platforms.iter() {
            let platform_center = platform_transform.translation.truncate();
            let platform_half = platform.size / 2.0;
            let enemy_center = next_translation.truncate();

            if !aabb_overlap(enemy_center, enemy_half, platform_center, platform_half) {
                continue;
            }

            let overlap_x = (enemy_half.x + platform_half.x)
                - (enemy_center.x - platform_center.x).abs();
            let overlap_y = (enemy_half.y + platform_half.y)
                - (enemy_center.y - platform_center.y).abs();

            if platform.kind == PlatformKind::OneWay {
                let platform_top = platform_center.y + platform_half.y;
                if velocity.0.y <= 0.0 && previous_bottom >= platform_top - 2.0 && next_bottom <= platform_top {
                    next_translation.y = platform_top + enemy_half.y;
                    velocity.0.y = 0.0;
                    next_bottom = next_translation.y - enemy_half.y;
                }
                continue;
            }

            if overlap_x < overlap_y {
                if velocity.0.x > 0.0 {
                    next_translation.x -= overlap_x;
                } else if velocity.0.x < 0.0 {
                    next_translation.x += overlap_x;
                }
                velocity.0.x = 0.0;
            } else {
                if velocity.0.y > 0.0 {
                    next_translation.y -= overlap_y;
                } else if velocity.0.y < 0.0 {
                    next_translation.y += overlap_y;
                }
                velocity.0.y = 0.0;
                next_bottom = next_translation.y - enemy_half.y;
            }
        }

        enemy_transform.translation = next_translation;
        let scale = 1.0 + enemy.depth * 0.35;
        enemy_transform.scale = Vec3::splat(scale);
    }
}

fn player_physics_system(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut dash_state: ResMut<DashState>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    platforms: Query<(&Transform, &Platform), Without<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
        let mut vel = velocity.0;
        let mut direction = 0.0;
        if keyboard.pressed(KeyCode::Q) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::D) {
            direction += 1.0;
        }

        let player_half = Vec2::new(32.0, 48.0);
        let current_bottom = transform.translation.y - player_half.y;
        let mut grounded = false;
        for (platform_transform, platform) in platforms.iter() {
            let platform_top = platform_transform.translation.y + platform.size.y / 2.0;
            let left = platform_transform.translation.x - platform.size.x / 2.0 - player_half.x;
            let right = platform_transform.translation.x + platform.size.x / 2.0 + player_half.x;
            if current_bottom >= platform_top - 2.0
                && current_bottom <= platform_top + 2.0
                && transform.translation.x >= left
                && transform.translation.x <= right
            {
                grounded = true;
                break;
            }
        }

        let dropping = keyboard.pressed(KeyCode::S);

        if keyboard.just_pressed(KeyCode::Space) && grounded {
            vel.y = 760.0;
            grounded = false;
        }

        dash_state.cooldown_timer += time.delta_seconds();
        if dash_state.active {
            dash_state.timer -= time.delta_seconds();
            if dash_state.timer <= 0.0 {
                dash_state.active = false;
            }
        }

        if (keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::ShiftRight))
            && dash_state.cooldown_timer >= dash_state.cooldown
        {
            if direction != 0.0 {
                dash_state.active = true;
                dash_state.timer = dash_state.duration;
                dash_state.cooldown_timer = 0.0;
            }
        }

        let base_speed = if grounded { 320.0 } else { 220.0 };
        let dash_speed = 900.0;
        if dash_state.active {
            vel.x = direction * dash_speed;
        } else {
            vel.x = direction * base_speed;
        }

        let fall_multiplier = if keyboard.pressed(KeyCode::S) { 2.4 } else { 1.0 };
        vel.y -= 1400.0 * fall_multiplier * time.delta_seconds();

        let previous_bottom = current_bottom;
        let mut next_translation = transform.translation + vel * time.delta_seconds();
        let mut next_bottom = next_translation.y - player_half.y;

        let delta = next_translation - transform.translation;
        for (platform_transform, platform) in platforms.iter() {
            let platform_center = platform_transform.translation.truncate();
            let platform_half = platform.size / 2.0;
            let player_center = next_translation.truncate();

            if !aabb_overlap(player_center, player_half, platform_center, platform_half) {
                continue;
            }

            let overlap_x = (player_half.x + platform_half.x)
                - (player_center.x - platform_center.x).abs();
            let overlap_y = (player_half.y + platform_half.y)
                - (player_center.y - platform_center.y).abs();

            if platform.kind == PlatformKind::OneWay {
                if dropping {
                    continue;
                }
                let platform_top = platform_center.y + platform_half.y;
                if vel.y <= 0.0 && previous_bottom >= platform_top - 2.0 && next_bottom <= platform_top {
                    next_translation.y = platform_top + player_half.y;
                    vel.y = 0.0;
                    continue;
                }
                continue;
            }

            if overlap_x < overlap_y {
                if delta.x > 0.0 {
                    next_translation.x -= overlap_x;
                } else if delta.x < 0.0 {
                    next_translation.x += overlap_x;
                }
                vel.x = 0.0;
            } else {
                if delta.y > 0.0 {
                    next_translation.y -= overlap_y;
                } else if delta.y < 0.0 {
                    next_translation.y += overlap_y;
                }
                vel.y = 0.0;
                next_bottom = next_translation.y - player_half.y;
            }
        }

        transform.translation = next_translation;
        velocity.0 = vel;
    }
}

fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let mut offset = Vec2::ZERO;
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_position) = window.cursor_position() {
                    let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
                    let screen_delta = (cursor_position - center) / center;
                    offset.x = screen_delta.x * 180.0;
                    offset.y = -screen_delta.y * 70.0;
                }
            }

            camera_transform.translation.x = player_transform.translation.x + offset.x;
            camera_transform.translation.y = (player_transform.translation.y + offset.y).clamp(-120.0, 120.0);
        }
    }
}

fn cooldown_system(
    time: Res<Time>,
    mut magic_state: ResMut<MagicState>,
) {
    magic_state.shoot_timer += time.delta_seconds();
}

fn magic_input_system(
    keyboard: Res<Input<KeyCode>>,
    mut magic_state: ResMut<MagicState>,
) {
    if keyboard.just_pressed(KeyCode::E) {
        magic_state.combo.push(MagicInput::Fire);
    }
    if keyboard.just_pressed(KeyCode::F) {
        magic_state.combo.push(MagicInput::Frost);
    }
    if keyboard.just_pressed(KeyCode::G) {
        magic_state.combo.push(MagicInput::Wind);
    }
    if keyboard.just_pressed(KeyCode::H) {
        magic_state.combo.push(MagicInput::Arcane);
    }

    if !magic_state.combo.is_empty() {
        let combo_text = magic_state
            .combo
            .iter()
            .map(|input| input.label())
            .collect::<Vec<_>>()
            .join(" + ");
        println!("Combo en cours : {}", combo_text);
    }

}

fn calculate_magic(combo: &[MagicInput]) -> MagicCast {
    match combo {
        [MagicInput::Fire, MagicInput::Wind] => MagicCast::FlameWave,
        [MagicInput::Frost, MagicInput::Arcane] => MagicCast::CrystalShield,
        [MagicInput::Wind, MagicInput::Arcane] => MagicCast::StormOrb,
        [MagicInput::Fire, MagicInput::Arcane] => MagicCast::ChaosBolt,
        [MagicInput::Fire, MagicInput::Frost, MagicInput::Wind] => MagicCast::WildSpark,
        _ => MagicCast::ChaosBolt,
    }
}

fn update_magic_material_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<MagicMaterial>>,
    query: Query<&Handle<MagicMaterial>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_position = if let Ok(transform) = player_query.get_single() {
        transform.translation
    } else {
        Vec3::ZERO
    };

    for handle in query.iter() {
        if let Some(material) = materials.get_mut(handle) {
            material.time += time.delta_seconds();
            material.light_position = player_position.extend(0.0);
            material.light_radius = 260.0;
        }
    }
}

fn update_sun_rays_system(
    time: Res<Time>,
    sun_light: Res<SunLight>,
    mut materials: ResMut<Assets<SunRaysMaterial>>,
) {
    for material in materials.iter_mut() {
        material.1.time += time.delta_seconds();
        material.1.sun_position = Vec4::new(sun_light.position.x, sun_light.position.y, 0.0, 0.0);
    }
}

fn update_light_mask_system(
    sun_light: Res<SunLight>,
    casters: Query<(&ShadowCaster, &GlobalTransform), With<ShadowCaster>>,
    platforms: Query<(&Platform, &GlobalTransform), With<Platform>>,
    mut light_masks: Query<(&LightMask, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let origin = sun_light.position;
    let mut obstacles = Vec::new();
    for (caster, transform) in casters.iter() {
        obstacles.push((transform.translation().truncate(), caster.size));
    }
    for (platform, transform) in platforms.iter() {
        obstacles.push((transform.translation().truncate(), platform.size));
    }

    let mut angles = Vec::new();
    let ray_count = 64;
    for i in 0..ray_count {
        angles.push(i as f32 / ray_count as f32 * std::f32::consts::TAU);
    }

    for (center, size) in obstacles.iter() {
        let half = *size * 0.5;
        for dx in [-half.x, half.x] {
            for dy in [-half.y, half.y] {
                let corner = *center + Vec2::new(dx, dy);
                let direction = corner - origin;
                if direction.length_squared() < 0.0001 {
                    continue;
                }
                let angle = f32::atan2(direction.y, direction.x);
                angles.push(angle - 0.0001);
                angles.push(angle);
                angles.push(angle + 0.0001);
            }
        }
    }

    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
    angles.dedup_by(|a, b| (*a - *b).abs() < 0.0001);

    let max_distance = 1400.0;
    let mut hit_points = Vec::with_capacity(angles.len());
    for angle in angles.iter() {
        let direction = Vec2::new(angle.cos(), angle.sin());
        hit_points.push(cast_light_ray(origin, direction, &obstacles, max_distance));
    }

    for (_light_mask, mesh_handle) in light_masks.iter_mut() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            let mut positions = Vec::with_capacity(hit_points.len() + 1);
            positions.push([origin.x, origin.y, 0.0]);
            for point in hit_points.iter() {
                positions.push([point.x, point.y, 0.0]);
            }

            let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
            let uvs = vec![[0.0, 0.0]; positions.len()];
            let mut indices = Vec::new();
            for i in 1..positions.len() - 1 {
                indices.push(0u32);
                indices.push(i as u32);
                indices.push((i + 1) as u32);
            }

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.set_indices(Some(Indices::U32(indices)));
        }
    }
}

fn cast_light_ray(origin: Vec2, direction: Vec2, obstacles: &[(Vec2, Vec2)], max_distance: f32) -> Vec2 {
    let mut distance = max_distance;
    for (center, size) in obstacles.iter() {
        if let Some(hit) = ray_aabb_intersection(origin, direction, *center, *size, distance) {
            if hit > 0.0 && hit < distance {
                distance = hit;
            }
        }
    }
    origin + direction * distance
}

fn create_light_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
    mesh
}

fn ray_aabb_intersection(origin: Vec2, direction: Vec2, obstacle_center: Vec2, size: Vec2, max_distance: f32) -> Option<f32> {
    let half = size * 0.5;
    let min = obstacle_center - half;
    let max = obstacle_center + half;
    let inv_dir = Vec2::new(
        if direction.x.abs() < 1e-6 { f32::INFINITY } else { 1.0 / direction.x },
        if direction.y.abs() < 1e-6 { f32::INFINITY } else { 1.0 / direction.y },
    );
    let mut tmin = (min.x - origin.x) * inv_dir.x;
    let mut tmax = (max.x - origin.x) * inv_dir.x;
    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }
    let mut tymin = (min.y - origin.y) * inv_dir.y;
    let mut tymax = (max.y - origin.y) * inv_dir.y;
    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }
    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }
    if tymin > tmin {
        tmin = tymin;
    }
    if tymax < tmax {
        tmax = tymax;
    }
    let t = if tmin > 0.0 { tmin } else { tmax };
    if t > 0.0 && t <= max_distance {
        Some(t)
    } else {
        None
    }
}
