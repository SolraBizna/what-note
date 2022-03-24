use std::{
    io::{BufRead, stdin},
    process::Command,
};
use clap::Parser;
use once_cell::sync::Lazy;
use rand::{Rng, thread_rng};
use regex::Regex;

const NOTE_NAMES: &[&str] = &["C","C#","D","D#","E",
                              "F","F#","G","G#","A","A#","B"];
const NOTES_PER_OCTAVE: u32 = 12;
const MIDDLE_C: u32 = 60;
// A440
const BASE_NOTE: f32 = 69.0;
const BASE_FREQ: f32 = 440.0;

#[derive(Parser,Debug)]
#[clap(author = "Solra Bizna <solra@bizna.name>", version,
       about = "Test and train your musical note distinguishingmentness!")]
struct Invocation {
    /// Octave range. 1 = middle octave only. 2 = middle and below. 3 = middle
    /// and above. etc. Max = 5, min = 1.
    ///
    /// Middle C is the one that's below the A that is 440Hz, and is C3.
    #[clap(short, default_value_t = 1)]
    octaves: u32,
    /// Number of notes to test.
    #[clap(short, default_value_t = 20)]
    test_count: u32,
    /// Number of tries per note.
    #[clap(short, default_value_t = 3)]
    attempt_limit: u32,
}

enum Guess { Wrong, WrongOctave, Perfect }

fn full_note_name(note: u32) -> String {
    let octave = note / NOTES_PER_OCTAVE - 1;
    let note = note % NOTES_PER_OCTAVE;
    format!("{}{}", NOTE_NAMES[note as usize], octave)
}

fn note_name(note: u32) -> String {
    let note = note % NOTES_PER_OCTAVE;
    format!("{}", NOTE_NAMES[note as usize])
}

fn play_note(note: u32) {
    let freq = BASE_FREQ * (2.0f32).powf((note as f32 - BASE_NOTE)
                                         / (NOTES_PER_OCTAVE as f32));
    let _ = Command::new("play").arg("-q").arg("-n")
        .arg("synth").arg("1").arg("sine").arg(&format!("{}", freq))
        .arg("fade").arg("0.1").arg("1").arg("0.7").arg("vol").arg("0.6")
        .spawn().expect("failed to start playback").wait();
}

static VALID_NOTE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([ACDFG]#?|[BE])?[2-6]$"#).unwrap()
});

fn guess_note(note: u32, note_name: &str, full_note_name: &str) -> Guess {
    let mut buf = String::new();
    let stdin = stdin();
    let mut stdin = stdin.lock();
    loop {
        println!("Your guess?");
        buf.clear();
        match stdin.read_line(&mut buf) {
            Ok(_) => (),
            Err(_) => std::process::exit(0),
        }
        while buf.ends_with("\n") { buf.pop(); }
        if VALID_NOTE_PATTERN.is_match(&buf) {
            if buf == full_note_name {
                return Guess::Perfect
            }
            else if &buf[..buf.len()-1] == note_name {
                return Guess::WrongOctave
            }
            else {
                return Guess::Wrong
            }
        }
        else if buf == "?" {
            play_note(note);
        }
        else {
            println!("Please enter a note in MIDI notation (e.g. \"C#4\"), or \
                      \"?\" to repeat the\nnote playback.");
        }
    }
}

fn main() {
    let invocation = Invocation::parse();
    let octaves = invocation.octaves.min(5).max(1);
    let octaves_below = octaves/2;
    let octaves_above = (octaves+1)/2;
    let min_note = MIDDLE_C - octaves_below * NOTES_PER_OCTAVE;
    let max_note = MIDDLE_C + octaves_above * NOTES_PER_OCTAVE;
    println!(" Lowest note we'll play: {}", full_note_name(min_note));
    println!("Highest note we'll play: {}", full_note_name(max_note));
    let mut perfect_count = 0;
    let mut right_count = 0;
    let mut rng = thread_rng();
    for _ in 0 .. invocation.test_count {
        let note = rng.gen_range(min_note ..= max_note);
        println!("---");
        play_note(note);
        let note_name = note_name(note);
        let full_note_name = full_note_name(note);
        for rem_guesses in (0 .. invocation.attempt_limit).rev() {
            match guess_note(note, &note_name, &full_note_name) {
                Guess::Wrong => {
                    if rem_guesses > 1 {
                        println!("Try again ({} guesses left)", rem_guesses);
                    }
                    else if rem_guesses > 0 {
                        println!("Try again (last guess)");
                    }
                    else {
                        println!("Out of guesses.");
                        println!("The note was: {}", full_note_name);
                    }
                },
                Guess::WrongOctave => {
                    println!("You got the note right, but the octave wrong.");
                    println!("The correct answer was: {}", full_note_name);
                    right_count += 1;
                    break
                },
                Guess::Perfect => {
                    println!("Correct!");
                    perfect_count += 1;
                    break
                },
            }
        }
    }
    println!("You got {}/{} correct. Half credit for {} wrong-octave guesses.",
             perfect_count, invocation.test_count, right_count);
    let score = ((perfect_count * 2 + right_count) * 100 / invocation.test_count + 1) / 2;
    println!("Your final score: {}% = {}", score,
             match score {
                 x if x >= 100 => "S",
                 x if x >= 97 => "A+",
                 x if x >= 94 => "A",
                 x if x >= 90 => "A-",
                 x if x >= 87 => "B+",
                 x if x >= 84 => "B",
                 x if x >= 80 => "B-",
                 x if x >= 77 => "C+",
                 x if x >= 74 => "C",
                 x if x >= 70 => "C-",
                 x if x >= 67 => "D+",
                 x if x >= 64 => "D",
                 x if x >= 60 => "D-",
                 _ => "F",
             });
}
