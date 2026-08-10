//! Loss-coefficient lookup for round elbows.
//!
//! The Python reference looks `zeta` up from a tabulated grid using scipy's
//! `RectBivariateSpline`. To stay dependency-free, this port interpolates the
//! same table with **bilinear** interpolation in (R/D, angle) space. Bilinear
//! reproduces the source table at vertices and stays within a few percent of
//! the spline between them — a documented approximation for a design-tool
//! coefficient.
//!
//! Source: Wentylacja i Klimatyzacja — Materiały pomocnicze do projektowania,
//! Jacek Hendiger, Piotr Ziętek, Marta Chludzińska.

/// R/D grid points.
pub const RD_GRID: [f64; 6] = [0.50, 0.75, 1.00, 1.50, 2.00, 2.50];
/// Bend-angle grid points [deg].
pub const ANGLE_GRID: [f64; 10] = [20., 30., 45., 60., 75., 90., 110., 130., 150., 180.];
/// Loss coefficient table (rows: R/D; cols: angle).
pub const ZETA_TABLE: [[f64; 10]; 6] = [
    [0.22, 0.32, 0.43, 0.55, 0.64, 0.71, 0.80, 0.85, 0.91, 0.99],
    [0.10, 0.15, 0.20, 0.26, 0.30, 0.33, 0.37, 0.40, 0.42, 0.46],
    [0.07, 0.10, 0.13, 0.17, 0.20, 0.22, 0.25, 0.26, 0.28, 0.31],
    [0.05, 0.07, 0.09, 0.12, 0.14, 0.15, 0.17, 0.18, 0.19, 0.21],
    [0.04, 0.06, 0.08, 0.10, 0.12, 0.13, 0.15, 0.16, 0.17, 0.18],
    [0.04, 0.05, 0.07, 0.09, 0.11, 0.12, 0.14, 0.14, 0.15, 0.17],
];

/// Round elbow with an interpolated loss coefficient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElbowRound {
    pub bend_radius: f64,
    pub diameter: f64,
    pub angle: f64, // [deg]
}

impl ElbowRound {
    pub fn new(bend_radius: f64, diameter: f64, angle: f64) -> Result<Self, String> {
        if diameter <= 0.0 {
            return Err(format!("diameter must be positive, got {diameter}"));
        }
        if bend_radius <= 0.0 {
            return Err(format!("bend_radius must be positive, got {bend_radius}"));
        }
        let rd = bend_radius / diameter;
        if !(RD_GRID[0] <= rd && rd <= RD_GRID[RD_GRID.len() - 1]) {
            return Err(format!(
                "R/D = {rd:.3} is outside the tabulated range [{}, {}]",
                RD_GRID[0],
                RD_GRID[RD_GRID.len() - 1]
            ));
        }
        if !(ANGLE_GRID[0] <= angle && angle <= ANGLE_GRID[ANGLE_GRID.len() - 1]) {
            return Err(format!(
                "angle = {angle} is outside the tabulated range [{}, {}]",
                ANGLE_GRID[0],
                ANGLE_GRID[ANGLE_GRID.len() - 1]
            ));
        }
        Ok(ElbowRound {
            bend_radius,
            diameter,
            angle,
        })
    }

    /// Interpolated loss coefficient (dimensionless).
    pub fn zeta(&self) -> f64 {
        bilinear(self.bend_radius / self.diameter, self.angle)
    }
}

/// Bilinear interpolation over `(rd, angle)` in the elbow table.
pub fn bilinear(rd: f64, angle: f64) -> f64 {
    let xi = locate(&RD_GRID, rd);
    let yi = locate(&ANGLE_GRID, angle);
    let x0 = RD_GRID[xi];
    let x1 = RD_GRID[xi + 1];
    let y0 = ANGLE_GRID[yi];
    let y1 = ANGLE_GRID[yi + 1];
    let tx = (rd - x0) / (x1 - x0);
    let ty = (angle - y0) / (y1 - y0);

    let f00 = ZETA_TABLE[xi][yi];
    let f10 = ZETA_TABLE[xi + 1][yi];
    let f01 = ZETA_TABLE[xi][yi + 1];
    let f11 = ZETA_TABLE[xi + 1][yi + 1];

    (1.0 - tx) * (1.0 - ty) * f00 + tx * (1.0 - ty) * f10 + (1.0 - tx) * ty * f01 + tx * ty * f11
}

/// Index `i` such that `grid[i] <= value <= grid[i+1]` (clamped).
fn locate(grid: &[f64], value: f64) -> usize {
    for i in 0..grid.len() - 1 {
        if value <= grid[i + 1] {
            return i;
        }
    }
    grid.len() - 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeta_at_grid_vertex_matches_table() {
        // R/D = 1.0, angle = 90 -> table[2][5] = 0.22
        let e = ElbowRound::new(0.4, 0.4, 90.0).unwrap();
        assert!((e.zeta() - 0.22).abs() < 1e-12, "zeta = {}", e.zeta());
    }

    #[test]
    fn zeta_at_another_vertex() {
        // R/D = 0.5, angle = 45 -> table[0][2] = 0.43
        let e = ElbowRound::new(0.2, 0.4, 45.0).unwrap();
        assert!((e.zeta() - 0.43).abs() < 1e-12);
    }

    #[test]
    fn larger_radius_reduces_loss() {
        let tight = ElbowRound::new(0.2, 0.4, 90.0).unwrap();
        let wide = ElbowRound::new(0.4, 0.4, 90.0).unwrap();
        assert!(wide.zeta() < tight.zeta());
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(ElbowRound::new(0.01, 0.4, 90.0).is_err()); // R/D too small
        assert!(ElbowRound::new(1.0, 0.4, 200.0).is_err()); // angle too big
        assert!(ElbowRound::new(0.4, 0.0, 90.0).is_err());
    }
}
