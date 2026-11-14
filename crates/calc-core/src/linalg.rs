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
        if d.abs() < 1e-11 {
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
        if self.det().abs() > 1e-9 {
            2
        } else if self.a11.abs() > 1e-9
            || self.a12.abs() > 1e-9
            || self.a21.abs() > 1e-9
            || self.a22.abs() > 1e-9
        {
            1
        } else {
            0
        }
    }

    pub fn singular_values(&self) -> (f64, f64) {
        // Eigenvalues of A^T * A
        let ata = self.transpose().mul(self);
        let tr = ata.trace().max(0.0);
        let det = ata.det().max(0.0);
        let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
        let s1 = (0.5 * (tr + disc)).max(0.0).sqrt();
        let s2 = (0.5 * (tr - disc)).max(0.0).sqrt();
        (s1, s2)
    }

    fn eigenvector_for(&self, lambda: f64) -> [f64; 2] {
        let r1x = self.a11 - lambda;
        let r1y = self.a12;
        let r2x = self.a21;
        let r2y = self.a22 - lambda;

        let (vx, vy) = if r1x.abs() + r1y.abs() >= r2x.abs() + r2y.abs() {
            if r1x.abs() + r1y.abs() < 1e-11 {
                (1.0, 0.0)
            } else {
                (-r1y, r1x)
            }
        } else if r2x.abs() + r2y.abs() < 1e-11 {
            (1.0, 0.0)
        } else {
            (-r2y, r2x)
        };

        let norm = (vx * vx + vy * vy).sqrt();
        if norm < 1e-12 {
            [1.0, 0.0]
        } else {
            [vx / norm, vy / norm]
        }
    }

    pub fn diagnostics(&self) -> Mat2Diagnostics {
        let tr = self.trace();
        let det = self.det();
        let disc = tr * tr - 4.0 * det;
        let eps = 1e-7;

        let spectrum = if disc > eps {
            let root = disc.sqrt();
            let l1 = 0.5 * (tr + root);
            let l2 = 0.5 * (tr - root);
            EigenSpectrum2D::RealDistinct {
                lambda1: l1,
                lambda2: l2,
                vec1: self.eigenvector_for(l1),
                vec2: self.eigenvector_for(l2),
            }
        } else if disc < -eps {
            EigenSpectrum2D::ComplexConjugate {
                real_part: 0.5 * tr,
                imag_part: 0.5 * (-disc).sqrt(),
            }
        } else {
            let l = 0.5 * tr;
            let is_scalar = (self.a11 - l).abs() < 1e-7
                && self.a12.abs() < 1e-7
                && self.a21.abs() < 1e-7
                && (self.a22 - l).abs() < 1e-7;
            EigenSpectrum2D::RealRepeated {
                lambda: l,
                primary_vec: self.eigenvector_for(l),
                geometric_mult: if is_scalar { 2 } else { 1 },
            }
        };

        let phase_portrait_class = if det < -eps {
            "Saddle Point (real opposite-sign eigenvalues)"
        } else if det.abs() <= eps {
            "Degenerate Line of Equilibria (det = 0)"
        } else if disc < -eps {
            if tr < -eps {
                "Asymptotically Stable Spiral Sink (alpha < 0)"
            } else if tr > eps {
                "Unstable Spiral Source (alpha > 0)"
            } else {
                "Lyapunov Neutral Center (purely imaginary +/- i*beta)"
            }
        } else if disc > eps {
            if tr < -eps {
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
