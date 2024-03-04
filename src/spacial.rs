use crate::{AudioInstance, AudioTween};
use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::prelude::{GlobalTransform, Query, Res, ResMut, Resource, With};
use bevy::transform::components::Transform;

enum SoundPath {
    Direct,
    Ambient,
}

#[doc(alias = "mix")]
#[inline]
pub fn lerp(lhs: f32, rhs: f32, s: f32) -> f32 {
    lhs + ((rhs - lhs) * s)
}

/// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component, Default)]
pub struct AudioEmitter {
    /// Direct attenuation
    /// Sounds facing away, and facing away from sounds will dampen
    /// the 'direct' component of a sound
    pub self_occlusion: f32,

    /// indirect attenuation
    /// being far away from a sound will decrease its base
    /// the range is the distance at which it sounds balanced.
    pub range: f32,

    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instances: Vec<Handle<AudioInstance>>,
}

impl AudioEmitter {
    // pub fn play(&mut self, instance: AudioInstance) {
    //     let mut ambient = instance.handle
    //     ambient.
    //     self.instances.push()
    // }
}

/// Component for the audio receiver
///
/// Most likely you will want to add this component to your player or you camera.
/// The entity needs a [`Transform`] and [`GlobalTransform`]. The view direction of the [`GlobalTransform`]
/// will
#[derive(Component)]
pub struct AudioReceiver {
    /// Direct attenuation
    /// Sounds facing away, and facing away from sounds will dampen
    /// the 'direct' component of a sound
    pub self_occlusion: f32,
}

/// Configuration resource for spacial audio
///
/// If this resource is not added to the ECS, spacial audio is not applied.
#[derive(Resource)]
pub struct SpacialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}

impl SpacialAudio {
    pub(crate) fn update(
        &self,
        receiver_transform: &GlobalTransform,
        receiver: &AudioReceiver,
        emitters: &Query<(&GlobalTransform, &AudioEmitter), With<AudioEmitter>>,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        for (emitter_transform, emitter) in emitters {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let volume =  4. * emitter.range / sound_path.length();

            let direct_volume = 4. * volume * lerp(1., emitter_transform.back().dot(sound_path.normalize_or_zero()) * 0.5 + 0.5, emitter.self_occlusion) *
                                                lerp(1., receiver_transform.forward().dot(sound_path.normalize_or_zero()) * 0.5 + 0.5, receiver.self_occlusion);

            let ambient_volume = volume / sound_path.length();
            // (1. - sound_path.length() / self.max_distance)
            //     .clamp(0., 1.)
            //     .powi(2);

            let right_ear_angle = receiver_transform.right().angle_between(sound_path);
            let panning = (right_ear_angle.cos() + 1.) / 2.;

            for instance in emitter.instances.iter() {
                if let Some(instance) = audio_instances.get_mut(instance) {
                    instance.set_volume(direct_volume as f64, AudioTween::default());
                    instance.set_panning(panning as f64, AudioTween::default());
                }
            }
        }
    }
}

pub(crate) fn run_spacial_audio(
    spacial_audio: Res<SpacialAudio>,
    receiver: Query<(&GlobalTransform, &AudioReceiver), With<AudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter), With<AudioEmitter>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok((receiver_transform, receiver)) = receiver.get_single() {
        spacial_audio.update(&receiver_transform, &receiver, &emitters, &mut audio_instances);
    }
}

pub(crate) fn cleanup_stopped_spacial_instances(
    mut emitters: Query<&mut AudioEmitter>,
    instances: ResMut<Assets<AudioInstance>>,
) {
    for mut emitter in emitters.iter_mut() {
        let handles = &mut emitter.instances;

        handles.retain(|handle| {
            if let Some(instance) = instances.get(handle) {
                instance.handle.state() != kira::sound::PlaybackState::Stopped
            } else {
                true
            }
        });
    }
}
