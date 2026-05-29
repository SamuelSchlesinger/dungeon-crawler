mod components;
mod events;
mod map;
mod maps;
mod resources;
mod state;
mod systems;
mod tuning;
mod utils;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use state::GameState;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_state::<GameState>()
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .add_systems(Startup, setup)
        .add_systems(
            EguiPrimaryContextPass,
            menu.run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            end_screen.run_if(
                in_state(GameState::Victory).or(in_state(GameState::Defeat)),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            hud.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Playing), setup_play)
        // Real-time action core. Everything is delta-time driven and runs each
        // frame in Update. Explicit ordering keeps the read-after-write chains
        // correct (input -> movement -> combat -> cleanup -> win/lose checks)
        // and avoids B0001 query conflicts where actor sets overlap.
        .add_systems(
            Update,
            (
                // Input + intent.
                track_mouse_movement,
                set_follow,
                dash.before(move_player),
                // Player real-time movement (reads dash state, syncs grid pos).
                move_player,
                // Player melee + enemy AI (after the player has moved).
                player_attack.after(move_player),
                enemy_move.after(move_player),
                enemy_attack.after(enemy_move),
                // Camera follows the post-move player position.
                follow.after(move_player),
                move_camera,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                // Pickups / stat application.
                health,
                pickup_weapon,
                // Feedback + visuals.
                hit_flash,
                update_transient_visuals,
                update_particles,
                animate_sprites,
                display_health.after(enemy_move),
                // Fog of war (reads synced grid positions).
                fog_of_war.after(move_player),
                set_visibility.after(fog_of_war),
                // Resource sync + win/lose, after combat may have despawned.
                cleanup_dead_enemies.after(player_attack).after(enemy_move),
                cleanup_collected_health.after(health),
                cleanup_weapon_drops.after(pickup_weapon),
                victory.after(cleanup_dead_enemies),
                defeat.after(enemy_attack),
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::NextFloor), next_floor)
        .add_systems(OnEnter(GameState::Victory), on_victory)
        .add_systems(OnEnter(GameState::Defeat), on_defeat)
        .add_systems(Update, restart)
        .run();
}
