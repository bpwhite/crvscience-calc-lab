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
use std::collections::HashMap;
use std::f64::consts::{E, PI};

pub type EvalEnv = HashMap<String, f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathFunc {
    Sin,
    Cos,
    Tan,
    Sec,
    Csc,
    Cot,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Exp,
    Ln,
    Sqrt,
    Abs,
}

impl MathFunc {
    pub fn from_ident(ident: &str) -> Option<Self> {
        match ident {
            "sin" => Some(Self::Sin),
            "cos" => Some(Self::Cos),
            "tan" => Some(Self::Tan),
            "sec" => Some(Self::Sec),
            "csc" => Some(Self::Csc),
            "cot" => Some(Self::Cot),
            "asin" | "arcsin" => Some(Self::Asin),
            "acos" | "arccos" => Some(Self::Acos),
            "atan" | "arctan" => Some(Self::Atan),
            "sinh" => Some(Self::Sinh),
            "cosh" => Some(Self::Cosh),
            "tanh" => Some(Self::Tanh),
            "exp" => Some(Self::Exp),
            "ln" | "log" => Some(Self::Ln),
            "sqrt" => Some(Self::Sqrt),
            "abs" => Some(Self::Abs),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Sec => "sec",
            Self::Csc => "csc",
            Self::Cot => "cot",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Exp => "exp",
            Self::Ln => "ln",
            Self::Sqrt => "sqrt",
            Self::Abs => "abs",
        }
    }

    pub fn apply(&self, v: f64) -> f64 {
        match self {
            Self::Sin => v.sin(),
            Self::Cos => v.cos(),
            Self::Tan => v.tan(),
            Self::Sec => 1.0 / v.cos(),
            Self::Csc => 1.0 / v.sin(),
            Self::Cot => v.cos() / v.sin(),
            Self::Asin => v.asin(),
            Self::Acos => v.acos(),
            Self::Atan => v.atan(),
            Self::Sinh => v.sinh(),
            Self::Cosh => v.cosh(),
            Self::Tanh => v.tanh(),
            Self::Exp => v.exp(),
            Self::Ln => {
                if v <= 0.0 {
                    f64::NAN
                } else {
                    v.ln()
                }
            }
            Self::Sqrt => {
                if v < 0.0 {
                    f64::NAN
                } else {
                    v.sqrt()
                }
            }
            Self::Abs => v.abs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Const(f64),
    Pi,
    EulerE,
    Var(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Func(MathFunc, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' | '−' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' | '·' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '(' | '[' | '{' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' | ']' | '}' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '|' => {
                return Err(
                    "Use abs(x) instead of vertical bars |x| for plain-text clarity.".to_string(),
                );
            }
            d if d.is_ascii_digit() || d == '.' => {
                let start = i;
                let mut seen_dot = d == '.';
                let mut seen_digit = d.is_ascii_digit();
                i += 1;
                while i < chars.len() {
                    let nc = chars[i];
                    if nc.is_ascii_digit() {
                        seen_digit = true;
                        i += 1;
                    } else if nc == '.' && !seen_dot {
                        seen_dot = true;
                        i += 1;
                    } else {
                        break;
                    }
                }
                if seen_digit && i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    let mut exp_idx = i + 1;
                    if exp_idx < chars.len() && (chars[exp_idx] == '+' || chars[exp_idx] == '-') {
                        exp_idx += 1;
                    }
                    if exp_idx < chars.len() && chars[exp_idx].is_ascii_digit() {
                        while exp_idx < chars.len() && chars[exp_idx].is_ascii_digit() {
                            exp_idx += 1;
                        }
                        i = exp_idx;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                let val = s
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid number literal '{}'", s))?;
                tokens.push(Token::Num(val));
            }
            a if a.is_ascii_alphabetic() || a == '_' || a == 'π' || a == 'θ' => {
                if a == 'π' {
                    tokens.push(Token::Ident("pi".to_string()));
                    i += 1;
                    continue;
                }
                if a == 'θ' {
                    tokens.push(Token::Ident("theta".to_string()));
                    i += 1;
                    continue;
                }
                let start = i;
                i += 1;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_')
                {
                    i += 1;
                }
                let ident: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(ident.to_lowercase()));
            }
            other => {
                return Err(format!("Unexpected character '{}' in expression", other));
            }
        }
    }

    if tokens.is_empty() {
        return Err("Expression cannot be empty.".to_string());
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn starts_implicit_mul(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Num(_)) | Some(Token::Ident(_)) | Some(Token::LParen)
        )
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Add(Box::new(lhs), Box::new(rhs));
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Mul(Box::new(lhs), Box::new(rhs));
                }
                Some(Token::Slash) => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Div(Box::new(lhs), Box::new(rhs));
                }
                _ if self.starts_implicit_mul() => {
                    let rhs = self.parse_power()?;
                    lhs = Expr::Mul(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Plus) => {
                self.next();
                self.parse_unary()
            }
            Some(Token::Minus) => {
                self.next();
                let inner = self.parse_unary()?;
                Ok(Expr::Neg(Box::new(inner)))
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_atom()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            let exponent = self.parse_unary()?;
            Ok(Expr::Pow(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(v)) => Ok(Expr::Const(v)),
            Some(Token::Ident(name)) => {
                if let Some(func) = MathFunc::from_ident(&name) {
                    if !matches!(self.peek(), Some(Token::LParen)) {
                        return Err(format!(
                            "Function '{}' requires parentheses, e.g. {}(x).",
                            name, name
                        ));
                    }
                    self.next();
                    let arg = self.parse_expr()?;
                    if !matches!(self.next(), Some(Token::RParen)) {
                        return Err(format!("Missing closing ')' after {}(...", name));
                    }
                    // Support standard shorthand like sin(x)^2 or even sin^2(x) when written cleanly
                    Ok(Expr::Func(func, Box::new(arg)))
                } else if name == "pi" {
                    Ok(Expr::Pi)
                } else if name == "e" {
                    Ok(Expr::EulerE)
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(Token::LParen) => {
                let inner = self.parse_expr()?;
                if !matches!(self.next(), Some(Token::RParen)) {
                    return Err("Unmatched opening parenthesis '('.".to_string());
                }
                Ok(inner)
            }
            Some(other) => Err(format!("Unexpected token {:?} while parsing.", other)),
            None => Err("Unexpected end of expression.".to_string()),
        }
    }
}

impl Expr {
    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();
        // Allow optional leading "y =" or "f(x) =" if learner types an equation LHS
        let rhs = if let Some(idx) = trimmed.find('=') {
            trimmed[idx + 1..].trim()
        } else {
            trimmed
        };
        let tokens = tokenize(rhs)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        if parser.peek().is_some() {
            return Err("Trailing unexpected characters at end of expression.".to_string());
        }
        Ok(expr)
    }

    pub fn eval(&self, env: &EvalEnv) -> f64 {
        match self {
            Self::Const(v) => *v,
            Self::Pi => PI,
            Self::EulerE => E,
            Self::Var(name) => env.get(name).copied().unwrap_or(f64::NAN),
            Self::Neg(a) => -a.eval(env),
            Self::Add(a, b) => a.eval(env) + b.eval(env),
            Self::Sub(a, b) => a.eval(env) - b.eval(env),
            Self::Mul(a, b) => a.eval(env) * b.eval(env),
            Self::Div(a, b) => {
                let denom = b.eval(env);
                if denom.abs() < 1e-14 {
                    f64::NAN
                } else {
                    a.eval(env) / denom
                }
            }
            Self::Pow(base, exp) => {
                let bv = base.eval(env);
                let ev = exp.eval(env);
                if bv < 0.0 {
                    let rounded = ev.round();
                    if (ev - rounded).abs() < 1e-12 {
                        bv.powi(rounded as i32)
                    } else {
                        f64::NAN
                    }
                } else if bv == 0.0 && ev <= 0.0 {
                    if ev == 0.0 {
                        1.0
                    } else {
                        f64::NAN
                    }
                } else {
                    bv.powf(ev)
                }
            }
            Self::Func(f, inner) => f.apply(inner.eval(env)),
        }
    }

    pub fn eval_1d(&self, var: &str, x: f64) -> f64 {
        let mut env = EvalEnv::new();
        env.insert(var.to_string(), x);
        self.eval(&env)
    }

    pub fn eval_2d(&self, x: f64, y: f64) -> f64 {
        let mut env = EvalEnv::new();
        env.insert("x".to_string(), x);
        env.insert("y".to_string(), y);
        self.eval(&env)
    }

    pub fn contains_var(&self, target: &str) -> bool {
        match self {
            Self::Const(_) | Self::Pi | Self::EulerE => false,
            Self::Var(v) => v == target,
            Self::Neg(a) | Self::Func(_, a) => a.contains_var(target),
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::Pow(a, b) => a.contains_var(target) || b.contains_var(target),
        }
    }

    /// Collects sorted, deduplicated variable identifiers appearing in the AST that are not in `allowed`.
    pub fn unknown_vars(&self, allowed: &[&str]) -> Vec<String> {
        fn walk(node: &Expr, allowed: &[&str], out: &mut std::collections::BTreeSet<String>) {
            match node {
                Expr::Const(_) | Expr::Pi | Expr::EulerE => {}
                Expr::Var(v) => {
                    if !allowed.iter().any(|&a| a == v) {
                        out.insert(v.clone());
                    }
                }
                Expr::Neg(a) | Expr::Func(_, a) => walk(a, allowed, out),
                Expr::Add(a, b)
                | Expr::Sub(a, b)
                | Expr::Mul(a, b)
                | Expr::Div(a, b)
                | Expr::Pow(a, b) => {
                    walk(a, allowed, out);
                    walk(b, allowed, out);
                }
            }
        }
        let mut set = std::collections::BTreeSet::new();
        walk(self, allowed, &mut set);
        set.into_iter().collect()
    }

    pub fn differentiate(&self, var: &str) -> Self {
        let raw = match self {
            Self::Const(_) | Self::Pi | Self::EulerE => Self::Const(0.0),
            Self::Var(v) => {
                if v == var {
                    Self::Const(1.0)
                } else {
                    Self::Const(0.0)
                }
            }
            Self::Neg(a) => Self::Neg(Box::new(a.differentiate(var))),
            Self::Add(a, b) => Self::Add(
                Box::new(a.differentiate(var)),
                Box::new(b.differentiate(var)),
            ),
            Self::Sub(a, b) => Self::Sub(
                Box::new(a.differentiate(var)),
                Box::new(b.differentiate(var)),
            ),
            Self::Mul(a, b) => {
                let da = a.differentiate(var);
                let db = b.differentiate(var);
                Self::Add(
                    Box::new(Self::Mul(Box::new(da), b.clone())),
                    Box::new(Self::Mul(a.clone(), Box::new(db))),
                )
            }
            Self::Div(a, b) => {
                let da = a.differentiate(var);
                let db = b.differentiate(var);
                let num = Self::Sub(
                    Box::new(Self::Mul(Box::new(da), b.clone())),
                    Box::new(Self::Mul(a.clone(), Box::new(db))),
                );
                let den = Self::Pow(b.clone(), Box::new(Self::Const(2.0)));
                Self::Div(Box::new(num), Box::new(den))
            }
            Self::Pow(base, exp) => {
                let base_has = base.contains_var(var);
                let exp_has = exp.contains_var(var);
                if !base_has && !exp_has {
                    Self::Const(0.0)
                } else if !exp_has {
                    let dbase = base.differentiate(var);
                    let new_exp = Self::Sub(exp.clone(), Box::new(Self::Const(1.0)));
                    Self::Mul(
                        Box::new(Self::Mul(
                            exp.clone(),
                            Box::new(Self::Pow(base.clone(), Box::new(new_exp))),
                        )),
                        Box::new(dbase),
                    )
                } else if !base_has {
                    let dexp = exp.differentiate(var);
                    let ln_base = if **base == Self::EulerE {
                        Self::Const(1.0)
                    } else {
                        Self::Func(MathFunc::Ln, base.clone())
                    };
                    Self::Mul(
                        Box::new(Self::Mul(
                            Box::new(self.clone()),
                            Box::new(ln_base),
                        )),
                        Box::new(dexp),
                    )
                } else {
                    let dbase = base.differentiate(var);
                    let dexp = exp.differentiate(var);
                    let term1 = Self::Mul(
                        Box::new(dexp),
                        Box::new(Self::Func(MathFunc::Ln, base.clone())),
                    );
                    let term2 = Self::Div(
                        Box::new(Self::Mul(exp.clone(), Box::new(dbase))),
                        base.clone(),
                    );
                    Self::Mul(
                        Box::new(self.clone()),
                        Box::new(Self::Add(Box::new(term1), Box::new(term2))),
                    )
                }
            }
            Self::Func(func, u) => {
                let du = u.differentiate(var);
                let df_du = match func {
                    MathFunc::Sin => Self::Func(MathFunc::Cos, u.clone()),
                    MathFunc::Cos => Self::Neg(Box::new(Self::Func(MathFunc::Sin, u.clone()))),
                    MathFunc::Tan => Self::Pow(
                        Box::new(Self::Func(MathFunc::Sec, u.clone())),
                        Box::new(Self::Const(2.0)),
                    ),
                    MathFunc::Sec => Self::Mul(
                        Box::new(Self::Func(MathFunc::Sec, u.clone())),
                        Box::new(Self::Func(MathFunc::Tan, u.clone())),
                    ),
                    MathFunc::Csc => Self::Neg(Box::new(Self::Mul(
                        Box::new(Self::Func(MathFunc::Csc, u.clone())),
                        Box::new(Self::Func(MathFunc::Cot, u.clone())),
                    ))),
                    MathFunc::Cot => Self::Neg(Box::new(Self::Pow(
                        Box::new(Self::Func(MathFunc::Csc, u.clone())),
                        Box::new(Self::Const(2.0)),
                    ))),
                    MathFunc::Asin => Self::Div(
                        Box::new(Self::Const(1.0)),
                        Box::new(Self::Func(
                            MathFunc::Sqrt,
                            Box::new(Self::Sub(
                                Box::new(Self::Const(1.0)),
                                Box::new(Self::Pow(u.clone(), Box::new(Self::Const(2.0)))),
                            )),
                        )),
                    ),
                    MathFunc::Acos => Self::Neg(Box::new(Self::Div(
                        Box::new(Self::Const(1.0)),
                        Box::new(Self::Func(
                            MathFunc::Sqrt,
                            Box::new(Self::Sub(
                                Box::new(Self::Const(1.0)),
                                Box::new(Self::Pow(u.clone(), Box::new(Self::Const(2.0)))),
                            )),
                        )),
                    ))),
                    MathFunc::Atan => Self::Div(
                        Box::new(Self::Const(1.0)),
                        Box::new(Self::Add(
                            Box::new(Self::Const(1.0)),
                            Box::new(Self::Pow(u.clone(), Box::new(Self::Const(2.0)))),
                        )),
                    ),
                    MathFunc::Sinh => Self::Func(MathFunc::Cosh, u.clone()),
                    MathFunc::Cosh => Self::Func(MathFunc::Sinh, u.clone()),
                    MathFunc::Tanh => Self::Sub(
                        Box::new(Self::Const(1.0)),
                        Box::new(Self::Pow(
                            Box::new(Self::Func(MathFunc::Tanh, u.clone())),
                            Box::new(Self::Const(2.0)),
                        )),
                    ),
                    MathFunc::Exp => Self::Func(MathFunc::Exp, u.clone()),
                    MathFunc::Ln => Self::Div(Box::new(Self::Const(1.0)), u.clone()),
                    MathFunc::Sqrt => Self::Div(
                        Box::new(Self::Const(1.0)),
                        Box::new(Self::Mul(
                            Box::new(Self::Const(2.0)),
                            Box::new(Self::Func(MathFunc::Sqrt, u.clone())),
                        )),
                    ),
                    MathFunc::Abs => Self::Div(
                        u.clone(),
                        Box::new(Self::Func(MathFunc::Abs, u.clone())),
                    ),
                };
                Self::Mul(Box::new(df_du), Box::new(du))
            }
        };
        raw.simplify()
    }

    pub fn simplify(&self) -> Self {
        match self {
            Self::Const(v) => {
                if v.abs() < 1e-13 {
                    Self::Const(0.0)
                } else {
                    Self::Const(*v)
                }
            }
            Self::Pi | Self::EulerE | Self::Var(_) => self.clone(),
            Self::Neg(a) => {
                let sa = a.simplify();
                match sa {
                    Self::Const(v) => {
                        if v.abs() < 1e-13 {
                            Self::Const(0.0)
                        } else {
                            Self::Const(-v)
                        }
                    }
                    Self::Neg(inner) => *inner,
                    other => Self::Neg(Box::new(other)),
                }
            }
            Self::Add(a, b) => {
                let sa = a.simplify();
                let sb = b.simplify();
                match (&sa, &sb) {
                    (Self::Const(va), Self::Const(vb)) => Self::Const(va + vb),
                    (Self::Const(va), _) if va.abs() < 1e-13 => sb,
                    (_, Self::Const(vb)) if vb.abs() < 1e-13 => sa,
                    (_, Self::Neg(inner)) => Self::Sub(Box::new(sa), inner.clone()),
                    _ => Self::Add(Box::new(sa), Box::new(sb)),
                }
            }
            Self::Sub(a, b) => {
                let sa = a.simplify();
                let sb = b.simplify();
                match (&sa, &sb) {
                    (Self::Const(va), Self::Const(vb)) => Self::Const(va - vb),
                    (_, Self::Const(vb)) if vb.abs() < 1e-13 => sa,
                    (Self::Const(va), _) if va.abs() < 1e-13 => Self::Neg(Box::new(sb)).simplify(),
                    (_, Self::Neg(inner)) => Self::Add(Box::new(sa), inner.clone()),
                    _ if sa == sb => Self::Const(0.0),
                    _ => Self::Sub(Box::new(sa), Box::new(sb)),
                }
            }
            Self::Mul(a, b) => {
                let sa = a.simplify();
                let sb = b.simplify();
                match (&sa, &sb) {
                    (Self::Const(va), _) if va.abs() < 1e-13 => Self::Const(0.0),
                    (_, Self::Const(vb)) if vb.abs() < 1e-13 => Self::Const(0.0),
                    (Self::Const(va), Self::Const(vb)) => Self::Const(va * vb),
                    (Self::Const(va), _) if (va - 1.0).abs() < 1e-13 => sb,
                    (_, Self::Const(vb)) if (vb - 1.0).abs() < 1e-13 => sa,
                    (Self::Const(va), _) if (va + 1.0).abs() < 1e-13 => {
                        Self::Neg(Box::new(sb)).simplify()
                    }
                    (_, Self::Const(vb)) if (vb + 1.0).abs() < 1e-13 => {
                        Self::Neg(Box::new(sa)).simplify()
                    }
                    _ => Self::Mul(Box::new(sa), Box::new(sb)),
                }
            }
            Self::Div(a, b) => {
                let sa = a.simplify();
                let sb = b.simplify();
                match (&sa, &sb) {
                    (Self::Const(va), _) if va.abs() < 1e-13 => Self::Const(0.0),
                    (_, Self::Const(vb)) if (vb - 1.0).abs() < 1e-13 => sa,
                    (Self::Const(va), Self::Const(vb)) if vb.abs() > 1e-12 => {
                        let q = va / vb;
                        if (q - q.round()).abs() < 1e-12 {
                            Self::Const(q.round())
                        } else {
                            Self::Div(Box::new(sa), Box::new(sb))
                        }
                    }
                    _ if sa == sb => Self::Const(1.0),
                    _ => Self::Div(Box::new(sa), Box::new(sb)),
                }
            }
            Self::Pow(base, exp) => {
                let sb = base.simplify();
                let se = exp.simplify();
                match (&sb, &se) {
                    (_, Self::Const(ve)) if ve.abs() < 1e-13 => Self::Const(1.0),
                    (_, Self::Const(ve)) if (ve - 1.0).abs() < 1e-13 => sb,
                    (Self::Const(vb), Self::Const(ve)) => {
                        let p = vb.powf(*ve);
                        if p.is_finite() && (p - p.round()).abs() < 1e-12 {
                            Self::Const(p.round())
                        } else {
                            Self::Pow(Box::new(sb), Box::new(se))
                        }
                    }
                    _ => Self::Pow(Box::new(sb), Box::new(se)),
                }
            }
            Self::Func(f, inner) => Self::Func(*f, Box::new(inner.simplify())),
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Add(_, _) | Self::Sub(_, _) => 1,
            Self::Mul(_, _) | Self::Div(_, _) => 2,
            Self::Neg(_) => 3,
            Self::Pow(_, _) => 4,
            Self::Const(_) | Self::Pi | Self::EulerE | Self::Var(_) | Self::Func(_, _) => 5,
        }
    }

    pub fn to_plain_string(&self) -> String {
        match self {
            Self::Const(v) => {
                if (*v - v.round()).abs() < 1e-10 {
                    format!("{}", v.round() as i64)
                } else {
                    format!("{:.4}", v)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                }
            }
            Self::Pi => "pi".to_string(),
            Self::EulerE => "e".to_string(),
            Self::Var(name) => name.clone(),
            Self::Neg(a) => {
                if a.precedence() < 3 {
                    format!("-({})", a.to_plain_string())
                } else {
                    format!("-{}", a.to_plain_string())
                }
            }
            Self::Add(a, b) => format!("{} + {}", a.to_plain_string(), b.to_plain_string()),
            Self::Sub(a, b) => {
                let right = if b.precedence() <= 1 {
                    format!("({})", b.to_plain_string())
                } else {
                    b.to_plain_string()
                };
                format!("{} - {}", a.to_plain_string(), right)
            }
            Self::Mul(a, b) => {
                let left = if a.precedence() < 2 {
                    format!("({})", a.to_plain_string())
                } else {
                    a.to_plain_string()
                };
                let right = if b.precedence() < 2 {
                    format!("({})", b.to_plain_string())
                } else {
                    b.to_plain_string()
                };
                format!("{} * {}", left, right)
            }
            Self::Div(a, b) => {
                let left = if a.precedence() < 2 {
                    format!("({})", a.to_plain_string())
                } else {
                    a.to_plain_string()
                };
                let right = if b.precedence() <= 2 {
                    format!("({})", b.to_plain_string())
                } else {
                    b.to_plain_string()
                };
                format!("{} / {}", left, right)
            }
            Self::Pow(base, exp) => {
                let left = if base.precedence() <= 4 {
                    format!("({})", base.to_plain_string())
                } else {
                    base.to_plain_string()
                };
                let right = if exp.precedence() < 5 {
                    format!("({})", exp.to_plain_string())
                } else {
                    exp.to_plain_string()
                };
                format!("{}^{}", left, right)
            }
            Self::Func(f, arg) => format!("{}({})", f.name(), arg.to_plain_string()),
        }
    }

    /// Formats the AST as canonical 2D LaTeX with stacked `\frac`, `\sqrt`, `\pi`, `^{...}`, and implicit multiplication.
    pub fn to_latex(&self) -> String {
        match self {
            Self::Const(v) => {
                if (*v - v.round()).abs() < 1e-10 {
                    format!("{}", v.round() as i64)
                } else {
                    format!("{:.4}", v)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                }
            }
            Self::Pi => r"\pi".to_string(),
            Self::EulerE => "e".to_string(),
            Self::Var(name) => format_var_latex(name),
            Self::Neg(a) => {
                if a.precedence() < 3 {
                    format!(r"-\left({}\right)", a.to_latex())
                } else {
                    format!("-{}", a.to_latex())
                }
            }
            Self::Add(a, b) => format!("{} + {}", a.to_latex(), b.to_latex()),
            Self::Sub(a, b) => {
                let right = if b.precedence() <= 1 {
                    format!(r"\left({}\right)", b.to_latex())
                } else {
                    b.to_latex()
                };
                format!("{} - {}", a.to_latex(), right)
            }
            Self::Mul(a, b) => {
                let left_raw = a.to_latex();
                let right_raw = b.to_latex();
                let left = if a.precedence() < 2 {
                    format!(r"\left({}\right)", left_raw)
                } else {
                    left_raw
                };
                let right = if b.precedence() < 2 || matches!(b.as_ref(), Self::Neg(_)) {
                    format!(r"\left({}\right)", right_raw)
                } else {
                    right_raw
                };
                let is_const_right = matches!(b.as_ref(), Self::Const(_));
                if is_const_right {
                    format!(r"{} \cdot {}", left, right)
                } else if needs_tex_space_after(&left, &right) {
                    format!("{} {}", left, right)
                } else {
                    format!("{}{}", left, right)
                }
            }
            Self::Div(a, b) => format!(r"\frac{{{}}}{{{}}}", a.to_latex(), b.to_latex()),
            Self::Pow(base, exp) => {
                let b_str = base.to_latex();
                let need_parens = matches!(
                    base.as_ref(),
                    Self::Add(_, _)
                        | Self::Sub(_, _)
                        | Self::Mul(_, _)
                        | Self::Div(_, _)
                        | Self::Neg(_)
                        | Self::Pow(_, _)
                        | Self::Func(_, _)
                );
                let left = if need_parens {
                    format!(r"\left({}\right)", b_str)
                } else {
                    b_str
                };
                format!("{}^{{{}}}", left, exp.to_latex())
            }
            Self::Func(f, arg) => {
                let inner = arg.to_latex();
                match f {
                    MathFunc::Sqrt => format!(r"\sqrt{{{}}}", inner),
                    MathFunc::Abs => format!(r"\left|{}\right|", inner),
                    MathFunc::Exp => {
                        if inner.contains(r"\frac") {
                            format!(r"\exp\left({}\right)", inner)
                        } else {
                            format!(r"e^{{{}}}", inner)
                        }
                    }
                    MathFunc::Ln => format!(r"\ln\left({}\right)", inner),
                    MathFunc::Asin => format!(r"\arcsin\left({}\right)", inner),
                    MathFunc::Acos => format!(r"\arccos\left({}\right)", inner),
                    MathFunc::Atan => format!(r"\arctan\left({}\right)", inner),
                    _ => format!(r"\{}\left({}\right)", f.name(), inner),
                }
            }
        }
    }
}

fn needs_tex_space_after(left: &str, right: &str) -> bool {
    let first_r = right.chars().next();
    if !matches!(first_r, Some(c) if c.is_ascii_alphanumeric()) {
        return false;
    }
    if let Some(bs) = left.rfind('\\') {
        let tail = &left[bs + 1..];
        !tail.is_empty() && tail.chars().all(|c| c.is_ascii_alphabetic())
    } else {
        false
    }
}

fn format_var_latex(name: &str) -> String {
    match name {
        "pi" => return r"\pi".to_string(),
        "theta" => return r"\theta".to_string(),
        "lambda" => return r"\lambda".to_string(),
        "omega" => return r"\omega".to_string(),
        "alpha" => return r"\alpha".to_string(),
        "beta" => return r"\beta".to_string(),
        "gamma" => return r"\gamma".to_string(),
        "delta" => return r"\delta".to_string(),
        "eps" | "epsilon" => return r"\varepsilon".to_string(),
        "mu" => return r"\mu".to_string(),
        "sigma" => return r"\sigma".to_string(),
        "phi" => return r"\phi".to_string(),
        "Delta" => return r"\Delta".to_string(),
        "Theta" => return r"\Theta".to_string(),
        "x_bar" => return r"\bar{x}".to_string(),
        "y_bar" => return r"\bar{y}".to_string(),
        "y_hat" => return r"\hat{y}".to_string(),
        "theta_hat" => return r"\hat{\theta}".to_string(),
        "lambda_hat" => return r"\hat{\lambda}".to_string(),
        "x_hat" => return r"\hat{x}".to_string(),
        _ => {}
    }
    if let Some((stem, sub)) = name.split_once('_') {
        let s_tex = format_var_latex(stem);
        let sub_tex = format_var_latex(sub);
        return format!("{}_{{{}}}", s_tex, sub_tex);
    }
    name.to_string()
}

pub fn plain_input_to_latex(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }
    if (trimmed.starts_with('<') || trimmed.starts_with('[')) && trimmed.contains(',') {
        let (p, q) = parse_vector2(trimmed)?;
        return Ok(format!(
            r"\left\langle {},\, {} \right\rangle",
            p.to_latex(),
            q.to_latex()
        ));
    }
    let ast = Expr::parse(trimmed)?;
    Ok(ast.to_latex())
}

pub fn parse_vector2(input: &str) -> Result<(Expr, Expr), String> {
    let trimmed = input.trim();
    let inner = trimmed
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .or_else(|| trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')))
        .or_else(|| trimmed.strip_prefix('(').and_then(|s| s.strip_suffix(')')))
        .unwrap_or(trimmed);

    let mut depth = 0i32;
    let mut split_at = None;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ',' if depth == 0 => {
                if split_at.is_some() {
                    return Err("Expected exactly 2 vector components <P, Q>.".to_string());
                }
                split_at = Some(idx);
            }
            _ => {}
        }
    }

    let comma_idx = split_at
        .ok_or_else(|| "Enter two comma-separated components, e.g. <2*x, 3*y>.".to_string())?;
    let left_str = inner[..comma_idx].trim();
    let right_str = inner[comma_idx + 1..].trim();
    Ok((Expr::parse(left_str)?, Expr::parse(right_str)?))
}

pub fn check_numeric_equivalence(user_input: &str, expected: f64, tol: f64) -> Result<bool, String> {
    let expr = Expr::parse(user_input)?;
    let env = EvalEnv::new();
    let val = expr.eval(&env);
    if !val.is_finite() {
        return Err("Expression must evaluate to a finite constant number (no free variables).".to_string());
    }
    let scale = 1.0_f64.max(expected.abs());
    Ok((val - expected).abs() <= tol * scale)
}

pub fn check_scalar_equivalence(
    user_input: &str,
    expected_expr_str: &str,
    vars: &[&str],
) -> Result<bool, String> {
    let user_expr = Expr::parse(user_input)?;
    let expected_expr = Expr::parse(expected_expr_str)?;
    compare_ast_on_grid(&user_expr, &expected_expr, vars)
}

pub fn check_vector2_equivalence(
    user_input: &str,
    expected_p: &str,
    expected_q: &str,
    vars: &[&str],
) -> Result<bool, String> {
    let (up, uq) = parse_vector2(user_input)?;
    let ep = Expr::parse(expected_p)?;
    let eq = Expr::parse(expected_q)?;
    let ok_p = compare_ast_on_grid(&up, &ep, vars)?;
    let ok_q = compare_ast_on_grid(&uq, &eq, vars)?;
    Ok(ok_p && ok_q)
}

fn compare_ast_on_grid(user: &Expr, expected: &Expr, vars: &[&str]) -> Result<bool, String> {
    let sample_bank: &[f64] = &[
        -2.37, -1.61, -0.83, -0.41, 0.23, 0.47, 0.79, 1.19, 1.67, 2.29, 2.83,
    ];
    let mut valid_samples = 0usize;

    for (idx, &base) in sample_bank.iter().enumerate() {
        let mut env = EvalEnv::new();
        for (v_idx, &vname) in vars.iter().enumerate() {
            let shifted = base + 0.17 * (v_idx as f64) + 0.03 * ((idx % 3) as f64);
            // Also include positive interior points for logs/radicals
            let val = if idx >= 4 { shifted.abs() + 0.19 } else { shifted };
            env.insert(vname.to_string(), val);
        }

        let ev = expected.eval(&env);
        if !ev.is_finite() || ev.abs() > 1e8 {
            continue;
        }
        let uv = user.eval(&env);
        if !uv.is_finite() {
            return Ok(false);
        }
        valid_samples += 1;
        let tol = 1e-4 * (1.0_f64.max(ev.abs()));
        if (uv - ev).abs() > tol {
            return Ok(false);
        }
    }

    if valid_samples < 3 {
        // Fallback probe on (0.35, 0.85) inside (-1, 1) for inverse trig / unit-disk domains
        for &pt in &[0.18, 0.37, 0.56, 0.74] {
            let mut env = EvalEnv::new();
            for (v_idx, &vname) in vars.iter().enumerate() {
                env.insert(vname.to_string(), pt + 0.05 * (v_idx as f64));
            }
            let ev = expected.eval(&env);
            if ev.is_finite() {
                let uv = user.eval(&env);
                if !uv.is_finite() || (uv - ev).abs() > 1e-4 * (1.0_f64.max(ev.abs())) {
                    return Ok(false);
                }
                valid_samples += 1;
            }
        }
    }

    Ok(valid_samples >= 2)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiemannMethod {
    Left,
    Right,
    Midpoint,
    Trapezoid,
    LowerDarboux,
    UpperDarboux,
}

impl RiemannMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Left => "Left Endpoint",
            Self::Right => "Right Endpoint",
            Self::Midpoint => "Midpoint Rule",
            Self::Trapezoid => "Trapezoidal Rule",
            Self::LowerDarboux => "Lower Darboux Sum L(f, P)",
            Self::UpperDarboux => "Upper Darboux Sum U(f, P)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiemannSummary {
    pub estimate: f64,
    pub exact_simpson: f64,
    pub abs_error: f64,
    /// Each tuple is (x_left, x_right, y_left_top, y_right_top)
    pub subintervals: Vec<(f64, f64, f64, f64)>,
}

pub fn riemann_sum(
    expr: &Expr,
    var: &str,
    a: f64,
    b: f64,
    n: usize,
    method: RiemannMethod,
) -> RiemannSummary {
    let steps = n.max(1);
    let dx = (b - a) / (steps as f64);
    let mut sum = 0.0;
    let mut subintervals = Vec::with_capacity(steps);

    for i in 0..steps {
        let xl = a + (i as f64) * dx;
        let xr = xl + dx;
        let xm = 0.5 * (xl + xr);
        let yl = expr.eval_1d(var, xl);
        let yr = expr.eval_1d(var, xr);
        let ym = expr.eval_1d(var, xm);

        let (h_left, h_right, contrib) = match method {
            RiemannMethod::Left => (yl, yl, yl * dx),
            RiemannMethod::Right => (yr, yr, yr * dx),
            RiemannMethod::Midpoint => (ym, ym, ym * dx),
            RiemannMethod::Trapezoid => (yl, yr, 0.5 * (yl + yr) * dx),
            RiemannMethod::LowerDarboux => {
                let mut inf = yl.min(yr).min(ym);
                for k in 1..8 {
                    let xk = xl + (k as f64 / 8.0) * dx;
                    let yk = expr.eval_1d(var, xk);
                    if yk.is_finite() {
                        inf = inf.min(yk);
                    }
                }
                (inf, inf, inf * dx)
            }
            RiemannMethod::UpperDarboux => {
                let mut sup = yl.max(yr).max(ym);
                for k in 1..8 {
                    let xk = xl + (k as f64 / 8.0) * dx;
                    let yk = expr.eval_1d(var, xk);
                    if yk.is_finite() {
                        sup = sup.max(yk);
                    }
                }
                (sup, sup, sup * dx)
            }
        };

        if contrib.is_finite() {
            sum += contrib;
            subintervals.push((xl, xr, h_left, h_right));
        }
    }

    let exact = adaptive_simpson(expr, var, a, b, 1e-8, 12);
    RiemannSummary {
        estimate: sum,
        exact_simpson: exact,
        abs_error: (sum - exact).abs(),
        subintervals,
    }
}

pub fn adaptive_simpson(
    expr: &Expr,
    var: &str,
    a: f64,
    b: f64,
    eps: f64,
    max_depth: u32,
) -> f64 {
    fn simpson_step(expr: &Expr, var: &str, a: f64, b: f64) -> (f64, f64, f64, f64, f64) {
        let c = 0.5 * (a + b);
        let fa = expr.eval_1d(var, a);
        let fb = expr.eval_1d(var, b);
        let fc = expr.eval_1d(var, c);
        let s = ((b - a) / 6.0) * (fa + 4.0 * fc + fb);
        (c, fa, fb, fc, s)
    }

    fn recurse(
        expr: &Expr,
        var: &str,
        a: f64,
        b: f64,
        eps: f64,
        whole: f64,
        fa: f64,
        fb: f64,
        fc: f64,
        depth: u32,
    ) -> f64 {
        let c = 0.5 * (a + b);
        let d = 0.5 * (a + c);
        let e = 0.5 * (c + b);
        let fd = expr.eval_1d(var, d);
        let fe = expr.eval_1d(var, e);
        if !fd.is_finite() || !fe.is_finite() {
            return f64::NAN;
        }
        let left = ((c - a) / 6.0) * (fa + 4.0 * fd + fc);
        let right = ((b - c) / 6.0) * (fc + 4.0 * fe + fb);
        if !left.is_finite() || !right.is_finite() {
            return f64::NAN;
        }
        if depth == 0 || (left + right - whole).abs() <= 15.0 * eps {
            left + right + (left + right - whole) / 15.0
        } else {
            recurse(expr, var, a, c, 0.5 * eps, left, fa, fc, fd, depth - 1)
                + recurse(expr, var, c, b, 0.5 * eps, right, fc, fb, fe, depth - 1)
        }
    }

    if !a.is_finite() || !b.is_finite() || !eps.is_finite() || eps <= 0.0 {
        return f64::NAN;
    }
    if (b - a).abs() == 0.0 {
        return if expr.eval_1d(var, a).is_finite() {
            0.0
        } else {
            f64::NAN
        };
    }

    let (_, fa, fb, fc, whole) = simpson_step(expr, var, a, b);
    if !fa.is_finite() || !fb.is_finite() || !fc.is_finite() || !whole.is_finite() {
        return f64::NAN;
    }
    recurse(expr, var, a, b, eps, whole, fa, fb, fc, max_depth)
}

/// Computes Taylor polynomial coefficients c_k = f^(k)(center) / k! for k = 0..=degree
pub fn taylor_polynomial_at(
    expr: &Expr,
    var: &str,
    center: f64,
    degree: usize,
) -> (Vec<f64>, String) {
    let mut coeffs = Vec::with_capacity(degree + 1);
    let mut deriv = expr.simplify();
    let mut factorial = 1.0_f64;
    let mut terms = Vec::new();

    for k in 0..=degree {
        if k > 0 {
            factorial *= k as f64;
            deriv = deriv.differentiate(var);
        }
        let val = deriv.eval_1d(var, center);
        let ck = if val.is_finite() { val / factorial } else { 0.0 };
        coeffs.push(ck);

        if ck.abs() > 1e-8 {
            let base_str = if center.abs() < 1e-9 {
                var.to_string()
            } else if center > 0.0 {
                format!("({} - {:.2})", var, center)
            } else {
                format!("({} + {:.2})", var, -center)
            };
            let term = match k {
                0 => format!("{:.4}", ck),
                1 => format!("{:.4}*{}", ck, base_str),
                _ => format!("{:.4}*{}^{}", ck, base_str, k),
            };
            terms.push(term);
        }
    }

    let summary = if terms.is_empty() {
        "0".to_string()
    } else {
        terms.join(" + ").replace("+ -", "- ")
    };
    (coeffs, summary)
}

#[derive(Debug, Clone)]
pub struct FieldDiagnostics2D {
    pub f_val: f64,
    pub grad: (f64, f64),
    pub grad_norm: f64,
    pub hessian: [[f64; 2]; 2],
    pub hessian_det: f64,
    pub classification: &'static str,
    pub divergence: f64,
    pub curl_z: f64,
}

pub fn evaluate_field_2d(
    scalar_f: &Expr,
    vec_p: &Expr,
    vec_q: &Expr,
    x: f64,
    y: f64,
) -> FieldDiagnostics2D {
    let fx = scalar_f.differentiate("x");
    let fy = scalar_f.differentiate("y");
    let fxx = fx.differentiate("x");
    let fxy = fx.differentiate("y");
    let fyy = fy.differentiate("y");

    let px = vec_p.differentiate("x");
    let py = vec_p.differentiate("y");
    let qx = vec_q.differentiate("x");
    let qy = vec_q.differentiate("y");

    let f_val = scalar_f.eval_2d(x, y);
    let gx = fx.eval_2d(x, y);
    let gy = fy.eval_2d(x, y);
    let grad_norm = (gx * gx + gy * gy).sqrt();

    let h00 = fxx.eval_2d(x, y);
    let h01 = fxy.eval_2d(x, y);
    let h11 = fyy.eval_2d(x, y);
    let det = h00 * h11 - h01 * h01;

    let classification = if grad_norm > 0.15 {
        "Non-stationary slope (gradient nonzero)"
    } else if det > 1e-5 && h00 > 0.0 {
        "Local Minimum (D > 0, f_xx > 0)"
    } else if det > 1e-5 && h00 < 0.0 {
        "Local Maximum (D > 0, f_xx < 0)"
    } else if det < -1e-5 {
        "Saddle Point (D < 0)"
    } else {
        "Degenerate / Flat Critical Region (D ~ 0)"
    };

    let divergence = px.eval_2d(x, y) + qy.eval_2d(x, y);
    let curl_z = qx.eval_2d(x, y) - py.eval_2d(x, y);

    FieldDiagnostics2D {
        f_val,
        grad: (gx, gy),
        grad_norm,
        hessian: [[h00, h01], [h01, h11]],
        hessian_det: det,
        classification,
        divergence,
        curl_z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_vars_identifies_unbound_identifiers() {
        let e1 = Expr::parse("2*sin(x) + cos(pi*x) - exp(-x)").expect("parse valid x expr");
        assert!(e1.unknown_vars(&["x"]).is_empty());

        let e2 = Expr::parse("sin(t) + y*x + z").expect("parse multi-var expr");
        assert_eq!(e2.unknown_vars(&["x"]), vec!["t", "y", "z"]);
        assert_eq!(e2.unknown_vars(&["x", "y"]), vec!["t", "z"]);
    }

    #[test]
    fn scientific_exponent_literals_lex_without_breaking_euler_e() {
        let env = EvalEnv::new();
        let v_neg = Expr::parse("1e-5")
            .expect("parse 1e-5")
            .eval(&env);
        assert!((v_neg - 1e-5).abs() < 1e-15, "got {v_neg}");

        let v_pos = Expr::parse("1.5e2")
            .expect("parse 1.5e2")
            .eval(&env);
        assert!((v_pos - 150.0).abs() < 1e-12, "got {v_pos}");

        let v_upper = Expr::parse("2.5E+3")
            .expect("parse 2.5E+3")
            .eval(&env);
        assert!((v_upper - 2500.0).abs() < 1e-12, "got {v_upper}");

        let v_euler = Expr::parse("2*e")
            .expect("parse 2*e")
            .eval(&env);
        assert!((v_euler - 2.0 * std::f64::consts::E).abs() < 1e-12, "got {v_euler}");

        let v_exp_x = Expr::parse("e^x")
            .expect("parse e^x")
            .eval_1d("x", 1.0);
        assert!((v_exp_x - std::f64::consts::E).abs() < 1e-12, "got {v_exp_x}");

        let v_e_minus_1 = Expr::parse("e-1")
            .expect("parse e-1")
            .eval(&env);
        assert!((v_e_minus_1 - (std::f64::consts::E - 1.0)).abs() < 1e-12, "got {v_e_minus_1}");
    }

    #[test]
    fn adaptive_simpson_propagates_nan_on_poles_and_domain_violations() {
        // Pole at interior dyadic point x = 0.5 (hit at first recursion split)
        let pole_mid = Expr::parse("1 / (x - 0.5)").unwrap();
        assert!(adaptive_simpson(&pole_mid, "x", 0.0, 2.0, 1e-8, 12).is_nan());

        // Pole at deeper dyadic point x = 0.25 on [0, 1]
        let pole_quarter = Expr::parse("1 / (x - 0.25)").unwrap();
        assert!(adaptive_simpson(&pole_quarter, "x", 0.0, 1.0, 1e-8, 12).is_nan());

        // Pole at x = 0 on [-1, 3]
        let inv_x = Expr::parse("1 / x").unwrap();
        assert!(adaptive_simpson(&inv_x, "x", -1.0, 3.0, 1e-8, 12).is_nan());

        // Domain violation sqrt(-1) at left endpoint
        let sqrt_x = Expr::parse("sqrt(x)").unwrap();
        assert!(adaptive_simpson(&sqrt_x, "x", -1.0, 2.0, 1e-8, 12).is_nan());

        // Smooth integrand still integrates to machine accuracy
        let poly = Expr::parse("x^2").unwrap();
        let val = adaptive_simpson(&poly, "x", 0.0, 3.0, 1e-10, 12);
        assert!((val - 9.0).abs() < 1e-10, "got {val}");
    }
}
