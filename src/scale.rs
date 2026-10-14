//! Small numeric mapping helpers shared by widgets.

/// Linear interpolation of a byte channel, rounded.
pub(crate) fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u8
}

/// Interpolate two RGB triples at `t`.
pub(crate) fn mix_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        lerp_u8(a.0, b.0, t),
        lerp_u8(a.1, b.1, t),
        lerp_u8(a.2, b.2, t),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_endpoints_and_mid() {
        assert_eq!(mix_rgb((0, 0, 0), (200, 100, 50), 0.0), (0, 0, 0));
        assert_eq!(mix_rgb((0, 0, 0), (200, 100, 50), 1.0), (200, 100, 50));
        assert_eq!(mix_rgb((0, 0, 0), (200, 100, 50), 0.5), (100, 50, 25));
        assert_eq!(lerp_u8(255, 0, 0.5), 128);
    }
}
