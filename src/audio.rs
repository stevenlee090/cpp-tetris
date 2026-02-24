use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::time::Duration;

use crate::game::AudioEvent;

const SAMPLE_RATE: u32 = 44100;

// ---------------------------------------------------------------------------
// Square-wave oscillator (single channel, f32 samples)
// ---------------------------------------------------------------------------

struct SquareWave {
    freq: f32,
    sample_pos: u64,
    total_samples: u64,
    volume: f32,
}

impl SquareWave {
    fn new(freq: f32, duration_ms: f32, volume: f32) -> Self {
        SquareWave {
            freq,
            sample_pos: 0,
            total_samples: (duration_ms * SAMPLE_RATE as f32 / 1000.0) as u64,
            volume,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.sample_pos >= self.total_samples {
            return None;
        }
        let t = self.sample_pos as f32 / SAMPLE_RATE as f32;
        let period = 1.0 / self.freq;
        let val = if (t % period) < period * 0.5 { self.volume } else { -self.volume };
        self.sample_pos += 1;
        Some(val)
    }
}

impl Source for SquareWave {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(
            self.total_samples * 1000 / SAMPLE_RATE as u64,
        ))
    }
}

// ---------------------------------------------------------------------------
// Looping 8-bit background music source
// ---------------------------------------------------------------------------

struct MusicSource {
    notes: Vec<(f32, usize)>, // (freq_hz, duration_samples); freq=0 → rest
    note_idx: usize,
    sample_pos: usize,
    volume: f32,
}

impl MusicSource {
    fn new(melody: &[(f32, f32)], bpm: f32, volume: f32) -> Self {
        let beat_samples = (60.0 / bpm * SAMPLE_RATE as f32) as usize;
        let notes = melody
            .iter()
            .map(|&(freq, beats)| (freq, ((beats * beat_samples as f32) as usize).max(1)))
            .collect();
        MusicSource { notes, note_idx: 0, sample_pos: 0, volume }
    }
}

impl Iterator for MusicSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.notes.is_empty() {
            return Some(0.0);
        }
        let (freq, dur) = self.notes[self.note_idx];

        // Short decay at the tail of each note (last 10%) for 8-bit articulation
        let fade_start = (dur as f32 * 0.90) as usize;
        let envelope = if self.sample_pos >= fade_start && dur > fade_start {
            let pos = self.sample_pos - fade_start;
            let fade_len = dur - fade_start;
            1.0 - (pos as f32 / fade_len as f32).min(1.0)
        } else {
            1.0
        };

        let val = if freq > 0.0 {
            let t = self.sample_pos as f32 / SAMPLE_RATE as f32;
            let period = 1.0 / freq;
            let sq = if (t % period) < period * 0.5 { 1.0f32 } else { -1.0f32 };
            sq * self.volume * envelope
        } else {
            0.0
        };

        self.sample_pos += 1;
        if self.sample_pos >= dur {
            self.sample_pos = 0;
            self.note_idx = (self.note_idx + 1) % self.notes.len();
        }
        Some(val)
    }
}

impl Source for MusicSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn total_duration(&self) -> Option<Duration> { None } // infinite loop
}

// ---------------------------------------------------------------------------
// Korobeiniki (Tetris Type-A) — traditional Russian folk song, public domain
// Represented as (frequency_hz, duration_in_quarter_beats)
// ---------------------------------------------------------------------------
const MELODY: &[(f32, f32)] = &[
    // Part 1
    (659.25, 1.0), // E5
    (493.88, 0.5), // B4
    (523.25, 0.5), // C5
    (587.33, 1.0), // D5
    (523.25, 0.5), // C5
    (493.88, 0.5), // B4
    (440.00, 1.0), // A4
    (440.00, 0.5), // A4
    (523.25, 0.5), // C5
    (659.25, 1.0), // E5
    (587.33, 0.5), // D5
    (523.25, 0.5), // C5
    (493.88, 1.5), // B4
    (523.25, 0.5), // C5
    (587.33, 1.0), // D5
    (659.25, 1.0), // E5
    (523.25, 1.0), // C5
    (440.00, 1.0), // A4
    (440.00, 2.0), // A4
    // Part 2
    (587.33, 1.5), // D5
    (698.46, 0.5), // F5
    (880.00, 1.0), // A5
    (783.99, 0.5), // G5
    (698.46, 0.5), // F5
    (659.25, 1.5), // E5
    (523.25, 0.5), // C5
    (659.25, 1.0), // E5
    (587.33, 0.5), // D5
    (523.25, 0.5), // C5
    (493.88, 1.0), // B4
    (493.88, 0.5), // B4
    (523.25, 0.5), // C5
    (587.33, 1.0), // D5
    (659.25, 1.0), // E5
    (523.25, 1.0), // C5
    (440.00, 1.0), // A4
    (440.00, 2.0), // A4
];

// ---------------------------------------------------------------------------
// AudioManager — owns the output stream and all active sinks
// ---------------------------------------------------------------------------

pub struct AudioManager {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    music_sink: Sink,
    sfx_sinks: Vec<Sink>,
    pub music_enabled: bool,
}

impl AudioManager {
    /// Returns `None` if no audio output device is available.
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let music_sink = Sink::try_new(&handle).ok()?;
        music_sink.append(MusicSource::new(MELODY, 160.0, 0.12));
        Some(AudioManager {
            _stream: stream,
            handle,
            music_sink,
            sfx_sinks: Vec::new(),
            music_enabled: true,
        })
    }

    // Play a sequence of square-wave notes as a fire-and-forget SFX.
    // Each tuple is (freq_hz, duration_ms, volume).
    fn play_notes(&mut self, notes: &[(f32, f32, f32)]) {
        // Drop sinks that have finished playing to avoid unbounded growth.
        self.sfx_sinks.retain(|s| !s.empty());

        let Ok(sink) = Sink::try_new(&self.handle) else { return };
        for &(freq, dur_ms, vol) in notes {
            sink.append(SquareWave::new(freq, dur_ms, vol));
        }
        self.sfx_sinks.push(sink);
    }

    pub fn play_event(&mut self, event: &AudioEvent) {
        match event {
            AudioEvent::Move => {
                self.play_notes(&[(200.0, 25.0, 0.15)]);
            }
            AudioEvent::Rotate => {
                self.play_notes(&[(330.0, 30.0, 0.18), (440.0, 30.0, 0.18)]);
            }
            AudioEvent::Lock => {
                self.play_notes(&[(130.0, 80.0, 0.20)]);
            }
            AudioEvent::HardDrop => {
                self.play_notes(&[(280.0, 35.0, 0.22), (140.0, 55.0, 0.22)]);
            }
            AudioEvent::LineClear(n) => match n {
                1 => {
                    self.play_notes(&[(523.25, 80.0, 0.22), (659.25, 100.0, 0.22)]);
                }
                2 => {
                    self.play_notes(&[
                        (523.25, 70.0, 0.22),
                        (659.25, 70.0, 0.22),
                        (783.99, 110.0, 0.22),
                    ]);
                }
                3 => {
                    self.play_notes(&[
                        (523.25, 60.0, 0.22),
                        (659.25, 60.0, 0.22),
                        (783.99, 60.0, 0.22),
                        (1046.50, 130.0, 0.22),
                    ]);
                }
                _ => {
                    // Tetris! Ascending fanfare
                    self.play_notes(&[
                        (523.25,  55.0, 0.25),
                        (659.25,  55.0, 0.25),
                        (783.99,  55.0, 0.25),
                        (1046.50, 55.0, 0.25),
                        (1318.51, 200.0, 0.25),
                    ]);
                }
            },
            AudioEvent::GameOver => {
                self.play_notes(&[
                    (440.00, 100.0, 0.20),
                    (370.00, 100.0, 0.20),
                    (311.00, 100.0, 0.20),
                    (261.63, 350.0, 0.20),
                ]);
            }
        }
    }

    pub fn toggle_music(&mut self) {
        self.music_enabled = !self.music_enabled;
        self.music_sink.set_volume(if self.music_enabled { 1.0 } else { 0.0 });
    }
}
