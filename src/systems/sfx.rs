//! Wave 5 -- procedural sound effects (no asset files).
//!
//! Every SFX is synthesized into a small PCM buffer at startup and exposed as a
//! custom `Decodable` Bevy audio asset (`ProceduralSound`). Gameplay systems
//! emit `SfxEvent` messages; `play_sfx` consumes them and spawns a
//! `DESPAWN`-mode `AudioPlayer<ProceduralSound>`. This is fully wasm-safe: no
//! threads, no file IO, no external crates -- just f32 sample math fed to
//! rodio's `Source` (cpal has a web backend).
//!
//! ## Browser autoplay
//! Web `AudioContext` only starts after a user gesture. `audio_gesture_gate`
//! flips an `AudioReady` flag on the first key/click; `play_sfx` stays silent
//! until then, so SFX begin working once the player interacts (which always
//! happens at the menu before gameplay). If audio can't init, playback is a
//! no-op and the game runs silently -- it never panics.

use std::time::Duration;

use bevy::audio::{AddAudioSource, Decodable, Source};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::resources::SfxEvent;
use crate::tuning;

/// Sample rate for all synthesized SFX.
const SAMPLE_RATE: u32 = 44_100;

/// A procedurally-generated mono sound: a fixed buffer of f32 samples played
/// once. Implements `Decodable` so Bevy can register + play it like any asset.
#[derive(Asset, Debug, Clone, TypePath)]
pub struct ProceduralSound {
    samples: std::sync::Arc<[f32]>,
}

/// A rodio `Source` that walks a `ProceduralSound`'s samples once.
pub struct ProceduralDecoder {
    samples: std::sync::Arc<[f32]>,
    index: usize,
}

impl Iterator for ProceduralDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = self.samples.get(self.index).copied();
        self.index += 1;
        s
    }
}

impl Source for ProceduralDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.index))
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / SAMPLE_RATE as f32,
        ))
    }
}

impl Decodable for ProceduralSound {
    type DecoderItem = f32;
    type Decoder = ProceduralDecoder;
    fn decoder(&self) -> Self::Decoder {
        ProceduralDecoder {
            samples: self.samples.clone(),
            index: 0,
        }
    }
}

/// Pre-built handles to every SFX, keyed by `SfxEvent`. Built once at startup.
#[derive(Resource)]
pub struct SfxHandles {
    melee_swing: Handle<ProceduralSound>,
    hit: Handle<ProceduralSound>,
    enemy_death: Handle<ProceduralSound>,
    player_hurt: Handle<ProceduralSound>,
    dash: Handle<ProceduralSound>,
    pickup: Handle<ProceduralSound>,
    boon: Handle<ProceduralSound>,
    explosion: Handle<ProceduralSound>,
    boss_attack: Handle<ProceduralSound>,
    floor_clear: Handle<ProceduralSound>,
}

impl SfxHandles {
    fn handle_and_volume(&self, event: SfxEvent) -> (Handle<ProceduralSound>, f32) {
        match event {
            SfxEvent::MeleeSwing => (self.melee_swing.clone(), tuning::SFX_VOL_MELEE),
            SfxEvent::Hit => (self.hit.clone(), tuning::SFX_VOL_HIT),
            SfxEvent::EnemyDeath => (self.enemy_death.clone(), tuning::SFX_VOL_ENEMY_DEATH),
            SfxEvent::PlayerHurt => (self.player_hurt.clone(), tuning::SFX_VOL_PLAYER_HURT),
            SfxEvent::Dash => (self.dash.clone(), tuning::SFX_VOL_DASH),
            SfxEvent::Pickup => (self.pickup.clone(), tuning::SFX_VOL_PICKUP),
            SfxEvent::BoonSelect => (self.boon.clone(), tuning::SFX_VOL_BOON),
            SfxEvent::Explosion => (self.explosion.clone(), tuning::SFX_VOL_EXPLOSION),
            SfxEvent::BossAttack => (self.boss_attack.clone(), tuning::SFX_VOL_BOSS_ATTACK),
            SfxEvent::FloorClear => (self.floor_clear.clone(), tuning::SFX_VOL_FLOOR_CLEAR),
        }
    }
}

/// Tracks whether a user gesture has unlocked audio (browser autoplay policy).
#[derive(Resource, Default)]
pub struct AudioReady(pub bool);

// ---------------------------------------------------------------------------
// Synthesis helpers (pure sample math -- wasm-safe)
// ---------------------------------------------------------------------------

/// Waveshape used by the tone synth.
#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Triangle,
    Saw,
}

fn osc(wave: Wave, phase: f32) -> f32 {
    // phase in turns (0..1)
    let p = phase.fract();
    match wave {
        Wave::Sine => (p * std::f32::consts::TAU).sin(),
        Wave::Square => {
            if p < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Triangle => 4.0 * (p - 0.5).abs() - 1.0,
        Wave::Saw => 2.0 * p - 1.0,
    }
}

/// A simple xorshift PRNG so noise synthesis doesn't depend on `rand` (and is
/// deterministic per call).
struct Rng(u32);
impl Rng {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        // map to -1..1
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Builds a tone: `start_freq`->`end_freq` glide, given waveshape, with a short
/// attack and exponential decay envelope, scaled by `amp`.
fn tone(
    samples: &mut Vec<f32>,
    duration: f32,
    start_freq: f32,
    end_freq: f32,
    wave: Wave,
    amp: f32,
    decay: f32,
) {
    let n = (duration * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let freq = start_freq + (end_freq - start_freq) * t;
        phase += freq / SAMPLE_RATE as f32;
        // 5ms attack, exponential decay tail.
        let attack = (t / 0.05).min(1.0);
        let env = attack * (-decay * t).exp();
        let idx = i;
        let sample = osc(wave, phase) * env * amp;
        if idx < samples.len() {
            samples[idx] += sample;
        } else {
            samples.push(sample);
        }
    }
}

/// Builds a noise burst with an exponential decay envelope.
fn noise(samples: &mut Vec<f32>, duration: f32, amp: f32, decay: f32, seed: u32) {
    let n = (duration * SAMPLE_RATE as f32) as usize;
    let mut rng = Rng(seed | 1);
    for i in 0..n {
        let t = i as f32 / n as f32;
        let env = (-decay * t).exp();
        let sample = rng.next_f32() * env * amp;
        if i < samples.len() {
            samples[i] += sample;
        } else {
            samples.push(sample);
        }
    }
}

/// Soft-clip the whole buffer to [-1, 1] so layered tones don't harshly clip.
fn finalize(mut samples: Vec<f32>) -> ProceduralSound {
    for s in samples.iter_mut() {
        *s = s.clamp(-1.0, 1.0);
    }
    ProceduralSound {
        samples: samples.into(),
    }
}

fn make(build: impl FnOnce(&mut Vec<f32>)) -> ProceduralSound {
    let mut samples: Vec<f32> = Vec::new();
    build(&mut samples);
    finalize(samples)
}

// ---------------------------------------------------------------------------
// Startup: synthesize + register every SFX
// ---------------------------------------------------------------------------

/// Startup system: synthesize each SFX and store its handle. The audio assets
/// are registered with `add_audio_source::<ProceduralSound>()` in `main`.
pub fn setup_sfx(mut commands: Commands, mut assets: ResMut<Assets<ProceduralSound>>) {
    // Melee swing: a quick airy downward "whoosh" (filtered noise + low tone).
    let melee = make(|s| {
        noise(s, 0.12, 0.35, 9.0, 0x1234_5678);
        tone(s, 0.12, 420.0, 160.0, Wave::Sine, 0.25, 8.0);
    });
    // Hit/impact: short bright tick (square) + a touch of noise.
    let hit = make(|s| {
        tone(s, 0.07, 880.0, 320.0, Wave::Square, 0.4, 22.0);
        noise(s, 0.05, 0.25, 30.0, 0x90ab_cdef);
    });
    // Enemy death: a descending "splat" (saw glide down + noise tail).
    let enemy_death = make(|s| {
        tone(s, 0.22, 300.0, 80.0, Wave::Saw, 0.45, 9.0);
        noise(s, 0.18, 0.3, 11.0, 0x0f0f_0f0f);
    });
    // Player hurt: harsh low buzzer (square, low freq).
    let player_hurt = make(|s| {
        tone(s, 0.22, 180.0, 110.0, Wave::Square, 0.5, 7.0);
        tone(s, 0.22, 90.0, 70.0, Wave::Triangle, 0.3, 6.0);
    });
    // Dash: fast upward whoosh (rising sine + noise).
    let dash = make(|s| {
        tone(s, 0.16, 300.0, 900.0, Wave::Sine, 0.35, 7.0);
        noise(s, 0.12, 0.2, 12.0, 0xdead_beef);
    });
    // Pickup: bright two-note "ding" (triangle).
    let pickup = make(|s| {
        tone(s, 0.08, 880.0, 880.0, Wave::Triangle, 0.4, 10.0);
        let mut tail: Vec<f32> = Vec::new();
        tone(&mut tail, 0.14, 1320.0, 1320.0, Wave::Triangle, 0.4, 9.0);
        // Offset the second note so it arpeggiates.
        let off = (0.07 * SAMPLE_RATE as f32) as usize;
        for (i, v) in tail.into_iter().enumerate() {
            let idx = off + i;
            if idx < s.len() {
                s[idx] += v;
            } else {
                s.push(v);
            }
        }
    });
    // Boon select: a confident rising 3-note arpeggio.
    let boon = make(|s| {
        let notes = [523.25_f32, 659.25, 783.99]; // C5 E5 G5
        for (k, &f) in notes.iter().enumerate() {
            let mut part: Vec<f32> = Vec::new();
            tone(&mut part, 0.16, f, f, Wave::Triangle, 0.4, 6.0);
            let off = (k as f32 * 0.08 * SAMPLE_RATE as f32) as usize;
            for (i, v) in part.into_iter().enumerate() {
                let idx = off + i;
                if idx < s.len() {
                    s[idx] += v;
                } else {
                    s.push(v);
                }
            }
        }
    });
    // Explosion: big low noise boom + sub rumble.
    let explosion = make(|s| {
        noise(s, 0.5, 0.7, 5.0, 0xcafe_babe);
        tone(s, 0.5, 110.0, 40.0, Wave::Sine, 0.5, 4.0);
    });
    // Boss attack: ominous low growl (detuned saws).
    let boss_attack = make(|s| {
        tone(s, 0.35, 140.0, 90.0, Wave::Saw, 0.4, 4.5);
        tone(s, 0.35, 146.0, 94.0, Wave::Saw, 0.35, 4.5);
        noise(s, 0.2, 0.2, 8.0, 0xb055_a77a);
    });
    // Floor clear: a brighter resolving 4-note fanfare.
    let floor_clear = make(|s| {
        let notes = [523.25_f32, 659.25, 783.99, 1046.5]; // C5 E5 G5 C6
        for (k, &f) in notes.iter().enumerate() {
            let mut part: Vec<f32> = Vec::new();
            tone(&mut part, 0.2, f, f, Wave::Triangle, 0.4, 5.0);
            let off = (k as f32 * 0.1 * SAMPLE_RATE as f32) as usize;
            for (i, v) in part.into_iter().enumerate() {
                let idx = off + i;
                if idx < s.len() {
                    s[idx] += v;
                } else {
                    s.push(v);
                }
            }
        }
    });

    commands.insert_resource(SfxHandles {
        melee_swing: assets.add(melee),
        hit: assets.add(hit),
        enemy_death: assets.add(enemy_death),
        player_hurt: assets.add(player_hurt),
        dash: assets.add(dash),
        pickup: assets.add(pickup),
        boon: assets.add(boon),
        explosion: assets.add(explosion),
        boss_attack: assets.add(boss_attack),
        floor_clear: assets.add(floor_clear),
    });
    commands.insert_resource(AudioReady::default());
}

/// Registers the custom audio source type on the app. Called from `main`.
pub fn register_sfx_assets(app: &mut App) {
    app.add_audio_source::<ProceduralSound>();
}

// ---------------------------------------------------------------------------
// Runtime: gesture gate + playback
// ---------------------------------------------------------------------------

/// Flips `AudioReady` on the first key press or mouse click, so we never try to
/// play before the browser `AudioContext` is unlocked (browser autoplay policy).
/// On native this fires on the first input too; either way audio is silent until
/// the player interacts, which always happens at the menu before gameplay.
pub fn audio_gesture_gate(
    mut ready: ResMut<AudioReady>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if ready.0 {
        return;
    }
    if keyboard.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some() {
        ready.0 = true;
    }
}

/// Consumes `SfxEvent` messages and plays the matching procedural sound (once,
/// then despawns). Silent until a user gesture has unlocked audio. If the SFX
/// resources somehow aren't present, this is a no-op (the game runs silently and
/// never panics).
pub fn play_sfx(
    mut commands: Commands,
    mut events: MessageReader<SfxEvent>,
    handles: Option<Res<SfxHandles>>,
    ready: Option<Res<AudioReady>>,
) {
    let ready = ready.map(|r| r.0).unwrap_or(false);
    let Some(handles) = handles else {
        events.clear();
        return;
    };
    for event in events.read() {
        if !ready {
            continue;
        }
        let (handle, vol) = handles.handle_and_volume(*event);
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::DESPAWN
                .with_volume(bevy::audio::Volume::Linear(vol * tuning::SFX_MASTER_VOLUME)),
        ));
    }
}
