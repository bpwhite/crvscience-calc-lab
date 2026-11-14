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

pub fn ln_gamma_int(n: u32) -> f64 {
    // ln((n-1)!) for integer factorials
    if n <= 2 {
        0.0
    } else {
        (2..n).map(|k| (k as f64).ln()).sum()
    }
}

pub fn ln_factorial(k: u32) -> f64 {
    ln_gamma_int(k + 1)
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
    let n = points.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mean_x = points.iter().map(|p| p[0]).sum::<f64>() / nf;
    let mean_y = points.iter().map(|p| p[1]).sum::<f64>() / nf;

    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;
    for p in points {
        let dx = p[0] - mean_x;
        let dy = p[1] - mean_y;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if sxx < 1e-12 {
        return None;
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let r_corr = if syy > 1e-12 {
        (sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let r_squared = r_corr * r_corr;

    let sse: f64 = points
        .iter()
        .map(|p| {
            let resid = p[1] - (intercept + slope * p[0]);
            resid * resid
        })
        .sum();
    let dof = (n.saturating_sub(2)).max(1) as f64;
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
