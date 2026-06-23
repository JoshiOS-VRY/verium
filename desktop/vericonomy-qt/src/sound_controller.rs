//! Block-mined chime — soft ascending major arpeggio with gentle attack/decay.

#![allow(non_snake_case)]

use std::f32::consts::PI;
use std::pin::Pin;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::Source;

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type SoundController = super::SoundControllerRust;

        #[qinvokable]
        fn playBlockChime(self: Pin<&mut SoundController>);
    }
}

pub struct SoundControllerRust;

impl Default for SoundControllerRust {
    fn default() -> Self {
        Self
    }
}

const SAMPLE_RATE: u32 = 44_100;

/// G4 · B4 · D5 — lower and softer than the old C5/E5/G5 stack.
const CHIME_NOTES: [(f32, f32); 3] = [
    (392.00, 0.00),
    (493.88, 0.16),
    (587.33, 0.32),
];

fn soft_envelope(sample_idx: u32, attack_samples: u32, total_samples: u32) -> f32 {
    if total_samples == 0 {
        return 0.0;
    }
    let t = sample_idx as f32 / total_samples as f32;
    let attack = if attack_samples == 0 {
        1.0
    } else {
        (sample_idx as f32 / attack_samples as f32).min(1.0)
    };
    // Fast fade-in, long gentle tail — avoids the harsh "blip" of a square envelope.
    let decay = (-4.2 * t).exp();
    attack * decay
}

fn synthesize_soft_chime() -> Vec<f32> {
    let note_samples = (SAMPLE_RATE as f32 * 0.38) as u32;
    let attack_samples = (SAMPLE_RATE as f32 * 0.045) as u32;
    let tail_samples = (SAMPLE_RATE as f32 * 0.75) as u32;
    let mut mix = vec![0.0_f32; tail_samples as usize];

    for &(freq, start_sec) in &CHIME_NOTES {
        let start = (start_sec * SAMPLE_RATE as f32) as u32;
        let peak = 0.035;

        for i in 0..note_samples {
            let idx = start as usize + i as usize;
            if idx >= mix.len() {
                break;
            }
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = soft_envelope(i, attack_samples, note_samples);
            // Mostly sine with a touch of 2nd harmonic for warmth, kept very subtle.
            let fundamental = (2.0 * PI * freq * t).sin();
            let warmth = (2.0 * PI * freq * 2.0 * t).sin() * 0.08;
            mix[idx] += (fundamental + warmth) * env * peak;
        }
    }

    // Gentle limiter so overlapping tails never clip.
    for sample in &mut mix {
        *sample = sample.tanh() * 0.85;
    }
    mix
}

fn play_chime_on_worker() {
    let Ok((_stream, handle)) = rodio::OutputStream::try_default() else {
        return;
    };
    let Ok(sink) = rodio::Sink::try_new(&handle) else {
        return;
    };

    let pcm = synthesize_soft_chime();
    let src = SamplesBuffer::new(1, SAMPLE_RATE, pcm).fade_in(Duration::from_millis(8));
    sink.append(src);
    sink.sleep_until_end();
}

impl qobject::SoundController {
    pub fn playBlockChime(self: Pin<&mut Self>) {
        std::thread::spawn(play_chime_on_worker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chime_is_short_and_bounded() {
        let pcm = synthesize_soft_chime();
        assert!(!pcm.is_empty());
        assert!(pcm.len() < SAMPLE_RATE as usize);
        let peak = pcm.iter().copied().fold(0.0_f32, f32::max);
        assert!(peak <= 0.12);
    }
}
