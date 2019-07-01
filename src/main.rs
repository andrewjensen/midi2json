use std::fs;
use clap::{
    Arg,
    App
};
use serde::{Serialize};
use serde_json;
use midly::{
    SmfBuffer,
    Event,
    EventKind,
    MidiMessage
};

#[derive(Clone, Copy, Debug, Serialize)]
struct Note {
    time_start: f64,
    time_end: f64,
    pitch_value: u32
}

#[derive(Serialize)]
struct NoteInfo {
    notes: Vec<Note>
}

fn main() {
    let matches = App::new("midi2json")
        .author("Andrew Jensen <andrewjensen90@gmail.com>")
        .about("Converts MIDI files into note information in JSON")
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("INPUT")
            .help("Sets the input MIDI file to read")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("bpm")
            .short("b")
            .long("bpm")
            .value_name("BPM")
            .help("Sets the tempo, in beats per minute")
            .required(true)
            .takes_value(true))
        .get_matches();

    println!("Got matches!");

    let input_filename = matches.value_of("input").unwrap();
    let bpm_raw = matches.value_of("bpm").unwrap();
    let bpm = bpm_raw.parse::<f32>()
        .expect("Cannot parse BPM");

    process(input_filename, bpm);
}

fn process(input_filename: &str, bpm: f32) {

    println!("Loading MIDI file...");

    let smf_buffer = SmfBuffer::open(input_filename)
        .expect("Could not read input file");
    let smf = smf_buffer.parse_collect()
        .expect("Could not parse MIDI file contents");

    let track = &smf.tracks[0];

    println!("Handling contents...");
    let notes = get_notes(track, bpm);
    println!("Notes:");
    for note in &notes {
        println!("  {} to {}: pitch {}", note.time_start, note.time_end, note.pitch_value);
    }

    println!("Saving output JSON file...");
    create_json(&notes);

    println!("Done.");
}

fn get_notes(track: &Vec<Event>, bpm: f32) -> Vec<Note> {
    let mut notes = Vec::<Note>::new();
    let mut cur_time: u32 = 0;
    let mut cur_note: Option<Note> = None;
    for event in track {
        let delta = event.delta.as_int();
        let kind = event.kind;
        cur_time = cur_time + delta;

        if let EventKind::Midi{ message, channel: _ } = kind {
            match message {
                MidiMessage::NoteOn(pitch, _) => {
                    cur_note = Some(Note {
                        pitch_value: pitch.as_int() as u32,
                        time_start: get_time_seconds(cur_time, bpm),
                        time_end: 0.0
                    });
                },
                MidiMessage::NoteOff(_, _) => {
                    let partial_note = cur_note.unwrap();
                    let updated_note = Note {
                        pitch_value: partial_note.pitch_value,
                        time_start: partial_note.time_start,
                        time_end: get_time_seconds(cur_time, bpm)
                    };

                    notes.push(updated_note);
                },
                _ => {}
            }
        }
    }

    notes
}

fn create_json(notes: &Vec<Note>) {
    let note_info = NoteInfo {
        notes: notes.clone()
    };

    let json_str = serde_json::to_string_pretty(&note_info).unwrap();
    fs::write("output/notes.json", json_str)
        .expect("Failed to save event frames");
}

fn get_time_seconds(ticks: u32, bpm: f32) -> f64 {
    // TODO:
    // This magic number equals 96 / 60,
    // and 96 is the metrical unit in the file header,
    // so maybe we should calculate it that way, with 60 (60 bpm) as a constant.
    let ticks_per_sec = (bpm as f64) * 1.6;

    (ticks as f64) / ticks_per_sec
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_time_seconds_120() {
        assert_eq!(get_time_seconds(0, 120.0), 0.0);
        assert_eq!(get_time_seconds(48, 120.0), 0.25);
        assert_eq!(get_time_seconds(192, 120.0), 1.0);
    }

    #[test]
    fn test_get_time_seconds_60() {
        assert_eq!(get_time_seconds(0, 60.0), 0.0);
        assert_eq!(get_time_seconds(48, 60.0), 0.5);
        assert_eq!(get_time_seconds(96, 60.0), 1.0);
    }
}
