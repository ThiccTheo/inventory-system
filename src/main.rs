use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use maplit::hashmap;
use std::collections::HashMap;

#[derive(Resource)]
struct SpriteSheet(Handle<TextureAtlas>);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Material;

#[derive(Component)]
struct Wood;

#[derive(Component)]
struct Stone;

// TODO: bool that says if item is final item in crafting tree
#[derive(Copy, Clone, Component, Reflect, Debug, PartialEq, Eq, Hash)]
enum InventoryItem {
    Wood = 2,
    Stone = 3,
    Axe = 4,
    Null = 0,
}

impl InventoryItem {
    fn is_null(&self) -> bool {
        matches!(*self, InventoryItem::Null)
    }
}

#[derive(Component, Reflect)]
struct Inventory {
    items: [(InventoryItem, usize); 10],
}

#[derive(PartialEq, Eq, Hash)]
struct Recipe {
    recipe: &'static str,
}

impl Recipe {
    fn new(recipe: [[InventoryItem; 3]; 3]) -> Self {
        let mut recipe: Vec<_> = recipe
            .into_iter()
            .filter(|row| !row[0].is_null() && !row[1].is_null() && !row[2].is_null()) // remove empty row
            .map(|row| row.into_iter().collect::<Vec<_>>())
            .collect();

        'outer: for col in (0..3).rev() {
            for row in recipe.iter() {
                if !row[col].is_null() {
                    continue 'outer;
                }
            }
            for row in recipe.iter_mut() {
                row.remove(col);
            }
        }

        Self {
            recipe: stringify!(recipe),
        }
    }
}

#[derive(Component)]
struct UiElement;

struct CraftingTable {
    recipes: HashMap<Recipe, InventoryItem>,
}

impl CraftingTable {
    fn new() -> Self {
        use InventoryItem::*;
        Self {
            recipes: hashmap! {
                Recipe::new([
                    [Stone, Stone, Null],
                    [Wood, Stone, Null],
                    [Wood, Null, Null],
                ]) => Axe,
            },
        }
    }
}

#[derive(Component)]
struct CraftingMenu {
    rows: usize,
    cols: usize,
}

#[derive(Component)]
struct Hotbar {
    num_slots: usize,
}

#[derive(Component)]
struct HotbarChild;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSampler::nearest_descriptor(),
        }))
        .add_plugin(WorldInspectorPlugin)
        .register_type::<Inventory>()
        .add_startup_system_to_stage(StartupStage::PreStartup, load_spritesheet)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_wood)
        .add_startup_system(spawn_stone)
        .add_startup_system(spawn_crafting_menu)
        .add_startup_system(spawn_hotbar)
        .add_system(player_movement)
        .add_system(pickup_material)
        .add_system(toggle_crafting_menu)
        .add_system(populate_hotbar)
        .run();
}

fn spawn_camera(mut cmds: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scale /= 2.;
    cmds.spawn((cam, MainCamera));
}

fn load_spritesheet(
    mut cmds: Commands,
    assets: Res<AssetServer>,
    mut tex_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let img = assets.load("spritesheet.png");
    let atlas = TextureAtlas::from_grid(img, Vec2::splat(16.), 5, 1, None, None);
    cmds.insert_resource(SpriteSheet(tex_atlases.add(atlas)));
}

fn spawn_player(mut cmds: Commands, tex_atlas: Res<SpriteSheet>) {
    cmds.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(1),
            texture_atlas: tex_atlas.0.clone(),
            transform: Transform::from_xyz(0., 0., 2.),
            ..default()
        },
        Player,
        Inventory {
            items: [(InventoryItem::Null, 0); 10],
        },
        Name::new("Player"),
    ));
}

fn spawn_wood(mut cmds: Commands, tex_atlas: Res<SpriteSheet>) {
    cmds.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: tex_atlas.0.clone(),
            transform: Transform::from_xyz(50., 50., 1.),
            ..default()
        },
        Wood,
        Material,
        InventoryItem::Wood,
        Name::new("Wood"),
    ));
}

fn spawn_stone(mut cmds: Commands, tex_atlas: Res<SpriteSheet>) {
    cmds.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(3),
            texture_atlas: tex_atlas.0.clone(),
            transform: Transform::from_xyz(-50., -50., 1.),
            ..default()
        },
        Stone,
        Material,
        InventoryItem::Stone,
        Name::new("Stone"),
    ));
}

fn spawn_hotbar(mut cmds: Commands, tex_atlas: Res<SpriteSheet>) {
    cmds.spawn((
        Name::new("Hotbar"),
        SpatialBundle::default(),
        Hotbar { num_slots: 10 },
        UiElement,
    ))
    .with_children(|parent| {
        for x in 0..10 {
            parent.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: 0,
                        ..default()
                    },
                    texture_atlas: tex_atlas.0.clone(),
                    transform: Transform::from_xyz(x as f32 * 16. - 72., -150., 3.),
                    ..default()
                },
                HotbarChild,
                UiElement,
            ));
        }
    });
}

fn spawn_crafting_menu(mut cmds: Commands, tex_atlas: Res<SpriteSheet>) {
    cmds.spawn((
        Name::new("Crafting Menu"),
        CraftingMenu { rows: 3, cols: 3 },
        SpatialBundle {
            visibility: Visibility { is_visible: false },
            ..default()
        },
        UiElement,
    ))
    .with_children(|parent| {
        for y in 0..3 {
            for x in 0..3 {
                parent.spawn((
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: 0,
                            ..default()
                        },
                        texture_atlas: tex_atlas.0.clone(),
                        transform: Transform::from_xyz(x as f32 * 16., y as f32 * 16., 1.),
                        ..default()
                    },
                    UiElement,
                ));
            }
        }
    });
}

fn toggle_crafting_menu(
    mut craft_qry: Query<&mut Visibility, With<CraftingMenu>>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok(mut visibility) = craft_qry.get_single_mut() else { return };
    if keys.just_pressed(KeyCode::LAlt) {
        visibility.is_visible = !visibility.is_visible;
    }
}

fn player_movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_qry: Query<&mut Transform, With<Player>>,
    mut cam_qry: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    mut ui_qry: Query<&mut Transform, (With<UiElement>, Without<Player>, Without<MainCamera>)>,
) {
    let offset = 100. * time.delta_seconds();
    let mut move_amount = Vec2::ZERO;
    let mut player_transform = player_qry.single_mut();
    let mut cam_transform = cam_qry.single_mut();

    if keys.pressed(KeyCode::W) {
        move_amount.y += offset;
    }
    if keys.pressed(KeyCode::A) {
        move_amount.x -= offset;
    }
    if keys.pressed(KeyCode::S) {
        move_amount.y -= offset;
    }
    if keys.pressed(KeyCode::D) {
        move_amount.x += offset;
    }

    let move_amount = move_amount.extend(0.);
    for mut ui_transform in ui_qry.iter_mut() {
        ui_transform.translation += move_amount;
    }

    player_transform.translation += move_amount;
    cam_transform.translation += move_amount;
}

fn populate_hotbar(
    player_qry: Query<&Inventory, With<Player>>,
    mut hotbar_qry: Query<&mut TextureAtlasSprite, With<HotbarChild>>,
) {
    let Ok(inventory) = player_qry.get_single() else { return };
    for (mut sprite, item) in hotbar_qry.iter_mut().zip(inventory.items.iter()) {
        sprite.index = item.0 as usize;
    }
}

fn pickup_material(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_qry: Query<(&Transform, &mut Inventory), With<Player>>,
    materials_qry: Query<(&Transform, Entity, &InventoryItem), (Without<Player>, With<Material>)>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let Ok((player_transform, mut inventory)) = player_qry.get_single_mut() else { return };
        let Some(open_slot_idx) = inventory.items.iter().position(|item| item.0 == InventoryItem::Null) else { return };
        for (material_transform, id, item_type) in materials_qry.into_iter() {
            if Vec2::distance(
                player_transform.translation.truncate(),
                material_transform.translation.truncate(),
            ) <= 8.
            {
                let mut flag = false;
                for item in inventory.items.iter_mut() {
                    if item.0 == *item_type {
                        println!("{:?} | {:?}", item.0, item_type);
                        item.1 += 1;
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    inventory.items[open_slot_idx] = (item_type.clone(), 1);
                }
                cmds.entity(id).despawn();
                return;
            }
        }
    }
}
