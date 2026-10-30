//! Loudness: ITU-R BS.1770-5 K-weighting, gating and true peak, with the EBU Tech 3341 time scales
//! (momentary 0.4 s, short-term 3 s) and the EBU Tech 3342 loudness range.
//!
//! Energy is kept in 1 ms sub-blocks, so every window is a sliding rectangular window and the
//! 400 ms gating blocks (75 % overlap, one every 100 ms) are exact at any sample rate.
//! A mono input is one channel with weight 1, as BS.1770 defines it. Stereo is L + R, both weight 1.
//!
//! This engine is the verified implementation shared with ArcVu; the EBU 3341/3342 calibration
//! tests live at the bottom and must not be weakened.

pub const FLOOR: f32 = -200.0;
const RING: usize = 30_008;
const BINS: usize = 10_000; // 0.01 LU steps from -70 to +30 LUFS
const BIN_LU: f64 = 0.01;
const ABS_GATE: f64 = -70.0;
const OFFSET: f64 = -0.691;

#[derive(Clone, Copy, Default)]
struct Biquad { b0: f64, b1: f64, b2: f64, a1: f64, a2: f64, z1: f64, z2: f64 }

impl Biquad {
    fn run(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Stage 1 (head shelf) and stage 2 (RLB high-pass). At 48 kHz these are the BS.1770 Table 1 and 2
/// values; other rates keep the same frequency response through the analogue prototype.
pub fn k_weighting(sr: f64) -> ([f64; 5], [f64; 5]) {
    if (sr - 48_000.0).abs() < 0.5 {
        return ([1.53512485958697, -2.69169618940638, 1.19839281085285, -1.69065929318241, 0.73248077421585],
                [1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621]);
    }
    let (f0, g, q) = (1681.974450955533_f64, 3.999843853973347_f64, 0.7071752369554196_f64);
    let k = (std::f64::consts::PI * f0 / sr).tan();
    let vh = 10f64.powf(g / 20.0);
    let vb = vh.powf(0.4996667741545416);
    let a0 = 1.0 + k / q + k * k;
    let s1 = [(vh + vb * k / q + k * k) / a0, 2.0 * (k * k - vh) / a0, (vh - vb * k / q + k * k) / a0,
              2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0];
    let (f0, q) = (38.13547087602444_f64, 0.5003270373238773_f64);
    let k = (std::f64::consts::PI * f0 / sr).tan();
    let a0 = 1.0 + k / q + k * k;
    let s2 = [1.0, -2.0, 1.0, 2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0];
    (s1, s2)
}

fn biquad(c: [f64; 5]) -> Biquad { Biquad { b0: c[0], b1: c[1], b2: c[2], a1: c[3], a2: c[4], z1: 0.0, z2: 0.0 } }

/// BS.1770-5 Annex 2, 4x interpolating FIR, 12 taps per phase (rows are taps, columns are phases).
const TP_FIR: [[f64; 4]; 12] = [
    [0.0017089843750, -0.0291748046875, -0.0189208984375, -0.0083007812500],
    [0.0109863281250, 0.0292968750000, 0.0330810546875, 0.0148925781250],
    [-0.0196533203125, -0.0517578125000, -0.0582275390625, -0.0266113281250],
    [0.0332031250000, 0.0891113281250, 0.1015625000000, 0.0476074218750],
    [-0.0594482421875, -0.1665039062500, -0.2003173828125, -0.1022949218750],
    [0.1373291015625, 0.4650878906250, 0.7797851562500, 0.9721679687500],
    [0.9721679687500, 0.7797851562500, 0.4650878906250, 0.1373291015625],
    [-0.1022949218750, -0.2003173828125, -0.1665039062500, -0.0594482421875],
    [0.0476074218750, 0.1015625000000, 0.0891113281250, 0.0332031250000],
    [-0.0266113281250, -0.0582275390625, -0.0517578125000, -0.0196533203125],
    [0.0148925781250, 0.0330810546875, 0.0292968750000, 0.0109863281250],
    [-0.0083007812500, -0.0189208984375, -0.0291748046875, 0.0017089843750],
];

/// Counts and energies of block loudness values in 0.01 LU bins, for the gated statistics.
struct Histogram { count: Vec<u32>, energy: Vec<f64> }

impl Histogram {
    fn new() -> Self { Self { count: vec![0; BINS], energy: vec![0.0; BINS] } }
    fn clear(&mut self) { self.count.iter_mut().for_each(|c| *c = 0); self.energy.iter_mut().for_each(|e| *e = 0.0); }
    fn add(&mut self, loudness: f64, z: f64) {
        if loudness <= ABS_GATE { return; }
        let i = (((loudness - ABS_GATE) / BIN_LU) as usize).min(BINS - 1);
        self.count[i] += 1;
        self.energy[i] += z;
    }
    /// First bin above the threshold that sits `rel` LU below the absolute-gated mean, or None when empty.
    fn relative_start(&self, rel: f64) -> Option<usize> {
        let (n, e) = self.count.iter().zip(&self.energy).fold((0u64, 0.0f64), |a, (c, e)| (a.0 + *c as u64, a.1 + e));
        if n == 0 { return None; }
        let threshold = OFFSET + 10.0 * (e / n as f64).log10() + rel;
        Some((((threshold - ABS_GATE) / BIN_LU).floor() + 1.0).clamp(0.0, BINS as f64) as usize)
    }
    fn integrated(&self) -> Option<f64> {
        let start = self.relative_start(-10.0)?;
        let (n, e) = (start..BINS).fold((0u64, 0.0f64), |a, i| (a.0 + self.count[i] as u64, a.1 + self.energy[i]));
        if n == 0 { return None; }
        Some(OFFSET + 10.0 * (e / n as f64).log10())
    }
    fn range(&self) -> Option<f64> {
        let start = self.relative_start(-20.0)?;
        let n: u64 = self.count[start..].iter().map(|c| *c as u64).sum();
        if n == 0 { return None; }
        let pick = |percent: f64| -> f64 {
            // Tech 3342 reference: sorted(round((n - 1) * p / 100 + 1)), one-based.
            let rank = (((n - 1) as f64) * percent / 100.0 + 1.0).round() as u64;
            let mut seen = 0u64;
            for i in start..BINS {
                seen += self.count[i] as u64;
                if seen >= rank { return ABS_GATE + (i as f64 + 0.5) * BIN_LU; }
            }
            ABS_GATE + (BINS as f64) * BIN_LU
        };
        Some(pick(95.0) - pick(10.0))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Readings {
    pub momentary: f32,
    pub short_term: f32,
    pub sliding: f32,
    pub integrated: f32,
    pub range: f32,
    pub true_peak: f32,
    pub max_momentary: f32,
    pub max_short_term: f32,
    /// 1 once 60 s have been measured since the reset (the range reading is not stable before that).
    pub range_stable: i32,
    /// Seconds measured since the reset.
    pub elapsed: f32,
    /// Highest absolute sample value since the reset, in dBFS.
    pub sample_peak: f32,
}

pub struct Loudness {
    sr: f64,
    stage1: [Biquad; 2],
    stage2: [Biquad; 2],
    tp_hist: [[f64; 24]; 2],
    tp_pos: usize,
    tp_max: f64,
    sample_max: f64,
    ring_e: Vec<f64>,
    ring_n: Vec<u32>,
    n_sub: u64,
    sub_energy: f64,
    sub_start: u64,
    samples: u64,
    edge: f64,
    win: [usize; 3],
    win_e: [f64; 3],
    win_n: [u64; 3],
    integrated_hist: Histogram,
    range_hist: Histogram,
    integrated: f64,
    range: f64,
    max_m: f64,
    max_s: f64,
}

fn db(z: f64) -> f64 { if z < 1.0e-14 { FLOOR as f64 } else { OFFSET + 10.0 * z.log10() } }

impl Loudness {
    pub fn new(sample_rate: f64) -> Self {
        let mut l = Self {
            sr: 48_000.0, stage1: [Biquad::default(); 2], stage2: [Biquad::default(); 2],
            tp_hist: [[0.0; 24]; 2], tp_pos: 0, tp_max: 0.0, sample_max: 0.0,
            ring_e: vec![0.0; RING], ring_n: vec![0; RING], n_sub: 0, sub_energy: 0.0, sub_start: 0,
            samples: 0, edge: 48.0, win: [400, 3000, 10_000], win_e: [0.0; 3], win_n: [0; 3],
            integrated_hist: Histogram::new(), range_hist: Histogram::new(),
            integrated: FLOOR as f64, range: 0.0, max_m: FLOOR as f64, max_s: FLOOR as f64,
        };
        l.set_sample_rate(sample_rate);
        l
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate.clamp(8_000.0, 768_000.0);
        let (s1, s2) = k_weighting(self.sr);
        self.stage1 = [biquad(s1); 2];
        self.stage2 = [biquad(s2); 2];
        self.reset();
    }

    pub fn sample_rate(&self) -> f64 { self.sr }

    /// Length of the extra sliding window, 1 to 30 s.
    pub fn set_window(&mut self, seconds: f32) {
        let w = ((seconds.clamp(1.0, 30.0) * 1000.0).round() as usize).min(RING - 8);
        if w == self.win[2] { return; }
        self.win[2] = w;
        let (mut e, mut n) = (0.0, 0u64);
        let have = (self.n_sub as usize).min(w);
        for k in 0..have {
            let i = (self.n_sub as usize - 1 - k) % RING;
            e += self.ring_e[i];
            n += self.ring_n[i] as u64;
        }
        self.win_e[2] = e;
        self.win_n[2] = n;
    }

    pub fn reset(&mut self) {
        for c in 0..2 {
            self.stage1[c].z1 = 0.0; self.stage1[c].z2 = 0.0;
            self.stage2[c].z1 = 0.0; self.stage2[c].z2 = 0.0;
        }
        self.tp_hist = [[0.0; 24]; 2];
        self.tp_max = 0.0;
        self.sample_max = 0.0;
        self.ring_e.iter_mut().for_each(|v| *v = 0.0);
        self.ring_n.iter_mut().for_each(|v| *v = 0);
        self.n_sub = 0; self.sub_energy = 0.0; self.sub_start = 0; self.samples = 0;
        self.edge = self.sr / 1000.0;
        self.win_e = [0.0; 3];
        self.win_n = [0; 3];
        self.integrated_hist.clear();
        self.range_hist.clear();
        self.integrated = FLOOR as f64;
        self.range = 0.0;
        self.max_m = FLOOR as f64;
        self.max_s = FLOOR as f64;
    }

    /// One frame. With `stereo` false only `l` is measured (a mono input, weight 1).
    pub fn process(&mut self, l: f32, r: f32, stereo: bool) {
        let channels = if stereo { 2 } else { 1 };
        let input = [l as f64, r as f64];
        let mut energy = 0.0;
        for c in 0..channels {
            let y = self.stage2[c].run(self.stage1[c].run(input[c]));
            energy += y * y;
            self.sample_max = self.sample_max.max(input[c].abs());
            // True peak: the four interpolated phases of the unweighted signal.
            let h = &mut self.tp_hist[c];
            h[self.tp_pos] = input[c];
            h[self.tp_pos + 12] = input[c];
            let taps = &h[self.tp_pos + 1..self.tp_pos + 13];
            for p in 0..4 {
                let mut acc = 0.0;
                for k in 0..12 { acc += TP_FIR[k][p] * taps[k]; }
                self.tp_max = self.tp_max.max(acc.abs());
            }
        }
        self.tp_pos = (self.tp_pos + 1) % 12;
        self.sub_energy += energy;
        self.samples += 1;
        if self.samples as f64 >= self.edge - 1.0e-9 {
            let n = (self.samples - self.sub_start) as u32;
            self.sub_start = self.samples;
            let e = std::mem::take(&mut self.sub_energy);
            self.push(e, n);
            self.edge = (self.n_sub + 1) as f64 * self.sr / 1000.0;
        }
    }

    pub fn process_interleaved(&mut self, samples: &[f32], channels: usize) {
        if channels == 0 { return; }
        for frame in samples.chunks_exact(channels) {
            self.process(frame[0], if channels > 1 { frame[1] } else { 0.0 }, channels > 1);
        }
    }

    fn push(&mut self, e: f64, n: u32) {
        let index = self.n_sub as usize % RING;
        self.ring_e[index] = e;
        self.ring_n[index] = n;
        for w in 0..3 {
            self.win_e[w] += e;
            self.win_n[w] += n as u64;
            if self.n_sub as usize >= self.win[w] {
                let old = (self.n_sub as usize - self.win[w]) % RING;
                self.win_e[w] = (self.win_e[w] - self.ring_e[old]).max(0.0);
                self.win_n[w] -= self.ring_n[old] as u64;
            }
        }
        self.n_sub += 1;
        if self.n_sub >= 400 { self.max_m = self.max_m.max(self.window_db(0)); }
        if self.n_sub >= 3000 { self.max_s = self.max_s.max(self.window_db(1)); }
        if self.n_sub % 100 == 0 {
            if self.n_sub >= 400 {
                let z = self.win_e[0] / self.win_n[0].max(1) as f64;
                self.integrated_hist.add(db(z), z);
                self.integrated = self.integrated_hist.integrated().unwrap_or(FLOOR as f64);
            }
            if self.n_sub >= 3000 {
                let z = self.win_e[1] / self.win_n[1].max(1) as f64;
                self.range_hist.add(db(z), z);
                self.range = self.range_hist.range().unwrap_or(0.0);
            }
        }
    }

    fn window_db(&self, w: usize) -> f64 {
        db(self.win_e[w] / self.win_n[w].max(1) as f64)
    }

    pub fn readings(&self) -> Readings {
        let heard = self.n_sub > 0;
        let tp = if self.tp_max > 1.0e-9 { 20.0 * self.tp_max.log10() } else { FLOOR as f64 };
        Readings {
            momentary: if heard { self.window_db(0) as f32 } else { FLOOR },
            short_term: if heard { self.window_db(1) as f32 } else { FLOOR },
            sliding: if heard { self.window_db(2) as f32 } else { FLOOR },
            integrated: self.integrated as f32,
            range: self.range as f32,
            true_peak: tp as f32,
            max_momentary: self.max_m as f32,
            max_short_term: self.max_s as f32,
            range_stable: (self.n_sub >= 60_000) as i32,
            elapsed: (self.n_sub as f32) / 1000.0,
            sample_peak: if self.sample_max > 1.0e-9 { (20.0 * self.sample_max.log10()) as f32 } else { FLOOR },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f64 = 48_000.0;

    fn tone(l: &mut Loudness, seconds: f64, dbfs: f64) {
        tone_at(l, seconds, dbfs, 1000.0, 0.0);
    }

    fn tone_at(l: &mut Loudness, seconds: f64, dbfs: f64, hz: f64, phase_deg: f64) {
        let amp = 10f64.powf(dbfs / 20.0);
        let n = (seconds * l.sample_rate()).round() as usize;
        let p = phase_deg.to_radians();
        for i in 0..n {
            let v = (amp * (std::f64::consts::TAU * hz * i as f64 / l.sample_rate() + p).sin()) as f32;
            l.process(v, v, true);
        }
    }

    fn silence(l: &mut Loudness, seconds: f64) {
        for _ in 0..(seconds * l.sample_rate()).round() as usize { l.process(0.0, 0.0, true); }
    }

    fn near(value: f32, expect: f64, tol: f64, what: &str) {
        assert!((value as f64 - expect).abs() <= tol, "{what}: {value} vs {expect} +/- {tol}");
    }

    #[test]
    fn k_weighting_at_other_rates_matches_the_48k_table_where_both_exist() {
        let (f0, g, q) = (1681.974450955533_f64, 3.999843853973347_f64, 0.7071752369554196_f64);
        let k = (std::f64::consts::PI * f0 / SR).tan();
        let vh = 10f64.powf(g / 20.0);
        let vb = vh.powf(0.4996667741545416);
        let a0 = 1.0 + k / q + k * k;
        let b0 = (vh + vb * k / q + k * k) / a0;
        let a1 = 2.0 * (k * k - 1.0) / a0;
        let (table1, _) = k_weighting(SR);
        assert!((b0 - table1[0]).abs() < 1.0e-6 && (a1 - table1[3]).abs() < 1.0e-6);
    }

    #[test]
    fn ebu_3341_case_1_to_2_calibration_tone() {
        let mut l = Loudness::new(SR);
        tone(&mut l, 20.0, -23.0);
        let r = l.readings();
        near(r.momentary, -23.0, 0.1, "M"); near(r.short_term, -23.0, 0.1, "S"); near(r.integrated, -23.0, 0.1, "I");
        let mut l = Loudness::new(SR);
        tone(&mut l, 20.0, -33.0);
        near(l.readings().integrated, -33.0, 0.1, "I at -33");
    }

    #[test]
    fn ebu_3341_case_3_and_4_the_relative_gate() {
        let mut l = Loudness::new(SR);
        tone(&mut l, 10.0, -36.0); tone(&mut l, 60.0, -23.0); tone(&mut l, 10.0, -36.0);
        near(l.readings().integrated, -23.0, 0.1, "case 3");
        let mut l = Loudness::new(SR);
        tone(&mut l, 20.0, -26.0); tone(&mut l, 20.1, -20.0); tone(&mut l, 20.0, -26.0);
        near(l.readings().integrated, -23.0, 0.1, "case 4");
    }

    #[test]
    fn ebu_3341_case_9_short_term_of_alternating_tones() {
        let mut l = Loudness::new(SR);
        for _ in 0..5 { tone(&mut l, 1.34, -20.0); tone(&mut l, 1.66, -30.0); }
        near(l.readings().short_term, -23.0, 0.1, "S");
    }

    #[test]
    fn ebu_3341_case_12_momentary_of_alternating_tones() {
        let mut l = Loudness::new(SR);
        for _ in 0..25 { tone(&mut l, 0.18, -20.0); tone(&mut l, 0.22, -30.0); }
        near(l.readings().momentary, -23.0, 0.1, "M");
    }

    #[test]
    fn ebu_3341_case_10_and_13_the_maximum_over_bursts() {
        for i in 0..20 {
            let mut l = Loudness::new(SR);
            silence(&mut l, i as f64 * 0.02);
            tone(&mut l, 0.4, -23.0);
            silence(&mut l, 1.0);
            near(l.readings().max_momentary, -23.0, 0.1, "max M");
        }
        for i in [0usize, 7, 19] {
            let mut l = Loudness::new(SR);
            silence(&mut l, i as f64 * 0.15);
            tone(&mut l, 3.0, -23.0);
            silence(&mut l, 1.0);
            near(l.readings().max_short_term, -23.0, 0.1, "max S");
        }
    }

    #[test]
    fn ebu_3342_loudness_range_cases_1_to_4() {
        let run = |levels: &[f64]| -> f32 {
            let mut l = Loudness::new(SR);
            for lv in levels { tone(&mut l, 20.0, *lv); }
            l.readings().range
        };
        near(run(&[-20.0, -30.0]), 10.0, 1.0, "case 1");
        near(run(&[-20.0, -15.0]), 5.0, 1.0, "case 2");
        near(run(&[-40.0, -20.0]), 20.0, 1.0, "case 3");
        near(run(&[-50.0, -35.0, -20.0, -35.0, -50.0]), 15.0, 1.0, "case 4");
    }

    #[test]
    fn true_peak_matches_ebu_3341_cases_15_to_19() {
        let tp = |hz_div: f64, amp_db: f64, phase: f64| -> f32 {
            let mut l = Loudness::new(SR);
            let amp = 10f64.powf(amp_db / 20.0);
            let n = (0.2 * SR) as usize;
            let fade = (0.01 * SR) as usize;
            for i in 0..n {
                let edge = i.min(n - 1 - i);
                let w = if edge < fade { 0.5 - 0.5 * (std::f64::consts::PI * edge as f64 / fade as f64).cos() } else { 1.0 };
                let v = (w * amp * (std::f64::consts::TAU * i as f64 / hz_div + phase.to_radians()).sin()) as f32;
                l.process(v, v, true);
            }
            l.readings().true_peak
        };
        let within = |v: f32, expect: f64, what: &str| assert!(v as f64 >= expect - 0.4 && v as f64 <= expect + 0.2, "{what}: {v} vs {expect}");
        within(tp(4.0, -6.0206, 0.0), -6.0, "case 15");
        within(tp(4.0, -6.0206, 45.0), -6.0, "case 16");
        within(tp(6.0, -6.0206, 60.0), -6.0, "case 17");
        within(tp(8.0, -6.0206, 67.5), -6.0, "case 18");
        within(tp(4.0, 20.0 * 1.41f64.log10(), 45.0), 3.0, "case 19");
    }

    #[test]
    fn the_sliding_window_follows_its_length() {
        let mut l = Loudness::new(SR);
        l.set_window(10.0);
        tone(&mut l, 10.0, -30.0); tone(&mut l, 5.0, -20.0);
        let r = l.readings();
        let expect = 10.0 * ((10f64.powf(-2.0) + 10f64.powf(-3.0)) / 2.0).log10();
        near(r.sliding, expect, 0.15, "10 s window");
        l.set_window(2.0);
        near(l.readings().sliding, -20.0, 0.1, "2 s window");
    }

    #[test]
    fn other_sample_rates_read_the_same_level() {
        for sr in [44_100.0, 88_200.0, 96_000.0] {
            let mut l = Loudness::new(sr);
            tone(&mut l, 12.0, -23.0);
            near(l.readings().integrated, -23.0, 0.15, &format!("I at {sr}"));
        }
    }

    #[test]
    fn mono_is_a_single_channel_and_reset_clears() {
        let mut l = Loudness::new(SR);
        let amp = 10f32.powf(-23.0 / 20.0);
        for i in 0..(SR as usize * 6) { l.process(amp * (std::f32::consts::TAU * 1000.0 * i as f32 / SR as f32).sin(), 0.0, false); }
        near(l.readings().short_term, -26.0, 0.15, "mono reads 3 dB under dual mono");
        l.reset();
        let r = l.readings();
        assert!(r.integrated <= FLOOR && r.true_peak <= FLOOR && r.elapsed == 0.0);
    }
}
