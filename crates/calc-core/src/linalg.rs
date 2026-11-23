// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat2 {
    pub a11: f64,
    pub a12: f64,
    pub a21: f64,
    pub a22: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EigenSpectrum2D {
    RealDistinct {
        lambda1: f64,
        lambda2: f64,
        vec1: [f64; 2],
        vec2: [f64; 2],
    },
    RealRepeated {
        lambda: f64,
        primary_vec: [f64; 2],
        geometric_mult: usize,
    },
    ComplexConjugate {
        real_part: f64,
        imag_part: f64,
    },
}

#[derive(Debug, Clone)]
pub struct Mat2Diagnostics {
    pub trace: f64,
    pub det: f64,
    pub discriminant: f64,
    pub rank: usize,
    pub singular_values: (f64, f64),
    pub inverse: Option<Mat2>,
    pub spectrum: EigenSpectrum2D,
    pub phase_portrait_class: &'static str,
}

impl Mat2 {
    pub const fn new(a11: f64, a12: f64, a21: f64, a22: f64) -> Self {
        Self { a11, a12, a21, a22 }
    }

    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    pub fn trace(&self) -> f64 {
        self.a11 + self.a22
    }

    pub fn det(&self) -> f64 {
        self.a11 * self.a22 - self.a12 * self.a21
    }

    pub fn apply(&self, v: [f64; 2]) -> [f64; 2] {
        [
            self.a11 * v[0] + self.a12 * v[1],
            self.a21 * v[0] + self.a22 * v[1],
        ]
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Self {
            a11: self.a11 * rhs.a11 + self.a12 * rhs.a21,
            a12: self.a11 * rhs.a12 + self.a12 * rhs.a22,
            a21: self.a21 * rhs.a11 + self.a22 * rhs.a21,
            a22: self.a21 * rhs.a12 + self.a22 * rhs.a22,
        }
    }

    pub fn transpose(&self) -> Self {
        Self::new(self.a11, self.a21, self.a12, self.a22)
    }

    pub fn inverse(&self) -> Option<Self> {
        let d = self.det();
        let (s1, s2) = self.singular_values();
        if !d.is_finite() || d == 0.0 || s1 == 0.0 || s2 <= 1e-12 * s1 {
            None
        } else {
            let inv_d = 1.0 / d;
            Some(Self::new(
                self.a22 * inv_d,
                -self.a12 * inv_d,
                -self.a21 * inv_d,
                self.a11 * inv_d,
            ))
        }
    }

    pub fn rank(&self) -> usize {
        let (s1, s2) = self.singular_values();
        if !s1.is_finite() || s1 == 0.0 {
            0
        } else if s2 > 1e-12 * s1 {
            2
        } else {
            1
        }
    }

    pub fn singular_values(&self) -> (f64, f64) {
        // Direct 2x2 SVD using (a11 +/- a22)^2 + (a21 -/+ a12)^2 via overflow-safe hypot
        // after max-entry scaling, avoiding condition-number squaring in A^T * A.
        let scale = self
            .a11
            .abs()
            .max(self.a12.abs())
            .max(self.a21.abs())
            .max(self.a22.abs());
        if !scale.is_finite() || scale == 0.0 {
            return (0.0, 0.0);
        }
        let b11 = self.a11 / scale;
        let b12 = self.a12 / scale;
        let b21 = self.a21 / scale;
        let b22 = self.a22 / scale;

        let s_plus = (b11 + b22).hypot(b21 - b12);
        let s_minus = (b11 - b22).hypot(b21 + b12);
        let s1_scaled = 0.5 * (s_plus + s_minus);
        let det_scaled = (b11 * b22 - b12 * b21).abs();
        let s2_scaled = if s1_scaled > 0.0 {
            (det_scaled / s1_scaled).min(s1_scaled)
        } else {
            0.0
        };
        (s1_scaled * scale, s2_scaled * scale)
    }

    fn eigenvector_for(&self, lambda: f64) -> [f64; 2] {
        let r1x = self.a11 - lambda;
        let r1y = self.a12;
        let r2x = self.a21;
        let r2y = self.a22 - lambda;
        let scale = (self.a11.abs() + self.a12.abs() + self.a21.abs() + self.a22.abs()).max(1e-30);
        let zero_tol = 1e-11 * scale;

        let (vx, vy) = if r1x.abs() + r1y.abs() >= r2x.abs() + r2y.abs() {
            if r1x.abs() + r1y.abs() < zero_tol {
                (1.0, 0.0)
            } else {
                (-r1y, r1x)
            }
        } else if r2x.abs() + r2y.abs() < zero_tol {
            (1.0, 0.0)
        } else {
            (-r2y, r2x)
        };

        let norm = vx.hypot(vy);
        if norm == 0.0 || !norm.is_finite() {
            [1.0, 0.0]
        } else {
            [vx / norm, vy / norm]
        }
    }

    pub fn diagnostics(&self) -> Mat2Diagnostics {
        let tr = self.trace();
        let det = self.det();
        let disc = tr * tr - 4.0 * det;
        let frob_sq = self.a11 * self.a11
            + self.a12 * self.a12
            + self.a21 * self.a21
            + self.a22 * self.a22;
        let frob = frob_sq.sqrt();
        // Scale quadratic tolerances (det, disc) by ||A||_F^2 and linear tolerances (tr, scalar check) by ||A||_F.
        let eps_sq = 1e-12 * frob_sq;
        let eps_lin = 1e-7 * frob;

        let spectrum = if disc > eps_sq {
            let root = disc.sqrt();
            let l1 = 0.5 * (tr + root);
            let l2 = 0.5 * (tr - root);
            EigenSpectrum2D::RealDistinct {
                lambda1: l1,
                lambda2: l2,
                vec1: self.eigenvector_for(l1),
                vec2: self.eigenvector_for(l2),
            }
        } else if disc < -eps_sq {
            EigenSpectrum2D::ComplexConjugate {
                real_part: 0.5 * tr,
                imag_part: 0.5 * (-disc).sqrt(),
            }
        } else {
            let l = 0.5 * tr;
            let is_scalar = (self.a11 - l).abs() <= eps_lin
                && self.a12.abs() <= eps_lin
                && self.a21.abs() <= eps_lin
                && (self.a22 - l).abs() <= eps_lin;
            EigenSpectrum2D::RealRepeated {
                lambda: l,
                primary_vec: self.eigenvector_for(l),
                geometric_mult: if is_scalar { 2 } else { 1 },
            }
        };

        let phase_portrait_class = if det < -eps_sq {
            "Saddle Point (real opposite-sign eigenvalues)"
        } else if det.abs() <= eps_sq {
            "Degenerate Line of Equilibria (det = 0)"
        } else if disc < -eps_sq {
            if tr < -eps_lin {
                "Asymptotically Stable Spiral Sink (alpha < 0)"
            } else if tr > eps_lin {
                "Unstable Spiral Source (alpha > 0)"
            } else {
                "Lyapunov Neutral Center (purely imaginary +/- i*beta)"
            }
        } else if disc > eps_sq {
            if tr < -eps_lin {
                "Asymptotically Stable Node / Sink (0 > lambda_1 > lambda_2)"
            } else {
                "Unstable Node / Source (lambda_1 > lambda_2 > 0)"
            }
        } else if tr < 0.0 {
            "Stable Star / Improper Node (repeated lambda < 0)"
        } else {
            "Unstable Star / Improper Node (repeated lambda > 0)"
        };

        Mat2Diagnostics {
            trace: tr,
            det,
            discriminant: disc,
            rank: self.rank(),
            singular_values: self.singular_values(),
            inverse: self.inverse(),
            spectrum,
            phase_portrait_class,
        }
    }
}

pub fn det3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

pub fn orthogonal_projection_2d(target: [f64; 2], onto: [f64; 2]) -> ([f64; 2], f64) {
    let denom = onto[0] * onto[0] + onto[1] * onto[1];
    if denom < 1e-12 {
        return ([0.0, 0.0], 0.0);
    }
    let scale = (target[0] * onto[0] + target[1] * onto[1]) / denom;
    ([scale * onto[0], scale * onto[1]], scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_scalar_matrix_scales_tolerances_by_frobenius_norm() {
        let m_pos = Mat2::new(1e-4, 0.0, 0.0, 1e-4);
        let d_pos = m_pos.diagnostics();
        assert_eq!(d_pos.rank, 2);
        assert!(d_pos.inverse.is_some());
        assert_eq!(
            d_pos.phase_portrait_class,
            "Unstable Star / Improper Node (repeated lambda > 0)"
        );
        assert!(matches!(
            d_pos.spectrum,
            EigenSpectrum2D::RealRepeated {
                lambda,
                geometric_mult: 2,
                ..
            } if (lambda - 1e-4).abs() < 1e-12
        ));
        assert!((d_pos.singular_values.0 - 1e-4).abs() < 1e-15);
        assert!((d_pos.singular_values.1 - 1e-4).abs() < 1e-15);

        let m_neg = Mat2::new(-1e-4, 0.0, 0.0, -1e-4);
        let d_neg = m_neg.diagnostics();
        assert_eq!(d_neg.rank, 2);
        assert_eq!(
            d_neg.phase_portrait_class,
            "Stable Star / Improper Node (repeated lambda < 0)"
        );

        let m_zero = Mat2::new(0.0, 0.0, 0.0, 0.0);
        let d_zero = m_zero.diagnostics();
        assert_eq!(d_zero.rank, 0);
        assert!(d_zero.inverse.is_none());
        assert_eq!(
            d_zero.phase_portrait_class,
            "Degenerate Line of Equilibria (det = 0)"
        );
    }

    #[test]
    fn direct_svd_preserves_high_condition_number_singular_values() {
        let m = Mat2::new(1.0, 0.0, 0.0, 1e-9);
        let (s1, s2) = m.singular_values();
        assert!((s1 - 1.0).abs() < 1e-15, "s1={s1}");
        assert!((s2 - 1e-9).abs() < 1e-15, "s2={s2}");
        assert_eq!(m.rank(), 2);
        assert!(m.inverse().is_some());
        let d = m.diagnostics();
        assert_eq!(
            d.phase_portrait_class,
            "Unstable Node / Source (lambda_1 > lambda_2 > 0)"
        );

        let m_big = Mat2::new(1e9, 0.0, 0.0, 1.0);
        let (b1, b2) = m_big.singular_values();
        assert!((b1 - 1e9).abs() < 1e-6, "b1={b1}");
        assert!((b2 - 1.0).abs() < 1e-12, "b2={b2}");

        // Rotated ill-conditioned matrix: R(theta) * diag(1, 1e-9) * R(phi)
        let theta = 0.6_f64;
        let phi = -0.4_f64;
        let u = Mat2::new(theta.cos(), -theta.sin(), theta.sin(), theta.cos());
        let v_t = Mat2::new(phi.cos(), -phi.sin(), phi.sin(), phi.cos());
        let rotated = u.mul(&Mat2::new(1.0, 0.0, 0.0, 1e-9)).mul(&v_t);
        let (rs1, rs2) = rotated.singular_values();
        assert!((rs1 - 1.0).abs() < 1e-12, "rs1={rs1}");
        assert!((rs2 - 1e-9).abs() < 1e-15, "rs2={rs2}");

        // Rank-1 singular matrix
        let rank1 = Mat2::new(0.8, 0.4, 0.4, 0.2);
        let (r1, r2) = rank1.singular_values();
        assert!((r1 - 1.0).abs() < 1e-12);
        assert!(r2 < 1e-12);
        assert_eq!(rank1.rank(), 1);
        assert!(rank1.inverse().is_none());
    }
}
