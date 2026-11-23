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
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionKind {
    Normal,
    Exponential,
    Poisson,
    Binomial,
}

impl DistributionKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal / Gaussian N(mu, sigma^2)",
            Self::Exponential => "Exponential Waiting Time Exp(lambda)",
            Self::Poisson => "Poisson Discrete Count Pois(lambda)",
            Self::Binomial => "Binomial Trials Bin(n, p)",
        }
    }
}

/// High-accuracy Abramowitz-Stegun 7.1.26 rational approximation to erf(x) (max error < 1.5e-7).
pub fn erf_approx(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let ax = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * ax);
    let poly = t
        * (0.254829592
            + t * (-0.284496736
                + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    sign * (1.0 - poly * (-ax * ax).exp())
}

pub fn normal_pdf(x: f64, mu: f64, sigma: f64) -> f64 {
    let s = sigma.max(1e-6);
    let z = (x - mu) / s;
    (-0.5 * z * z).exp() / (s * (2.0 * PI).sqrt())
}

pub fn normal_cdf(x: f64, mu: f64, sigma: f64) -> f64 {
    let s = sigma.max(1e-6);
    let z = (x - mu) / (s * std::f64::consts::SQRT_2);
    0.5 * (1.0 + erf_approx(z))
}

pub fn exponential_pdf(x: f64, rate: f64) -> f64 {
    if x < 0.0 || rate <= 0.0 {
        0.0
    } else {
        rate * (-rate * x).exp()
    }
}

pub fn exponential_cdf(x: f64, rate: f64) -> f64 {
    if x <= 0.0 || rate <= 0.0 {
        0.0
    } else {
        1.0 - (-rate * x).exp()
    }
}

const LANCZOS_G: f64 = 7.0;
const LANCZOS_COEFFS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];

/// Fast O(1) Lanczos approximation to ln(Gamma(z)) for positive real z (with reflection for z < 0.5).
pub fn ln_gamma(z: f64) -> f64 {
    if !z.is_finite() {
        return f64::NAN;
    }
    if z < 0.5 {
        let s = (PI * z).sin();
        if s == 0.0 {
            return f64::INFINITY;
        }
        return PI.ln() - s.abs().ln() - ln_gamma(1.0 - z);
    }
    let zm1 = z - 1.0;
    let mut x = LANCZOS_COEFFS[0];
    for (idx, &coeff) in LANCZOS_COEFFS.iter().enumerate().skip(1) {
        x += coeff / (zm1 + idx as f64);
    }
    let t = zm1 + LANCZOS_G + 0.5;
    0.5 * (2.0 * PI).ln() + (zm1 + 0.5) * t.ln() - t + x.ln()
}

pub fn ln_gamma_int(n: u32) -> f64 {
    // ln((n-1)!) for integer factorials
    if n == 0 {
        f64::INFINITY
    } else if n <= 2 {
        0.0
    } else {
        ln_gamma(n as f64)
    }
}

pub fn ln_factorial(k: u32) -> f64 {
    if k <= 1 {
        0.0
    } else {
        ln_gamma((k as f64) + 1.0)
    }
}

pub fn poisson_pmf(k: u32, lambda: f64) -> f64 {
    if lambda <= 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    ((k as f64) * lambda.ln() - lambda - ln_factorial(k)).exp()
}

pub fn binomial_pmf(k: u32, n: u32, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    let pc = p.clamp(0.0, 1.0);
    if pc <= 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if pc >= 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }
    let ln_choose = ln_factorial(n) - ln_factorial(k) - ln_factorial(n - k);
    (ln_choose + (k as f64) * pc.ln() + ((n - k) as f64) * (1.0 - pc).ln()).exp()
}

#[derive(Debug, Clone, Copy)]
pub struct BayesDiagnosticSummary {
    pub prior: f64,
    pub sensitivity: f64,
    pub specificity: f64,
    pub false_positive_rate: f64,
    pub marginal_positive: f64,
    pub posterior_given_pos: f64,
    pub posterior_given_neg: f64,
}

pub fn compute_bayes_binary(
    prior: f64,
    sensitivity: f64,
    specificity: f64,
) -> BayesDiagnosticSummary {
    let p = prior.clamp(0.0, 1.0);
    let sens = sensitivity.clamp(0.0, 1.0);
    let spec = specificity.clamp(0.0, 1.0);
    let fpr = 1.0 - spec;
    let p_pos = sens * p + fpr * (1.0 - p);
    let post_pos = if p_pos > 1e-12 {
        (sens * p) / p_pos
    } else {
        0.0
    };
    let p_neg = (1.0 - sens) * p + spec * (1.0 - p);
    let post_neg = if p_neg > 1e-12 {
        ((1.0 - sens) * p) / p_neg
    } else {
        0.0
    };

    BayesDiagnosticSummary {
        prior: p,
        sensitivity: sens,
        specificity: spec,
        false_positive_rate: fpr,
        marginal_positive: p_pos,
        posterior_given_pos: post_pos,
        posterior_given_neg: post_neg,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OlsRegressionSummary {
    pub n: usize,
    pub mean_x: f64,
    pub mean_y: f64,
    pub slope: f64,
    pub intercept: f64,
    pub r_corr: f64,
    pub r_squared: f64,
    pub residual_std_err: f64,
}

pub fn fit_ols_linear(points: &[[f64; 2]]) -> Option<OlsRegressionSummary> {
    let finite: Vec<[f64; 2]> = points
        .iter()
        .copied()
        .filter(|p| p[0].is_finite() && p[1].is_finite())
        .collect();
    let n = finite.len();
    // Require n >= 3 finite points so residual degrees of freedom (df = n - 2) >= 1.
    if n <= 2 {
        return None;
    }
    let nf = n as f64;
    let mean_x = finite.iter().map(|p| p[0]).sum::<f64>() / nf;
    let mean_y = finite.iter().map(|p| p[1]).sum::<f64>() / nf;

    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;
    for p in &finite {
        let dx = p[0] - mean_x;
        let dy = p[1] - mean_y;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if !sxx.is_finite() || sxx <= 1e-12 {
        return None;
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    if !slope.is_finite() || !intercept.is_finite() {
        return None;
    }
    let r_corr = if syy > 1e-12 {
        (sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let r_squared = r_corr * r_corr;

    let sse: f64 = finite
        .iter()
        .map(|p| {
            let resid = p[1] - (intercept + slope * p[0]);
            resid * resid
        })
        .sum();
    if !sse.is_finite() {
        return None;
    }
    let dof = (n - 2) as f64;
    let residual_std_err = (sse / dof).sqrt();

    Some(OlsRegressionSummary {
        n,
        mean_x,
        mean_y,
        slope,
        intercept,
        r_corr,
        r_squared,
        residual_std_err,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lanczos_ln_factorial_matches_exact_small_and_large_n() {
        let mut exact_sum = 0.0_f64;
        for k in 0..=20_u32 {
            if k >= 2 {
                exact_sum += (k as f64).ln();
            }
            let approx = ln_factorial(k);
            let err = (approx - exact_sum).abs();
            assert!(
                err < 1e-13,
                "ln_factorial({k}) error too large: approx={approx}, exact={exact_sum}, err={err}"
            );
        }

        // Reflection identity: Gamma(0.5) = sqrt(pi)
        let g_half = ln_gamma(0.5);
        assert!((g_half - 0.5 * PI.ln()).abs() < 1e-14);

        // Large n checked against high-order Stirling asymptotic expansion
        for &n in &[1_000_u32, 100_000_u32] {
            let nf = n as f64;
            let stirling = nf * nf.ln() - nf
                + 0.5 * (2.0 * PI * nf).ln()
                + 1.0 / (12.0 * nf)
                - 1.0 / (360.0 * nf.powi(3));
            let approx = ln_factorial(n);
            let rel_err = (approx - stirling).abs() / stirling;
            assert!(
                rel_err < 1e-14,
                "ln_factorial({n}) rel_err={rel_err} (approx={approx}, stirling={stirling})"
            );
        }

        // PMF base case sanity
        assert!((poisson_pmf(0, 2.5) - (-2.5_f64).exp()).abs() < 1e-15);
    }

    #[test]
    fn fit_ols_linear_filters_non_finite_and_guards_n_le_2() {
        assert!(fit_ols_linear(&[]).is_none());
        assert!(fit_ols_linear(&[[0.0, 1.0]]).is_none());
        // Exactly 2 finite points -> df = n - 2 = 0 -> must return None
        assert!(fit_ols_linear(&[[0.0, 1.0], [1.0, 3.0]]).is_none());

        // Non-finite points leaving <= 2 finite points -> None
        assert!(fit_ols_linear(&[
            [0.0, 1.0],
            [1.0, 3.0],
            [f64::NAN, 5.0],
            [2.0, f64::INFINITY],
        ])
        .is_none());

        // Non-finite points filtered cleanly when >= 3 finite points remain
        let fit = fit_ols_linear(&[
            [0.0, 1.0],
            [f64::NAN, 99.0],
            [1.0, 3.0],
            [3.0, f64::NEG_INFINITY],
            [2.0, 5.0],
        ])
        .expect("3 finite collinear points produce exact OLS fit");
        assert_eq!(fit.n, 3);
        assert!((fit.slope - 2.0).abs() < 1e-12);
        assert!((fit.intercept - 1.0).abs() < 1e-12);
        assert!((fit.r_squared - 1.0).abs() < 1e-12);
        assert!(fit.residual_std_err < 1e-12);

        // Collinear vertical x points -> None
        assert!(fit_ols_linear(&[[1.0, 1.0], [1.0, 2.0], [1.0, 3.0]]).is_none());
    }
}
