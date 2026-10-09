//! Ikona stanu w pasku menu: szare kółko poza nagrywaniem, czerwone w trakcie (VA-UI-1).
//!
//! Kółka rysujemy w kodzie (RGBA 36×36 px = 18 pt na ekranie Retina), zamiast trzymać
//! pliki PNG — ten sam efekt bez binarnych zasobów, a kolor i rozmiar są sprawdzalne testem.

use va_core::state::State;

pub const ICON_PIXELS: u32 = 36;

const GRAY: [u8; 3] = [142, 142, 147];
const RED: [u8; 3] = [255, 59, 48];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dot {
    Gray,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indicator {
    pub dot: Dot,
    pub tooltip: &'static str,
}

pub fn indicator_for(state: State) -> Indicator {
    match state {
        State::Idle => Indicator {
            dot: Dot::Gray,
            tooltip: "VoiceAsystent — kliknij albo ctrl+cmd+r, aby nagrywać",
        },
        State::Recording => Indicator {
            dot: Dot::Red,
            tooltip: "Nagrywanie… — kliknij albo ctrl+cmd+s, aby zakończyć",
        },
        State::Transcribing => Indicator {
            dot: Dot::Gray,
            tooltip: "Transkrypcja…",
        },
        State::Error => Indicator {
            dot: Dot::Gray,
            tooltip: "VoiceAsystent — ostatnie nagranie nieudane",
        },
    }
}

/// Wypełnione kółko z wygładzoną krawędzią na przezroczystym tle, RGBA wierszami.
pub fn dot_rgba(dot: Dot) -> Vec<u8> {
    let [red, green, blue] = match dot {
        Dot::Gray => GRAY,
        Dot::Red => RED,
    };
    let size = ICON_PIXELS as f32;
    let center = size / 2.0;
    let radius = size * 0.35;
    let mut rgba = Vec::with_capacity((ICON_PIXELS * ICON_PIXELS * 4) as usize);
    for y in 0..ICON_PIXELS {
        for x in 0..ICON_PIXELS {
            let distance =
                ((x as f32 + 0.5 - center).powi(2) + (y as f32 + 0.5 - center).powi(2)).sqrt();
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[red, green, blue, (coverage * 255.0).round() as u8]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
        let index = ((y * ICON_PIXELS + x) * 4) as usize;
        rgba[index..index + 4].try_into().unwrap()
    }

    #[test]
    // specky: crit 01M4EKHD1PEEATZBYQYNAE6NGM
    fn idle_shows_gray_dot() {
        assert_eq!(indicator_for(State::Idle).dot, Dot::Gray);
    }

    #[test]
    // specky: crit 01M4EKHD1PF30TMM4X3RP8Q5MY
    fn recording_shows_red_dot() {
        assert_eq!(indicator_for(State::Recording).dot, Dot::Red);
    }

    #[test]
    // specky: crit 01M4EKHD1PJ32E01218FSE9G8F
    fn after_recording_the_dot_is_gray_again() {
        assert_eq!(indicator_for(State::Transcribing).dot, Dot::Gray);
        assert_eq!(indicator_for(State::Idle).dot, Dot::Gray);
        assert_eq!(indicator_for(State::Error).dot, Dot::Gray);
    }

    #[test]
    fn transcribing_tooltip_says_so() {
        assert_eq!(indicator_for(State::Transcribing).tooltip, "Transkrypcja…");
    }

    #[test]
    fn dot_is_opaque_in_center_and_transparent_in_corners() {
        let red = dot_rgba(Dot::Red);

        assert_eq!(red.len(), (ICON_PIXELS * ICON_PIXELS * 4) as usize);
        assert_eq!(pixel(&red, 18, 18), [255, 59, 48, 255]);
        assert_eq!(pixel(&red, 0, 0)[3], 0);
        assert_eq!(pixel(&red, 35, 35)[3], 0);
        assert_eq!(pixel(&dot_rgba(Dot::Gray), 18, 18), [142, 142, 147, 255]);
    }
}
