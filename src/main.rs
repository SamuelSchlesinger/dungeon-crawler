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
use resources::{Juice, SfxEvent};
use state::GameState;
use systems::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_state::<GameState>()
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        // Wave 5 juice + audio plumbing.
        .init_resource::<Juice>()
        .add_message::<SfxEvent>()
        .add_systems(Startup, setup_sfx)
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
            boon_select.run_if(in_state(GameState::BoonSelect)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (hud, minimap, objective_arrow).run_if(in_state(GameState::Playing)),
        )
        // Wave 5: the screen-flash overlay draws in every state (no-op when no
        // flash is active) so the floor-transition fade reads across the
        // BoonSelect -> Playing handoff and hurt/explosion flashes show in play.
        .add_systems(EguiPrimaryContextPass, draw_screen_flash)
        .add_systems(OnEnter(GameState::Playing), (setup_play, floor_transition_flash))
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
                // Player melee/ranged/AoE + enemy AI (after the player moved).
                player_attack.after(move_player),
                enemy_move.after(move_player),
                enemy_attack.after(enemy_move),
                // Wave 4 special enemy behaviors. Run after enemy_move so the
                // default mover and these never alias the same enemy WorldPos in
                // one system; charger/boss take over position only during their
                // own active phases (enemy_move yields via the self_driven check).
                archer_shoot.after(enemy_move),
                charger_ai.after(enemy_move),
                bomber_ai.after(enemy_move),
                boss_ai.after(enemy_move),
                // Projectiles (player ranged + enemy/archer/boss shots) travel +
                // resolve hits after everything that may have spawned them.
                move_projectiles
                    .after(player_attack)
                    .after(archer_shoot)
                    .after(boss_ai),
                // Resolve pending bomber/explosion AoE after the bomber fuse +
                // any death-detonations are queued.
                resolve_explosions.after(bomber_ai),
                // Camera follows the post-move player position, then applies the
                // Wave 5 screen-shake offset. `move_camera` (manual arrow-key
                // pan) runs BEFORE follow so follow always applies the shake
                // last, on top of the final anchor (avoids offset drift).
                move_camera.after(move_player),
                follow.after(move_player).after(move_camera),
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                // Pickups / stat application.
                health,
                pickup_weapon,
                // Wave 4 room hazards: DoT for actors on hazard tiles. After the
                // movers so grid positions are synced this frame.
                hazard_tick.after(move_player).after(enemy_move),
                // Feedback + visuals.
                hit_flash,
                update_transient_visuals,
                update_particles,
                update_damage_numbers,
                animate_sprites,
                display_health.after(enemy_move),
                // Fog of war (reads synced grid positions).
                fog_of_war.after(move_player),
                set_visibility.after(fog_of_war),
                // Reap enemies killed indirectly (thorns, explosions, hazards,
                // charger/boss damage) and award their gold, BEFORE rebuilding the
                // occupancy resource. After every system that can drop enemy HP.
                reap_dead_enemies
                    .after(player_attack)
                    .after(enemy_attack)
                    .after(move_projectiles)
                    .after(resolve_explosions)
                    .after(hazard_tick),
                // Resource sync + win/lose, after combat may have despawned.
                cleanup_dead_enemies
                    .after(player_attack)
                    .after(enemy_move)
                    .after(reap_dead_enemies),
                cleanup_collected_health.after(health),
                cleanup_weapon_drops.after(pickup_weapon),
                victory.after(cleanup_dead_enemies),
                defeat.after(enemy_attack),
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::BoonSelect), setup_boon_select)
        .add_systems(OnEnter(GameState::NextFloor), next_floor)
        .add_systems(OnEnter(GameState::Victory), on_victory)
        .add_systems(OnEnter(GameState::Defeat), on_defeat)
        .add_systems(Update, restart)
        // Wave 5 juice/audio systems that run in EVERY state (not gated to
        // Playing): the gesture gate must catch the menu click that unlocks
        // browser audio; the hit-stop must reliably restore `Time<Virtual>`
        // speed even across a state change; the flash timer + SFX pump run
        // everywhere so the boon-pick / floor-clear cues and floor fade work.
        .add_systems(
            Update,
            (
                audio_gesture_gate,
                play_sfx,
                hit_stop,
                update_screen_flash,
            ),
        );

    // Register the procedural-audio asset type (requires the audio + asset
    // plugins, added by DefaultPlugins above).
    register_sfx_assets(&mut app);

    app.run();
}
