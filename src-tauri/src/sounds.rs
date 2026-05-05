// Cross-platform system-sound playback.
//
// Resolves a portable sound ID to the OS's native system-sound file:
//   macOS:   /System/Library/Sounds/<name>.aiff   (played via `afplay -v <vol>`)
//   Windows: C:\Windows\Media\<name>.wav         (played via `rodio`)
//
// Non-blocking. Missing files or playback failures are silently ignored —
// audio is a UX nicety, never load-bearing.

#[cfg(target_os = "macos")]
fn filename(id: &str) -> Option<&'static str> {
    Some(match id {
        "Glass"     => "Glass.aiff",
        "Tink"      => "Tink.aiff",
        "Pop"       => "Pop.aiff",
        "Hero"      => "Hero.aiff",
        "Ping"      => "Ping.aiff",
        "Submarine" => "Submarine.aiff",
        "Funk"      => "Funk.aiff",
        "Bottle"    => "Bottle.aiff",
        _ => return None,
    })
}

#[cfg(target_os = "windows")]
fn filename(id: &str) -> Option<&'static str> {
    // Windows Media has no 1:1 equivalents for macOS sound names — these are
    // chosen for similar character (length + tone), not literal matches.
    Some(match id {
        "Glass"     => "chimes.wav",
        "Tink"      => "ding.wav",
        "Pop"       => "ding.wav",
        "Hero"      => "tada.wav",
        "Ping"      => "notify.wav",
        "Submarine" => "ringin.wav",
        "Funk"      => "recycle.wav",
        "Bottle"    => "chord.wav",
        _ => return None,
    })
}

#[cfg(target_os = "macos")]
pub fn play(id: &str, volume: f32) {
    let Some(name) = filename(id) else { return };
    let path = format!("/System/Library/Sounds/{name}");
    let v = volume.clamp(0.0, 1.0).to_string();
    let _ = std::process::Command::new("afplay")
        .args(["-v", &v, &path])
        .spawn();
}

#[cfg(target_os = "windows")]
pub fn play(id: &str, volume: f32) {
    let Some(name) = filename(id) else { return };
    let path = std::path::PathBuf::from(r"C:\Windows\Media").join(name);
    let volume = volume.clamp(0.0, 1.0);

    std::thread::spawn(move || {
        use rodio::{Decoder, OutputStream, Sink};

        let Ok((_stream, handle)) = OutputStream::try_default() else { return };
        let Ok(file) = std::fs::File::open(&path) else { return };
        let Ok(decoder) = Decoder::new(std::io::BufReader::new(file)) else { return };
        let Ok(sink) = Sink::try_new(&handle) else { return };
        sink.set_volume(volume);
        sink.append(decoder);
        sink.sleep_until_end();
    });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn play(_id: &str, _volume: f32) {}
