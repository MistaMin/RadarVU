//! Six code-drawn skins. Colours are roles, so light and dark screens keep readable labels,
//! visible controls and consistent warning colours without changing any measurements.
use goodlookindri::color::Color;

pub const NAMES: [&str; 6] = [
    "STOCK DARK", "RETRO CONSOLE", "IVORY TERMINAL", "MATRIX", "MODERN CONSOLE", "STUDIO AMBER",
];

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg_top: Color, pub bg_bottom: Color, pub header: Color,
    pub panel: Color, pub panel_edge: Color, pub cell: Color, pub cell_bottom: Color,
    pub screen: Color, pub bezel: Color, pub grid: Color, pub tick: Color,
    pub label: Color, pub bright: Color, pub white: Color, pub dim: Color,
    pub red: Color, pub amber: Color, pub green: Color, pub blue: Color, pub phosphor: Color,
    pub button: Color, pub button_bottom: Color, pub active: Color,
    pub active_bottom: Color, pub active_text: Color,
    pub heat: [Color; 6],
    pub panel_radius: f32, pub button_radius: f32,
    pub mono: bool, pub scanlines: bool, pub hardware: bool,
}

pub fn palette(index: i32) -> Palette {
    let dark = Palette {
        bg_top: Color(0xff020406), bg_bottom: Color(0xff080d12), header: Color(0xff030608),
        panel: Color(0xff03080d), panel_edge: Color(0xff3c5362),
        cell: Color(0xff080f16), cell_bottom: Color(0xff050a10),
        screen: Color(0xff010508), bezel: Color(0xff15232e),
        grid: Color(0xff253743), tick: Color(0xff67818f),
        label: Color(0xffa0b5c2), bright: Color(0xffe4edf3), white: Color(0xfff2f7fb),
        dim: Color(0xff91a6b4), red: Color(0xffff6152), amber: Color(0xffffcc53),
        green: Color(0xff6ce48f), blue: Color(0xff74c9f1), phosphor: Color(0xff4be4ff),
        button: Color(0xff182832), button_bottom: Color(0xff0b151d),
        active: Color(0xff63e6ff), active_bottom: Color(0xff2087a7), active_text: Color(0xff041016),
        heat: [Color(0xff2563eb), Color(0xff06b6d4), Color(0xff22c55e), Color(0xffeab308), Color(0xfff97316), Color(0xffef4444)],
        panel_radius: 12.0, button_radius: 13.0, mono: false, scanlines: false, hardware: false,
    };
    match index {
        1 => Palette {
            bg_top: Color(0xff44483d), bg_bottom: Color(0xff232b25), header: Color(0xff1d2922),
            panel: Color(0xff171e17), panel_edge: Color(0xff7d886b),
            cell: Color(0xff111b13), cell_bottom: Color(0xff0c160f),
            screen: Color(0xff08140d), bezel: Color(0xff69705b),
            grid: Color(0xff2e4428), tick: Color(0xff8a9b68),
            label: Color(0xffc2cda6), bright: Color(0xffe5e7bc), white: Color(0xfffff1bf),
            dim: Color(0xffa5b28e), phosphor: Color(0xffffd77c), blue: Color(0xffc6dfa2),
            button: Color(0xff45533b), button_bottom: Color(0xff293a2d),
            active: Color(0xffefda9d), active_bottom: Color(0xffb39b60), active_text: Color(0xff18251b),
            heat: [Color(0xff456d43), Color(0xff75944d), Color(0xffa4b15c), Color(0xffd4c878), Color(0xffe9af58), Color(0xffff7854)],
            panel_radius: 3.0, button_radius: 3.0, mono: true, scanlines: true, hardware: true,
            ..dark
        },
        2 => Palette {
            bg_top: Color(0xfff3f1e9), bg_bottom: Color(0xffd3dbd5), header: Color(0xffe0e6de),
            panel: Color(0xffc4d0c7), panel_edge: Color(0xff6b8178),
            cell: Color(0xfff9fcf6), cell_bottom: Color(0xffedf3e9),
            screen: Color(0xfff4f8ef), bezel: Color(0xffb0bfb3),
            grid: Color(0xffc5d2c6), tick: Color(0xff6d8a78),
            label: Color(0xff374e42), bright: Color(0xff152e25), white: Color(0xff17382b),
            dim: Color(0xff4a6256), red: Color(0xffb12b2b), amber: Color(0xff886000),
            green: Color(0xff146447), blue: Color(0xff235e83), phosphor: Color(0xff14695e),
            button: Color(0xfff9fcf5), button_bottom: Color(0xffd5e0d3),
            active: Color(0xff287c69), active_bottom: Color(0xff145a4c), active_text: Color(0xfff9fff8),
            heat: [Color(0xff3b6f8e), Color(0xff267b7f), Color(0xff528a51), Color(0xff8d842e), Color(0xffb17033), Color(0xffb84439)],
            panel_radius: 3.0, button_radius: 3.0, mono: true, scanlines: true, hardware: true,
            ..dark
        },
        3 => Palette {
            bg_top: Color(0xff010603), bg_bottom: Color(0xff031109), header: Color(0xff010a04),
            panel: Color(0xff020d06), panel_edge: Color(0xff257b42),
            cell: Color(0xff04190b), cell_bottom: Color(0xff020d05),
            screen: Color(0xff000903), bezel: Color(0xff0b2c14),
            grid: Color(0xff113c1e), tick: Color(0xff3a9950),
            label: Color(0xff81c99a), bright: Color(0xffc4ffd5), white: Color(0xffd7ffe0),
            dim: Color(0xff72b989), green: Color(0xff81ff8e), blue: Color(0xff78d7a2), phosphor: Color(0xff63ff87),
            button: Color(0xff10371b), button_bottom: Color(0xff05190b),
            active: Color(0xff8aff9d), active_bottom: Color(0xff37be5a), active_text: Color(0xff001a08),
            heat: [Color(0xff126b39), Color(0xff258c3e), Color(0xff45ad4a), Color(0xff68ce5b), Color(0xff9ee578), Color(0xffd8ffaf)],
            panel_radius: 1.0, button_radius: 1.0, mono: true, scanlines: true,
            ..dark
        },
        4 => Palette {
            bg_top: Color(0xff3b454c), bg_bottom: Color(0xff202a31), header: Color(0xff29343c),
            panel: Color(0xff101b20), panel_edge: Color(0xff687c83),
            cell: Color(0xff101f23), cell_bottom: Color(0xff0b181c),
            screen: Color(0xff061317), bezel: Color(0xff63747a),
            grid: Color(0xff244346), tick: Color(0xff72918c),
            label: Color(0xffa2bab3), bright: Color(0xffe8eee4), white: Color(0xfffcf6da),
            dim: Color(0xff91aaa3), green: Color(0xffb0e5ac), blue: Color(0xffa7d7d3), phosphor: Color(0xffd1e7b8),
            button: Color(0xff283a44), button_bottom: Color(0xff1c2c34),
            active: Color(0xffdce5cb), active_bottom: Color(0xffa1b399), active_text: Color(0xff162b28),
            panel_radius: 5.0, button_radius: 4.0, hardware: true,
            ..dark
        },
        5 => Palette {
            bg_top: Color(0xff17120e), bg_bottom: Color(0xff0b0907), header: Color(0xff110d09),
            panel: Color(0xff100d09), panel_edge: Color(0xff755d3a),
            cell: Color(0xff1b150d), cell_bottom: Color(0xff110d08),
            screen: Color(0xff0b0804), bezel: Color(0xff30251a),
            grid: Color(0xff40301b), tick: Color(0xff9b7a49),
            label: Color(0xffc8b18c), bright: Color(0xffffe4b8), white: Color(0xffffefcf),
            dim: Color(0xffb49b75), phosphor: Color(0xffffc15d), blue: Color(0xffdfc292),
            button: Color(0xff3b2d1b), button_bottom: Color(0xff20170e),
            active: Color(0xffffd17d), active_bottom: Color(0xffb77931), active_text: Color(0xff23170a),
            heat: [Color(0xff6b4b24), Color(0xff9c6b2e), Color(0xffc38a39), Color(0xffe3ad50), Color(0xffffc570), Color(0xffff8c55)],
            ..dark
        },
        _ => dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(c: Color) -> f64 {
        let linear = |shift: u32| {
            let value = ((c.0 >> shift) & 255u32) as f64 / 255.0;
            if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * linear(16) + 0.7152 * linear(8) + 0.0722 * linear(0)
    }

    #[test]
    fn all_skins_keep_text_and_active_controls_readable() {
        for i in 0..6 {
            let p = palette(i);
            for (foreground, background) in [
                (p.label, p.cell), (p.label, p.header), (p.label, p.button),
                (p.bright, p.panel), (p.active_text, p.active), (p.active_text, p.active_bottom), (p.blue, p.cell),
                (p.red, p.cell), (p.amber, p.cell), (p.green, p.cell),
            ] {
                let a = luminance(foreground);
                let b = luminance(background);
                let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                assert!(ratio >= 4.5, "skin {i}, {:x} on {:x}: {ratio:.2}", foreground.0, background.0);
            }
        }
    }
}
