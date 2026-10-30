//! Scales, correlation and meter-type names shared by RadarVU's detector and radar display.

/// (lo, hi) dB range of each meter type's scale — the radar rings and the radius mapping.
pub fn scale_ends(kind: i32) -> (f32, f32) {
    match kind {
        1 => (-40.0, 4.0),        // RMS (absolute dBFS)
        2 | 3 | 4 => (-24.0, 12.0), // K-12 / K-14 / K-20 (relative to their reference)
        5 => (-50.0, 5.0),        // DIN
        6 => (-36.0, 9.0),        // Nordic
        7 | 8 => (-12.0, 12.0),   // BBC / EBU
        9 => (-60.0, 0.0),        // LUFS radar
        _ => (-20.0, 3.0),        // VU
    }
}

/// Normalized 0..1 position of a dB reading on its meter's scale.
pub fn position(kind: i32, db: f32) -> f32 {
    let (lo, hi) = scale_ends(kind);
    ((db - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// Correlation coefficient in −1..1 mapped onto the phase scale: 1 → 0, 0 → −20, −1 → −39.
pub fn correlation_db(coeff: f32) -> f32 {
    let c = coeff.clamp(-1.0, 1.0);
    if c >= 0.0 { -20.0 * (1.0 - c) } else { -20.0 + c * 19.0 }
}

/// Ring label at a fraction of the scale, rounded to whole dB.
pub fn scale_label(kind: i32, frac: f32) -> String {
    let (lo, hi) = scale_ends(kind);
    let db = lo + (hi - lo) * frac;
    if db.abs() < 0.05 { "0".into() }
    else if db > 0.0 { format!("+{db:.0}") }
    else { format!("{db:.0}") }
}

/// Display name of a meter type (kind 0..9).
pub fn kind_name(kind: i32) -> &'static str {
    match kind {
        0 => "VU", 1 => "RMS", 2 => "K-12", 3 => "K-14", 4 => "K-20",
        5 => "DIN", 6 => "NORDIC", 7 => "BBC", 8 => "EBU", 9 => "LOUDNESS",
        _ => "VU",
    }
}

/// Signed dB text for the meter readout: "+0.5", "−1.2", "0.0", or "−−.−" when silent.
pub fn signed_db(v: f32) -> String {
    if v <= -100.0 { "−−.−".to_string() }
    else if v > 0.049 { format!("+{:.1}", v) }
    else if v < -0.049 { format!("−{:.1}", v.abs()) }
    else { "0.0".to_string() }
}
