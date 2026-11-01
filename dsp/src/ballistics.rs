//! Meter ballistics checks: VU (IEC 60268-17), PPM (ITU-R BS.645-2 Table 1, EBU Tech 3205),
//! holds, sample-rate and block-size invariance, silence and stereo policy.
//! Measured from the detector output, not the GUI.

use super::*;

const TAU: f32 = std::f32::consts::TAU;

fn meter(rate: f32, kind: i32) -> Meter {
    let mut m = Meter::new(rate as f64);
    m.kind = kind;
    m.reference_db = 0.0;
    m
}

/// Feeds `total` frames of `f(frame)` (same signal on both channels) in blocks of `block` frames.
fn feed(m: &mut Meter, total: usize, block: usize, mut f: impl FnMut(usize) -> f32) {
    let mut done = 0;
    let mut buf = Vec::with_capacity(block * 2);
    while done < total {
        let n = block.min(total - done);
        buf.clear();
        for i in 0..n {
            let v = f(done + i);
            buf.push(v);
            buf.push(v);
        }
        m.process_interleaved(&buf, 2);
        done += n;
    }
}

fn sine(rate: f32, hz: f32, amp: f32) -> impl FnMut(usize) -> f32 {
    move |i| amp * (TAU * hz * i as f32 / rate).sin()
}

#[test]
fn vu_step_meets_iec_60268_17_at_all_rates() {
    // 99 % in 0.3 s +/-10 %, overswing 1 % to 1.5 %.
    for rate in [44_100.0f32, 48_000.0, 96_000.0, 192_000.0] {
        let mut m = meter(rate, 0);
        let input = 1.0 / SINE_TO_RMS;
        let (mut first, mut max_x, mut n) = (None, 0.0f32, 0usize);
        while n < (0.8 * rate) as usize {
            feed(&mut m, 32, 32, |_| input);
            n += 32;
            max_x = max_x.max(m.channels[0].x);
            if first.is_none() && m.channels[0].x >= 0.99 { first = Some(n as f32 / rate); }
        }
        let rise = first.expect("never reached 99 percent");
        let over = max_x - 1.0;
        assert!((0.27..=0.33).contains(&rise), "{rate} Hz: rise {rise} s");
        assert!((0.010..=0.015).contains(&over), "{rate} Hz: overswing {over}");
    }
}

#[test]
fn vu_calibration_holds_at_100_1k_10k_and_three_rates() {
    // A sine at the reference peak reads 0 VU within 0.2 dB (rate high enough to sample the rectifier).
    for rate in [48_000.0f32, 96_000.0, 192_000.0] {
        for hz in [100.0f32, 1_000.0, 10_000.0] {
            let mut m = meter(rate, 0);
            m.reference_db = -18.0;
            feed(&mut m, (3.0 * rate) as usize, 512, sine(rate, hz, 10f32.powf(-18.0 / 20.0)));
            let r = m.reading(0);
            assert!(r.abs() < 0.2, "VU {hz} Hz @ {rate}: {r}");
        }
    }
}

#[test]
fn vu_falls_from_reference_and_from_loud() {
    // After the signal stops the reading falls monotonically through -20 dB without undershooting zero.
    for loud_db in [0.0f32, 12.0] {
        let mut m = meter(48_000.0, 0);
        let amp = 10f32.powf(loud_db / 20.0) / SINE_TO_RMS;
        feed(&mut m, 48_000, 256, |_| amp);
        let mut prev = m.reading(0);
        let mut t20 = None;
        for i in 0..96_000 / 256 {
            feed(&mut m, 256, 256, |_| 0.0);
            let r = m.reading(0);
            assert!(r.is_finite());
            if t20.is_none() && r <= loud_db - 20.0 { t20 = Some(i * 256); }
            assert!(r <= prev + 0.5, "VU rose while falling: {prev} -> {r}");
            prev = r;
        }
        assert!(t20.is_some(), "VU never fell 20 dB from {loud_db}");
    }
}

#[test]
fn ppm_fall_times_match_itu_table_1_at_two_rates() {
    for rate in [48_000.0f32, 96_000.0] {
        for (kind, fall_db, fall_s) in [(5, 20.0f32, 1.7f32), (6, 20.0, 1.7), (7, 26.0, 3.0), (8, 24.0, 2.8)] {
            let mut m = meter(rate, kind);
            feed(&mut m, rate as usize, 480, sine(rate, 1000.0, 10f32.powf(9.0 / 20.0)));
            let start = m.reading(0);
            let mut t = None;
            let mut frame = 0usize;
            while frame < (10.0 * rate) as usize {
                feed(&mut m, 480, 480, |_| 0.0);
                frame += 480;
                if t.is_none() && m.reading(0) <= start - fall_db { t = Some(frame as f32 / rate); }
            }
            let t = t.expect("never fell");
            assert!((t - fall_s).abs() <= 0.1, "kind {kind} @ {rate}: {fall_db} dB in {t} s, want {fall_s}");
        }
    }
}

#[test]
fn ppm_reads_reference_for_a_steady_sine() {
    // The peak-programme meters read the sine's peak: reference peak -> 0, within 0.5 dB at 1 kHz.
    for kind in [5, 6, 7, 8] {
        let mut m = meter(48_000.0, kind);
        m.reference_db = -18.0;
        feed(&mut m, 3 * 48_000, 480, sine(48_000.0, 1000.0, 10f32.powf(-18.0 / 20.0)));
        let r = m.reading(0);
        assert!(r.abs() < 0.5, "kind {kind}: {r}");
    }
}

#[test]
fn ebu_integration_time_is_10_ms() {
    // Tech 3205 clause 3.3: a 5 kHz burst reads 2 dB below the steady tone at 10 +/-2 ms.
    let rate = 96_000.0f32;
    let amp = 10f32.powf(9.0 / 20.0);
    let mut s = meter(rate, 8);
    feed(&mut s, rate as usize, 480, sine(rate, 5000.0, amp));
    let steady = s.reading(0);
    let mut found = None;
    for step in 1..=150 {
        let dur = step as f32 * 1.0e-4;
        let burst = (dur * rate) as usize;
        let mut m = meter(rate, 8);
        let mut f = sine(rate, 5000.0, amp);
        let mut peak = f32::MIN;
        let mut frame = 0usize;
        while frame < burst + (0.1 * rate) as usize {
            feed(&mut m, 48, 48, |_| { let v = if frame < burst { f(frame) } else { 0.0 }; frame += 1; v });
            peak = peak.max(m.reading(0));
        }
        if peak >= steady - 2.0 { found = Some(dur * 1000.0); break; }
    }
    let ms = found.expect("never reached -2 dB");
    assert!((8.0..=12.0).contains(&ms), "integration time {ms} ms");
}

#[test]
fn readings_do_not_depend_on_sample_rate_or_block_size() {
    // Same -14 dBFS 1 kHz sine, 0.4 + 3 s, two rates and three block sizes.
    for kind in [0, 1, 5, 7, 8, 9] {
        let mut readings = Vec::new();
        for rate in [44_100.0f32, 48_000.0, 96_000.0] {
            for block in [1usize, 64, 4096] {
                let mut m = meter(rate, kind);
                m.reference_db = -18.0;
                feed(&mut m, (4.0 * rate) as usize, block, sine(rate, 1000.0, 10f32.powf(-14.0 / 20.0)));
                readings.push((rate, block, m.reading(0)));
            }
        }
        let first = readings[0].2;
        for (rate, block, r) in &readings {
            assert!((r - first).abs() < 0.35, "kind {kind}: {rate} Hz block {block} read {r}, first {first}");
        }
    }
}

#[test]
fn correlation_does_not_depend_on_sample_rate() {
    // Two noise-free partially correlated tones: the measured coefficient must match across rates.
    let mut vals = Vec::new();
    for rate in [44_100.0f32, 48_000.0, 96_000.0, 192_000.0] {
        let mut m = meter(rate, 0);
        let total = (2.0 * rate) as usize;
        let mut buf = Vec::with_capacity(total * 2);
        for i in 0..total {
            let t = i as f32 / rate;
            let a = 0.3 * (TAU * 220.0 * t).sin();
            let b = 0.3 * (TAU * 220.0 * t + 1.2).sin(); // 1.2 rad apart -> coefficient cos(1.2)
            buf.push(a);
            buf.push(b);
        }
        m.process_interleaved(&buf, 2);
        vals.push((rate, m.correlation));
    }
    let first = vals[0].1;
    for (rate, v) in &vals {
        assert!((v - first).abs() < 1.5, "correlation at {rate} Hz = {v}, at 44.1k = {first}");
    }
}

#[test]
fn correlation_reads_out_of_phase_and_mono() {
    let rate = 48_000.0;
    let mut m = meter(rate, 0);
    let mut buf = Vec::new();
    for i in 0..48_000 {
        let v = 0.3 * (TAU * 440.0 * i as f32 / rate).sin();
        buf.push(v);
        buf.push(-v);
    }
    m.process_interleaved(&buf, 2);
    assert!(m.correlation < -35.0, "inverted reads {}", m.correlation);
    let mut m = meter(rate, 0);
    let mut buf = Vec::new();
    for i in 0..48_000 {
        let v = 0.3 * (TAU * 440.0 * i as f32 / rate).sin();
        buf.push(v);
        buf.push(v);
    }
    m.process_interleaved(&buf, 2);
    assert!(m.correlation > -0.5, "mono reads {}", m.correlation);
}

#[test]
fn silence_reads_the_floor_and_is_never_nan() {
    for kind in 0..=9 {
        let mut m = meter(48_000.0, kind);
        feed(&mut m, 48_000, 256, |_| 0.0);
        for ch in 0..2 {
            let r = m.reading(ch);
            assert!(r.is_finite(), "kind {kind} ch {ch} reading {r}");
            assert!(m.peak_db(ch).is_finite(), "kind {kind} ch {ch} peak");
        }
        assert!(m.radar_reading().is_finite());
        assert!(m.correlation.is_finite());
    }
}

#[test]
fn over_full_scale_counts_overs_and_does_not_wrap() {
    let mut m = meter(48_000.0, 0);
    feed(&mut m, 4800, 256, |i| if (i / 8) % 2 == 0 { 1.8 } else { -1.8 });
    assert!(m.overs > 0, "overs {}", m.overs);
    assert!(m.any_clip());
    assert!(m.reading(0).is_finite() && m.peak_db(0) > 0.0);
}

#[test]
fn peak_marker_holds_then_falls() {
    let mut m = meter(48_000.0, 0);
    feed(&mut m, 4800, 256, |i| if i == 100 { 0.8 } else { 0.0 });
    let held = m.channels[0].peak;
    feed(&mut m, 2 * 48_000, 256, |_| 0.0);
    assert!((m.channels[0].peak - held).abs() < 1.0e-6, "peak moved during the 3 s hold");
    feed(&mut m, 4 * 48_000, 256, |_| 0.0);
    assert!(m.channels[0].peak < held * 0.5, "peak did not release after the hold");
    // Numeric peak holds for VALUE_HOLD_S (5 s), then follows.
    let mut m = meter(48_000.0, 0);
    feed(&mut m, 4800, 256, |i| if i == 100 { 0.8 } else { 0.0 });
    feed(&mut m, 4 * 48_000, 256, |_| 0.0);
    assert!(m.channels[0].numeric_peak > 0.79);
    feed(&mut m, 2 * 48_000, 256, |_| 0.0);
    assert!(m.channels[0].numeric_peak < 0.1);
}

#[test]
fn radar_follows_the_louder_channel_in_either_order() {
    let rate = 48_000.0f32;
    for swap in [false, true] {
        let mut m = meter(rate, 1);
        let total = 2 * 48_000;
        let mut buf = Vec::new();
        for i in 0..total {
            let loud = 0.5 * (TAU * 1000.0 * i as f32 / rate).sin();
            let quiet = loud * 0.05;
            if swap { buf.push(quiet); buf.push(loud); } else { buf.push(loud); buf.push(quiet); }
        }
        m.process_interleaved(&buf, 2);
        let louder = m.reading(0).max(m.reading(1));
        assert!((m.radar_reading() - louder).abs() < 1.0e-4);
        assert!(louder - m.reading(0).min(m.reading(1)) > 20.0, "channels not independent");
    }
}

#[test]
fn mono_input_reads_the_same_as_dual_mono() {
    let rate = 48_000.0f32;
    let mut a = meter(rate, 1);
    let mut b = meter(rate, 1);
    let total = 2 * 48_000;
    let mono: Vec<f32> = (0..total).map(|i| 0.3 * (TAU * 1000.0 * i as f32 / rate).sin()).collect();
    let dual: Vec<f32> = mono.iter().flat_map(|v| [*v, *v]).collect();
    a.process_interleaved(&mono, 1);
    b.process_interleaved(&dual, 2);
    assert!((a.reading(0) - b.reading(0)).abs() < 0.05, "{} vs {}", a.reading(0), b.reading(0));
    assert!((a.loud.readings().momentary - b.loud.readings().momentary).abs() < 3.1, "loudness mono vs dual");
}

#[test]
fn meter_type_change_does_not_leave_stale_peak_programme_state() {
    // Run EBU, switch to DIN: the DIN reading must reflect the DIN detector, not the old EBU stage.
    let rate = 48_000.0f32;
    let mut m = meter(rate, 8);
    feed(&mut m, 48_000, 256, sine(rate, 1000.0, 1.0));
    m.kind = 5;
    feed(&mut m, 10 * 48_000, 256, |_| 0.0);
    assert!(m.reading(0) < -60.0, "DIN after 10 s silence reads {}", m.reading(0));
}

#[test]
fn correlation_time_constant_is_the_same_at_every_rate() {
    // Mono for 1 s, then inverted: the time for the coefficient to cross 0 is a fixed fraction of the window.
    let mut crossings = Vec::new();
    for rate in [44_100.0f32, 48_000.0, 96_000.0, 192_000.0] {
        let mut m = meter(rate, 0);
        feed(&mut m, rate as usize, 256, sine(rate, 440.0, 0.3));
        let mut t = None;
        let mut frame = 0usize;
        while frame < (0.5 * rate) as usize && t.is_none() {
            let start = frame;
            let mut buf = Vec::new();
            for i in 0..16 {
                let v = 0.3 * (TAU * 440.0 * (start + i) as f32 / rate).sin();
                buf.push(v);
                buf.push(-v);
            }
            m.process_interleaved(&buf, 2);
            frame += 16;
            if m.correlation <= -20.0 { t = Some(frame as f32 / rate); }
        }
        crossings.push((rate, t.expect("never crossed zero")));
    }
    let first = crossings[0].1;
    for (rate, t) in &crossings {
        assert!((t - first).abs() < 0.004, "{rate} Hz crossed after {t} s, 44.1k after {first} s");
    }
}
