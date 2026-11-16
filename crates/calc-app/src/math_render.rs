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

use egui::{
    self, Align2, Color32, FontFamily, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2,
};

/// Zero-dependency 2D LaTeX mathematical expression renderer on `egui::Painter`.
///
/// Lays out fractions with horizontal bars, tall integral / sigma / pi / limit operators,
/// square roots with overline vinculums, matrices and piecewise cases, scripts, accents,
/// scaled delimiters, and crisp vector glyphs so every equation renders without missing-glyph boxes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BigOpKind {
    Int,
    Iint,
    Oint,
    Sum,
    Prod,
    Lim,
    Argmax,
    Max,
    Min,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimKind {
    ParenL,
    ParenR,
    BracketL,
    BracketR,
    BraceL,
    BraceR,
    Pipe,
    DoublePipe,
    AngleL,
    AngleR,
    Dot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentKind {
    Hat,
    Bar,
    Vec,
    Dot,
    Ddot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixStyle {
    Bracket,
    Paren,
    Vbar,
    Cases,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphKind {
    Pi,
    Theta,
    Lambda,
    Alpha,
    Beta,
    Gamma,
    Delta,
    DeltaCap,
    Varepsilon,
    Mu,
    Sigma,
    Phi,
    Omega,
    OmegaCap,
    Infty,
    Forall,
    Exists,
    InSet,
    SubsetEq,
    MathbbR,
    Nabla,
    Partial,
    Ell,
    CalL,
    To,
    Mapsto,
    Rightarrow,
    Implies,
    Iff,
    Le,
    Ge,
    Ne,
    Approx,
    Equiv,
    Pm,
    Cdot,
    Times,
    Circ,
    Neg,
    Sim,
    AngleL,
    AngleR,
    DoublePipe,
}

#[derive(Debug, Clone)]
pub enum MathNode {
    Row(Vec<MathNode>),
    Run {
        text: String,
        italic: bool,
        bold: bool,
        is_op: bool,
    },
    Space(f32),
    Frac {
        num: Box<MathNode>,
        den: Box<MathNode>,
    },
    Sqrt {
        index: Option<Box<MathNode>>,
        inner: Box<MathNode>,
    },
    SupSub {
        base: Box<MathNode>,
        sub: Option<Box<MathNode>>,
        sup: Option<Box<MathNode>>,
    },
    BigOp {
        kind: BigOpKind,
        sub: Option<Box<MathNode>>,
        sup: Option<Box<MathNode>>,
    },
    Delim {
        left: DelimKind,
        right: DelimKind,
        inner: Box<MathNode>,
    },
    Accent {
        kind: AccentKind,
        inner: Box<MathNode>,
    },
    Matrix {
        style: MatrixStyle,
        rows: Vec<Vec<MathNode>>,
    },
    Glyph(GlyphKind),
}

// =========================================================================
// UNICODE -> LATEX CONVERTER (for Notation Studio & Lesson Identities)
// =========================================================================

pub fn unicode_math_to_latex(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 32);
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        // Combined character checks (e.g. θ + combining circumflex, x + combining dot/bar)
        if i + 1 < chars.len() && chars[i + 1] == '\u{0302}' {
            if ch == 'θ' {
                out.push_str(r"\hat{\theta}");
            } else {
                out.push_str(&format!(r"\hat{{{}}}", ch));
            }
            i += 2;
            continue;
        }
        if i + 1 < chars.len() && chars[i + 1] == '\u{0304}' {
            out.push_str(&format!(r"\bar{{{}}}", ch));
            i += 2;
            continue;
        }
        if i + 1 < chars.len() && chars[i + 1] == '\u{0307}' {
            out.push_str(&format!(r"\dot{{{}}}", ch));
            i += 2;
            continue;
        }

        match ch {
            '∀' => out.push_str(r"\forall "),
            '∃' => out.push_str(r"\exists "),
            '∈' => out.push_str(r"\in "),
            '⊆' => out.push_str(r"\subseteq "),
            'ℝ' => out.push_str(r"\mathbb{R}"),
            '↦' => out.push_str(r"\mapsto "),
            '→' => out.push_str(r"\to "),
            '⇒' => out.push_str(r"\Rightarrow "),
            '≡' => out.push_str(r"\equiv "),
            '∘' => out.push_str(r"\circ "),
            '∑' => out.push_str(r"\sum"),
            '∏' => out.push_str(r"\prod"),
            '∫' => out.push_str(r"\int"),
            '∮' => out.push_str(r"\oint"),
            '∬' => out.push_str(r"\iint"),
            '∇' => out.push_str(r"\nabla "),
            '∂' => out.push_str(r"\partial "),
            'Δ' => out.push_str(r"\Delta "),
            'α' => out.push_str(r"\alpha "),
            'ε' => out.push_str(r"\varepsilon "),
            'δ' => out.push_str(r"\delta "),
            'θ' => out.push_str(r"\theta "),
            'λ' => out.push_str(r"\lambda "),
            'μ' => out.push_str(r"\mu "),
            'σ' => out.push_str(r"\sigma "),
            'φ' => out.push_str(r"\phi "),
            'ω' => out.push_str(r"\omega "),
            'Ω' => out.push_str(r"\Omega "),
            'π' => out.push_str(r"\pi "),
            '‖' => out.push_str(r"\|"),
            '⟨' => out.push_str(r"\langle "),
            '⟩' => out.push_str(r"\rangle "),
            'ᵀ' => out.push_str(r"^{T}"),
            '²' => out.push_str(r"^{2}"),
            '³' => out.push_str(r"^{3}"),
            '₀' => out.push_str(r"_{0}"),
            '₁' => out.push_str(r"_{1}"),
            '₂' => out.push_str(r"_{2}"),
            '√' => out.push_str(r"\sqrt"),
            '·' => out.push_str(r"\cdot "),
            '×' => out.push_str(r"\times "),
            '≤' => out.push_str(r"\le "),
            '≥' => out.push_str(r"\ge "),
            '≠' => out.push_str(r"\ne "),
            '≈' => out.push_str(r"\approx "),
            '±' => out.push_str(r"\pm "),
            'ẋ' => out.push_str(r"\dot{x}"),
            '∞' => out.push_str(r"\infty "),
            '¬' => out.push_str(r"\neg "),
            ' ' => {
                // Collapse long ASCII padding runs in notation strings into clean LaTeX quad spacing
                let mut sp = 0;
                while i + sp < chars.len() && chars[i + sp] == ' ' {
                    sp += 1;
                }
                if sp >= 3 {
                    out.push_str(r"\quad ");
                } else {
                    out.push(' ');
                }
                i += sp;
                continue;
            }
            _ => {
                // Multi-character ASCII keywords inside plain notation strings
                let rem: String = chars[i..].iter().collect();
                let word_boundary_before =
                    i == 0 || (!chars[i - 1].is_ascii_alphanumeric() && chars[i - 1] != '\\');
                if rem.starts_with("d_Omega") && word_boundary_before {
                    out.push_str(r"\partial \Omega ");
                    i += 7;
                    continue;
                } else if rem.starts_with("Omega") && word_boundary_before {
                    out.push_str(r"\Omega ");
                    i += 5;
                    continue;
                } else if rem.starts_with("integral") && word_boundary_before {
                    out.push_str(r"\int");
                    i += 8;
                    continue;
                } else if rem.starts_with("oint") && word_boundary_before {
                    out.push_str(r"\oint");
                    i += 4;
                    continue;
                } else if rem.starts_with("argmax") && word_boundary_before {
                    out.push_str(r"\operatorname{argmax}");
                    i += 6;
                    continue;
                } else if rem.starts_with("grad") && word_boundary_before {
                    out.push_str(r"\operatorname{grad}");
                    i += 4;
                    continue;
                } else if rem.starts_with("div")
                    && word_boundary_before
                    && rem
                        .chars()
                        .nth(3)
                        .is_none_or(|c| !c.is_ascii_alphanumeric())
                {
                    out.push_str(r"\operatorname{div}");
                    i += 3;
                    continue;
                } else if rem.starts_with("curl") && word_boundary_before {
                    out.push_str(r"\operatorname{curl}");
                    i += 4;
                    continue;
                } else if rem.starts_with("s.t.") {
                    out.push_str(r"\text{s.t.}");
                    i += 4;
                    continue;
                } else if rem.starts_with("vs") && i > 0 && chars[i - 1] == ' ' {
                    out.push_str(r"\text{vs}");
                    i += 2;
                    continue;
                } else if rem.starts_with("<=>") {
                    out.push_str(r"\iff ");
                    i += 3;
                    continue;
                } else if rem.starts_with("<=") {
                    out.push_str(r"\le ");
                    i += 2;
                    continue;
                } else if rem.starts_with(">=") {
                    out.push_str(r"\ge ");
                    i += 2;
                    continue;
                } else if rem.starts_with("!=") {
                    out.push_str(r"\ne ");
                    i += 2;
                    continue;
                } else if rem.starts_with("->") {
                    out.push_str(r"\to ");
                    i += 2;
                    continue;
                } else if rem.starts_with("=>") {
                    out.push_str(r"\implies ");
                    i += 2;
                    continue;
                }
                out.push(ch);
            }
        }
        i += 1;
    }
    out
}

/// Replaces Unicode mathematical glyphs missing from egui's default proportional/monospace fonts
/// with crisp readable ASCII equivalents for standard `ui.label` / button / heading controls.
pub fn ui_safe_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if i + 1 < chars.len() && chars[i + 1] == '\u{0302}' {
            match ch {
                'θ' => out.push_str("theta_hat"),
                'σ' => out.push_str("sigma_hat"),
                'β' => out.push_str("beta_hat"),
                _ => {
                    out.push(ch);
                    out.push_str("_hat");
                }
            }
            i += 2;
            continue;
        }
        if i + 1 < chars.len() && chars[i + 1] == '\u{0304}' {
            out.push(ch);
            out.push_str("_bar");
            i += 2;
            continue;
        }
        if i + 1 < chars.len() && chars[i + 1] == '\u{0307}' {
            out.push(ch);
            out.push_str("_dot");
            i += 2;
            continue;
        }
        let nxt = chars.get(i + 1).copied();
        let needs_sp = nxt.is_some_and(|c| !c.is_whitespace() && !matches!(c, ')' | ',' | '.' | ';'));
        match ch {
            '∀' => {
                out.push_str("for all");
                if needs_sp {
                    out.push(' ');
                }
            }
            '∃' => {
                out.push_str("exists");
                if needs_sp {
                    out.push(' ');
                }
            }
            '⇒' => out.push_str("=>"),
            '⇔' => out.push_str("<=>"),
            '→' => out.push_str("->"),
            '↦' => out.push_str("|->"),
            '∈' => out.push_str("in"),
            '⊆' => out.push_str("subset"),
            'ℝ' => out.push('R'),
            '∮' => out.push_str("oint"),
            '∬' => out.push_str("double_integral"),
            '∫' => out.push_str("integral"),
            '∇' => {
                out.push_str("del");
                if nxt.is_some_and(|c| c.is_ascii_alphabetic()) {
                    out.push(' ');
                }
            }
            '∑' => out.push('Σ'),
            '∏' => out.push('Π'),
            '∘' => out.push('o'),
            '≡' => out.push_str("==="),
            'ᵀ' => out.push_str("^T"),
            '‖' => out.push_str("||"),
            '⟨' => out.push('<'),
            '⟩' => out.push('>'),
            '³' => out.push_str("^3"),
            'ⁿ' => out.push_str("^n"),
            'ᵐ' => out.push_str("^m"),
            '₀' => out.push_str("_0"),
            '₁' => out.push_str("_1"),
            '₂' => out.push_str("_2"),
            'ẋ' => out.push_str("x_dot"),
            'ẍ' => out.push_str("x_ddot"),
            '≤' => out.push_str("<="),
            '≥' => out.push_str(">="),
            '≠' => out.push_str("!="),
            '¬' => out.push_str("not "),
            '•' => out.push('*'),
            other => out.push(other),
        }
        i += 1;
    }
    out
}

// =========================================================================
// LATEX TOKENIZER & PARSER
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Cmd(String),
    Text(char),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Caret,
    Underscore,
    Amp,
    Newline,
}

fn tokenize_latex(input: &str) -> Vec<Tok> {
    let mut toks = Vec::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' => {
                if i + 1 >= chars.len() {
                    i += 1;
                    continue;
                }
                let nxt = chars[i + 1];
                if nxt == '\\' {
                    toks.push(Tok::Newline);
                    i += 2;
                } else if !nxt.is_ascii_alphabetic() {
                    let mut cmd = String::new();
                    cmd.push(nxt);
                    toks.push(Tok::Cmd(cmd));
                    i += 2;
                } else {
                    let mut j = i + 1;
                    while j < chars.len() && chars[j].is_ascii_alphabetic() {
                        j += 1;
                    }
                    let cmd: String = chars[i + 1..j].iter().collect();
                    toks.push(Tok::Cmd(cmd));
                    // Consume single trailing whitespace after alpha command (TeX rule)
                    if j < chars.len() && chars[j] == ' ' {
                        j += 1;
                    }
                    i = j;
                }
            }
            '{' => {
                toks.push(Tok::LBrace);
                i += 1;
            }
            '}' => {
                toks.push(Tok::RBrace);
                i += 1;
            }
            '[' => {
                toks.push(Tok::LBracket);
                i += 1;
            }
            ']' => {
                toks.push(Tok::RBracket);
                i += 1;
            }
            '^' => {
                toks.push(Tok::Caret);
                i += 1;
            }
            '_' => {
                toks.push(Tok::Underscore);
                i += 1;
            }
            '&' => {
                toks.push(Tok::Amp);
                i += 1;
            }
            c if c.is_whitespace() => {
                i += 1;
            }
            other => {
                toks.push(Tok::Text(other));
                i += 1;
            }
        }
    }
    toks
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            toks: tokenize_latex(input),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        if self.pos < self.toks.len() {
            let t = self.toks[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn parse_row(&mut self, stop_rbrace: bool, stop_right: bool, stop_cell: bool) -> MathNode {
        let mut nodes = Vec::new();
        while let Some(tok) = self.peek() {
            match tok {
                Tok::RBrace if stop_rbrace => {
                    self.next();
                    break;
                }
                Tok::Cmd(c) if stop_right && c == "right" => {
                    break;
                }
                Tok::Cmd(c) if stop_cell && c == "end" => {
                    break;
                }
                Tok::Amp | Tok::Newline if stop_cell => {
                    break;
                }
                Tok::Newline => {
                    self.next();
                    nodes.push(MathNode::Space(0.5));
                }
                _ => {
                    if let Some(item) = self.parse_scripted() {
                        nodes.push(item);
                    } else {
                        break;
                    }
                }
            }
        }
        if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            MathNode::Row(nodes)
        }
    }

    fn parse_group_or_atom(&mut self) -> MathNode {
        match self.peek() {
            Some(Tok::LBrace) => {
                self.next();
                self.parse_row(true, false, false)
            }
            _ => self.parse_atom().unwrap_or(MathNode::Row(Vec::new())),
        }
    }

    fn parse_bracket_group(&mut self) -> Option<MathNode> {
        if matches!(self.peek(), Some(Tok::LBracket)) {
            self.next();
            let mut nodes = Vec::new();
            while let Some(tok) = self.peek() {
                if matches!(tok, Tok::RBracket) {
                    self.next();
                    break;
                }
                if let Some(it) = self.parse_scripted() {
                    nodes.push(it);
                } else {
                    break;
                }
            }
            Some(if nodes.len() == 1 {
                nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(nodes)
            })
        } else {
            None
        }
    }

    fn parse_plain_text_arg(&mut self) -> String {
        let mut s = String::new();
        if matches!(self.peek(), Some(Tok::LBrace)) {
            self.next();
            let mut depth = 1usize;
            while let Some(t) = self.next() {
                match t {
                    Tok::LBrace => depth += 1,
                    Tok::RBrace => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    Tok::Text(c) => s.push(c),
                    Tok::Cmd(c) if c == "," => s.push(' '),
                    Tok::Cmd(c) if c == " " => s.push(' '),
                    Tok::Cmd(c) if c == "quad" => s.push_str("  "),
                    Tok::Cmd(c) => s.push_str(&c),
                    _ => {}
                }
            }
        } else if let Some(Tok::Text(c)) = self.next() {
            s.push(c);
        }
        s
    }

    fn parse_delim_token(&mut self) -> DelimKind {
        match self.next() {
            Some(Tok::Text('(')) => DelimKind::ParenL,
            Some(Tok::Text(')')) => DelimKind::ParenR,
            Some(Tok::LBracket) | Some(Tok::Text('[')) => DelimKind::BracketL,
            Some(Tok::RBracket) | Some(Tok::Text(']')) => DelimKind::BracketR,
            Some(Tok::Text('|')) => DelimKind::Pipe,
            Some(Tok::Text('.')) => DelimKind::Dot,
            Some(Tok::Cmd(c)) => match c.as_str() {
                "{" => DelimKind::BraceL,
                "}" => DelimKind::BraceR,
                "|" => DelimKind::DoublePipe,
                "langle" => DelimKind::AngleL,
                "rangle" => DelimKind::AngleR,
                _ => DelimKind::Dot,
            },
            _ => DelimKind::Dot,
        }
    }

    fn parse_scripted(&mut self) -> Option<MathNode> {
        let atom = self.parse_atom()?;
        let mut sub: Option<Box<MathNode>> = None;
        let mut sup: Option<Box<MathNode>> = None;
        loop {
            match self.peek() {
                Some(Tok::Underscore) => {
                    self.next();
                    sub = Some(Box::new(self.parse_group_or_atom()));
                }
                Some(Tok::Caret) => {
                    self.next();
                    sup = Some(Box::new(self.parse_group_or_atom()));
                }
                _ => break,
            }
        }
        if let MathNode::BigOp { kind, .. } = atom {
            return Some(MathNode::BigOp { kind, sub, sup });
        }
        if sub.is_some() || sup.is_some() {
            Some(MathNode::SupSub {
                base: Box::new(atom),
                sub,
                sup,
            })
        } else {
            Some(atom)
        }
    }

    fn parse_atom(&mut self) -> Option<MathNode> {
        let tok = self.next()?;
        match tok {
            Tok::LBrace => Some(self.parse_row(true, false, false)),
            Tok::RBrace => Some(MathNode::Row(Vec::new())),
            Tok::LBracket => Some(MathNode::Run {
                text: "[".to_string(),
                italic: false,
                bold: false,
                is_op: false,
            }),
            Tok::RBracket => Some(MathNode::Run {
                text: "]".to_string(),
                italic: false,
                bold: false,
                is_op: false,
            }),
            Tok::Caret | Tok::Underscore | Tok::Amp | Tok::Newline => Some(MathNode::Row(Vec::new())),
            Tok::Text(ch) => Some(self.text_atom(ch)),
            Tok::Cmd(cmd) => Some(self.command_atom(&cmd)),
        }
    }

    fn text_atom(&self, ch: char) -> MathNode {
        match ch {
            '<' => MathNode::Run {
                text: "<".to_string(),
                italic: false,
                bold: false,
                is_op: true,
            },
            '>' => MathNode::Run {
                text: ">".to_string(),
                italic: false,
                bold: false,
                is_op: true,
            },
            '+' | '-' | '=' => MathNode::Run {
                text: ch.to_string(),
                italic: false,
                bold: false,
                is_op: true,
            },
            c if c.is_ascii_alphabetic() => MathNode::Run {
                text: c.to_string(),
                italic: true,
                bold: false,
                is_op: false,
            },
            other => MathNode::Run {
                text: other.to_string(),
                italic: false,
                bold: false,
                is_op: false,
            },
        }
    }

    fn command_atom(&mut self, cmd: &str) -> MathNode {
        match cmd {
            "frac" | "dfrac" | "tfrac" => {
                let num = self.parse_group_or_atom();
                let den = self.parse_group_or_atom();
                MathNode::Frac {
                    num: Box::new(num),
                    den: Box::new(den),
                }
            }
            "sqrt" => {
                let index = self.parse_bracket_group().map(Box::new);
                let inner = self.parse_group_or_atom();
                MathNode::Sqrt {
                    index,
                    inner: Box::new(inner),
                }
            }
            "left" => {
                let left = self.parse_delim_token();
                let inner = self.parse_row(false, true, false);
                let right = if matches!(self.peek(), Some(Tok::Cmd(c)) if c == "right") {
                    self.next();
                    self.parse_delim_token()
                } else {
                    DelimKind::Dot
                };
                MathNode::Delim {
                    left,
                    right,
                    inner: Box::new(inner),
                }
            }
            "begin" => {
                let env = self.parse_plain_text_arg();
                let style = match env.as_str() {
                    "bmatrix" | "Bmatrix" => MatrixStyle::Bracket,
                    "pmatrix" => MatrixStyle::Paren,
                    "vmatrix" => MatrixStyle::Vbar,
                    "cases" => MatrixStyle::Cases,
                    _ => MatrixStyle::Plain,
                };
                let mut rows = Vec::new();
                let mut cur_row = Vec::new();
                loop {
                    let cell = self.parse_row(false, false, true);
                    cur_row.push(cell);
                    match self.peek() {
                        Some(Tok::Amp) => {
                            self.next();
                        }
                        Some(Tok::Newline) => {
                            self.next();
                            rows.push(cur_row);
                            cur_row = Vec::new();
                        }
                        Some(Tok::Cmd(c)) if c == "end" => {
                            self.next();
                            let _ = self.parse_plain_text_arg();
                            if !cur_row.is_empty() {
                                rows.push(cur_row);
                            }
                            break;
                        }
                        None => {
                            if !cur_row.is_empty() {
                                rows.push(cur_row);
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                MathNode::Matrix { style, rows }
            }
            "text" | "mathrm" | "operatorname" => {
                let s = self.parse_plain_text_arg();
                if s == "argmax" {
                    return MathNode::BigOp {
                        kind: BigOpKind::Argmax,
                        sub: None,
                        sup: None,
                    };
                }
                MathNode::Run {
                    text: s,
                    italic: false,
                    bold: false,
                    is_op: false,
                }
            }
            "mathbf" => {
                let s = self.parse_plain_text_arg();
                MathNode::Run {
                    text: s,
                    italic: false,
                    bold: true,
                    is_op: false,
                }
            }
            "mathbb" => {
                let s = self.parse_plain_text_arg();
                if s == "R" {
                    MathNode::Glyph(GlyphKind::MathbbR)
                } else {
                    MathNode::Run {
                        text: s,
                        italic: false,
                        bold: true,
                        is_op: false,
                    }
                }
            }
            "mathcal" => {
                let s = self.parse_plain_text_arg();
                if s == "L" {
                    MathNode::Glyph(GlyphKind::CalL)
                } else {
                    MathNode::Run {
                        text: s,
                        italic: true,
                        bold: true,
                        is_op: false,
                    }
                }
            }
            "hat" => MathNode::Accent {
                kind: AccentKind::Hat,
                inner: Box::new(self.parse_group_or_atom()),
            },
            "bar" | "overline" => MathNode::Accent {
                kind: AccentKind::Bar,
                inner: Box::new(self.parse_group_or_atom()),
            },
            "vec" => MathNode::Accent {
                kind: AccentKind::Vec,
                inner: Box::new(self.parse_group_or_atom()),
            },
            "dot" => MathNode::Accent {
                kind: AccentKind::Dot,
                inner: Box::new(self.parse_group_or_atom()),
            },
            "ddot" => MathNode::Accent {
                kind: AccentKind::Ddot,
                inner: Box::new(self.parse_group_or_atom()),
            },
            "int" => MathNode::BigOp {
                kind: BigOpKind::Int,
                sub: None,
                sup: None,
            },
            "iint" => MathNode::BigOp {
                kind: BigOpKind::Iint,
                sub: None,
                sup: None,
            },
            "oint" => MathNode::BigOp {
                kind: BigOpKind::Oint,
                sub: None,
                sup: None,
            },
            "sum" => MathNode::BigOp {
                kind: BigOpKind::Sum,
                sub: None,
                sup: None,
            },
            "prod" => MathNode::BigOp {
                kind: BigOpKind::Prod,
                sub: None,
                sup: None,
            },
            "lim" => MathNode::BigOp {
                kind: BigOpKind::Lim,
                sub: None,
                sup: None,
            },
            "max" => MathNode::BigOp {
                kind: BigOpKind::Max,
                sub: None,
                sup: None,
            },
            "min" => MathNode::BigOp {
                kind: BigOpKind::Min,
                sub: None,
                sup: None,
            },
            "sin" | "cos" | "tan" | "sec" | "csc" | "cot" | "arcsin" | "arccos" | "arctan"
            | "sinh" | "cosh" | "tanh" | "ln" | "log" | "exp" | "det" | "tr" | "deg" => {
                MathNode::Run {
                    text: cmd.to_string(),
                    italic: false,
                    bold: false,
                    is_op: false,
                }
            }
            "pi" => MathNode::Glyph(GlyphKind::Pi),
            "theta" => MathNode::Glyph(GlyphKind::Theta),
            "lambda" => MathNode::Glyph(GlyphKind::Lambda),
            "alpha" => MathNode::Glyph(GlyphKind::Alpha),
            "beta" => MathNode::Glyph(GlyphKind::Beta),
            "gamma" => MathNode::Glyph(GlyphKind::Gamma),
            "delta" => MathNode::Glyph(GlyphKind::Delta),
            "Delta" => MathNode::Glyph(GlyphKind::DeltaCap),
            "varepsilon" | "epsilon" => MathNode::Glyph(GlyphKind::Varepsilon),
            "mu" => MathNode::Glyph(GlyphKind::Mu),
            "sigma" => MathNode::Glyph(GlyphKind::Sigma),
            "phi" => MathNode::Glyph(GlyphKind::Phi),
            "omega" => MathNode::Glyph(GlyphKind::Omega),
            "Omega" => MathNode::Glyph(GlyphKind::OmegaCap),
            "infty" => MathNode::Glyph(GlyphKind::Infty),
            "forall" => MathNode::Glyph(GlyphKind::Forall),
            "exists" => MathNode::Glyph(GlyphKind::Exists),
            "in" => MathNode::Glyph(GlyphKind::InSet),
            "subseteq" => MathNode::Glyph(GlyphKind::SubsetEq),
            "nabla" => MathNode::Glyph(GlyphKind::Nabla),
            "partial" => MathNode::Glyph(GlyphKind::Partial),
            "ell" => MathNode::Glyph(GlyphKind::Ell),
            "to" | "rightarrow" => MathNode::Glyph(GlyphKind::To),
            "mapsto" => MathNode::Glyph(GlyphKind::Mapsto),
            "Rightarrow" => MathNode::Glyph(GlyphKind::Rightarrow),
            "implies" => MathNode::Glyph(GlyphKind::Implies),
            "iff" => MathNode::Glyph(GlyphKind::Iff),
            "le" | "leq" => MathNode::Glyph(GlyphKind::Le),
            "ge" | "geq" => MathNode::Glyph(GlyphKind::Ge),
            "ne" | "neq" => MathNode::Glyph(GlyphKind::Ne),
            "approx" => MathNode::Glyph(GlyphKind::Approx),
            "equiv" => MathNode::Glyph(GlyphKind::Equiv),
            "pm" => MathNode::Glyph(GlyphKind::Pm),
            "cdot" => MathNode::Glyph(GlyphKind::Cdot),
            "times" => MathNode::Glyph(GlyphKind::Times),
            "circ" => MathNode::Glyph(GlyphKind::Circ),
            "neg" => MathNode::Glyph(GlyphKind::Neg),
            "sim" => MathNode::Glyph(GlyphKind::Sim),
            "langle" => MathNode::Glyph(GlyphKind::AngleL),
            "rangle" => MathNode::Glyph(GlyphKind::AngleR),
            "|" => MathNode::Glyph(GlyphKind::DoublePipe),
            "{" => MathNode::Run {
                text: "{".to_string(),
                italic: false,
                bold: false,
                is_op: false,
            },
            "}" => MathNode::Run {
                text: "}".to_string(),
                italic: false,
                bold: false,
                is_op: false,
            },
            "," => MathNode::Space(0.32),
            ";" | ":" => MathNode::Space(0.32),
            "!" => MathNode::Space(-0.08),
            " " => MathNode::Space(0.28),
            "quad" => MathNode::Space(0.9),
            "qquad" => MathNode::Space(1.6),
            "Big" | "big" | "left." | "right." => MathNode::Row(Vec::new()),
            other => MathNode::Run {
                text: other.to_string(),
                italic: false,
                bold: false,
                is_op: false,
            },
        }
    }
}

pub fn parse_latex(input: &str) -> MathNode {
    Parser::new(input).parse_row(false, false, false)
}

// =========================================================================
// BOX LAYOUT MEASUREMENT & PAINTER RENDERING
// =========================================================================

#[derive(Debug, Clone)]
enum DrawnItem {
    Row(Vec<(f32, f32, MeasuredBox)>),
    Text {
        text: String,
        font_size: f32,
        italic: bool,
        bold: bool,
    },
    Frac {
        num: Box<MeasuredBox>,
        den: Box<MeasuredBox>,
        num_dx: f32,
        num_dy: f32,
        den_dx: f32,
        den_dy: f32,
        bar_y: f32,
        bar_w: f32,
        stroke_w: f32,
    },
    Sqrt {
        index: Option<Box<MeasuredBox>>,
        idx_dx: f32,
        idx_dy: f32,
        inner: Box<MeasuredBox>,
        inner_dx: f32,
        radical_x0: f32,
        radical_w: f32,
        top_y: f32,
        bot_y: f32,
        stroke_w: f32,
    },
    BigOp {
        kind: BigOpKind,
        font_size: f32,
        op_w: f32,
        op_ascent: f32,
        op_descent: f32,
        op_dx: f32,
        op_dy: f32,
        sub: Option<(f32, f32, Box<MeasuredBox>)>,
        sup: Option<(f32, f32, Box<MeasuredBox>)>,
    },
    Delim {
        left: DelimKind,
        right: DelimKind,
        lw: f32,
        rw: f32,
        top_y: f32,
        bot_y: f32,
        inner_dx: f32,
        inner: Box<MeasuredBox>,
        stroke_w: f32,
    },
    Accent {
        kind: AccentKind,
        inner: Box<MeasuredBox>,
        inner_dy: f32,
        accent_y: f32,
        w: f32,
        stroke_w: f32,
    },
    Matrix {
        style: MatrixStyle,
        cells: Vec<Vec<(f32, f32, MeasuredBox)>>,
        top_y: f32,
        bot_y: f32,
        total_w: f32,
        bracket_w: f32,
        stroke_w: f32,
    },
    Glyph {
        kind: GlyphKind,
        font_size: f32,
    },
    Empty,
}

#[derive(Debug, Clone)]
struct MeasuredBox {
    width: f32,
    ascent: f32,
    descent: f32,
    item: DrawnItem,
}

impl MeasuredBox {
    fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

fn measure_text(ui: &Ui, text: &str, font_size: f32, bold: bool) -> Vec2 {
    let family = FontFamily::Proportional;
    let font = FontId::new(font_size, family);
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            text.to_string(),
            font,
            if bold { Color32::WHITE } else { Color32::WHITE },
        )
    });
    galley.size()
}

fn measure_node(ui: &Ui, node: &MathNode, font_size: f32) -> MeasuredBox {
    let axis = font_size * 0.28;
    let stroke_w = (font_size * 0.068).clamp(1.0, 2.2);
    match node {
        MathNode::Space(em) => MeasuredBox {
            width: (*em * font_size * 0.45).max(0.0),
            ascent: font_size * 0.5,
            descent: font_size * 0.15,
            item: DrawnItem::Empty,
        },
        MathNode::Run {
            text,
            italic,
            bold,
            is_op,
        } => {
            if text.is_empty() {
                return MeasuredBox {
                    width: 0.0,
                    ascent: font_size * 0.65,
                    descent: font_size * 0.18,
                    item: DrawnItem::Empty,
                };
            }
            let pad = if *is_op { font_size * 0.18 } else { 0.0 };
            let sz = measure_text(ui, text, font_size, *bold);
            let w = sz.x + if *bold { font_size * 0.04 } else { 0.0 } + 2.0 * pad;
            let asc = sz.y * 0.78;
            let desc = sz.y * 0.22;
            MeasuredBox {
                width: w,
                ascent: asc,
                descent: desc,
                item: DrawnItem::Text {
                    text: text.clone(),
                    font_size,
                    italic: *italic,
                    bold: *bold,
                },
            }
        }
        MathNode::Glyph(kind) => {
            let w = match kind {
                GlyphKind::To | GlyphKind::Mapsto | GlyphKind::Rightarrow => font_size * 0.92,
                GlyphKind::Implies | GlyphKind::Iff => font_size * 1.15,
                GlyphKind::Le | GlyphKind::Ge | GlyphKind::Ne | GlyphKind::Approx | GlyphKind::Equiv => {
                    font_size * 0.78
                }
                GlyphKind::InSet | GlyphKind::SubsetEq | GlyphKind::Sim => font_size * 0.74,
                GlyphKind::Cdot | GlyphKind::Circ => font_size * 0.36,
                GlyphKind::Pm | GlyphKind::Times | GlyphKind::Neg => font_size * 0.68,
                GlyphKind::AngleL | GlyphKind::AngleR => font_size * 0.42,
                GlyphKind::DoublePipe => font_size * 0.36,
                GlyphKind::Infty => font_size * 0.92,
                GlyphKind::MathbbR => font_size * 0.72,
                GlyphKind::Forall | GlyphKind::Exists | GlyphKind::Nabla | GlyphKind::DeltaCap | GlyphKind::OmegaCap => {
                    font_size * 0.74
                }
                GlyphKind::CalL => font_size * 0.72,
                GlyphKind::Varepsilon | GlyphKind::Pi | GlyphKind::Mu | GlyphKind::Sigma | GlyphKind::Omega | GlyphKind::Phi => {
                    font_size * 0.60
                }
                _ => font_size * 0.56,
            };
            MeasuredBox {
                width: w,
                ascent: font_size * 0.74,
                descent: font_size * 0.20,
                item: DrawnItem::Glyph {
                    kind: *kind,
                    font_size,
                },
            }
        }
        MathNode::Row(children) => {
            // Merge consecutive italic/plain Run atoms when appropriate? Keeping per-node spacing clean:
            let mut items = Vec::with_capacity(children.len());
            let mut x = 0.0f32;
            let mut max_asc = font_size * 0.65;
            let mut max_desc = font_size * 0.18;
            for (idx, child) in children.iter().enumerate() {
                let m = measure_node(ui, child, font_size);
                // Add slight breathing gap between binary glyph ops (+, =, ->) and neighbors
                let is_rel_glyph = matches!(
                    child,
                    MathNode::Glyph(
                        GlyphKind::To
                            | GlyphKind::Mapsto
                            | GlyphKind::Rightarrow
                            | GlyphKind::Implies
                            | GlyphKind::Iff
                            | GlyphKind::Le
                            | GlyphKind::Ge
                            | GlyphKind::Ne
                            | GlyphKind::Approx
                            | GlyphKind::Equiv
                            | GlyphKind::InSet
                            | GlyphKind::SubsetEq
                            | GlyphKind::Sim
                            | GlyphKind::Cdot
                            | GlyphKind::Times
                            | GlyphKind::Pm
                    )
                );
                let extra = if is_rel_glyph { font_size * 0.14 } else { 0.0 };
                // Also subtle spacing for +/- inside non-leading position
                if matches!(child, MathNode::Run { is_op: true, .. }) && idx > 0 {
                    // Already padded in Run width
                }
                x += extra;
                max_asc = max_asc.max(m.ascent);
                max_desc = max_desc.max(m.descent);
                let w = m.width;
                items.push((x, 0.0, m));
                x += w + extra;
                let _ = &mut items;
            }
            MeasuredBox {
                width: x,
                ascent: max_asc,
                descent: max_desc,
                item: DrawnItem::Row(items),
            }
        }
        MathNode::Frac { num, den } => {
            let child_fs = (font_size * 0.80).max(9.5);
            let nm = measure_node(ui, num, child_fs);
            let dm = measure_node(ui, den, child_fs);
            let bar_pad = font_size * 0.16;
            let w = nm.width.max(dm.width) + 2.0 * bar_pad;
            let gap = (font_size * 0.14).max(2.0);
            let bar_y = -axis;
            let num_dy = bar_y - gap - nm.descent;
            let den_dy = bar_y + gap + dm.ascent;
            let asc = (-num_dy + nm.ascent).max(axis + gap);
            let desc = (den_dy + dm.descent).max(0.0);
            let num_dx = (w - nm.width) * 0.5;
            let den_dx = (w - dm.width) * 0.5;
            MeasuredBox {
                width: w,
                ascent: asc,
                descent: desc,
                item: DrawnItem::Frac {
                    num: Box::new(nm),
                    den: Box::new(dm),
                    num_dx,
                    num_dy,
                    den_dx,
                    den_dy,
                    bar_y,
                    bar_w: w,
                    stroke_w,
                },
            }
        }
        MathNode::Sqrt { index, inner } => {
            let im = measure_node(ui, inner, font_size);
            let rad_w = font_size * 0.55;
            let top_pad = (font_size * 0.16).max(2.5);
            let top_y = -(im.ascent + top_pad);
            let bot_y = im.descent + font_size * 0.05;
            let idx_box = index.as_ref().map(|ix| measure_node(ui, ix, (font_size * 0.62).max(8.5)));
            let idx_extra = idx_box
                .as_ref()
                .map(|ib| (ib.width - rad_w * 0.45).max(0.0))
                .unwrap_or(0.0);
            let radical_x0 = idx_extra;
            let inner_dx = radical_x0 + rad_w + font_size * 0.08;
            let w = inner_dx + im.width + font_size * 0.10;
            let idx_dy = -font_size * 0.36;
            let asc = (-top_y + stroke_w).max(
                idx_box
                    .as_ref()
                    .map(|ib| -idx_dy + ib.ascent)
                    .unwrap_or(0.0),
            );
            MeasuredBox {
                width: w,
                ascent: asc,
                descent: bot_y,
                item: DrawnItem::Sqrt {
                    index: idx_box.map(Box::new),
                    idx_dx: 0.0,
                    idx_dy,
                    inner: Box::new(im),
                    inner_dx,
                    radical_x0,
                    radical_w: rad_w,
                    top_y,
                    bot_y,
                    stroke_w,
                },
            }
        }
        MathNode::SupSub { base, sub, sup } => {
            let bm = measure_node(ui, base, font_size);
            let script_fs = (font_size * 0.68).max(6.8);
            let sub_m = sub.as_ref().map(|s| measure_node(ui, s, script_fs));
            let sup_m = sup.as_ref().map(|s| measure_node(ui, s, script_fs));

            let gap = font_size * 0.04;
            let sup_dy = -(bm.ascent * 0.72);
            let sub_dy = bm.descent * 0.65 + script_fs * 0.28;
            let mut max_w = bm.width;
            let mut asc = bm.ascent;
            let mut desc = bm.descent;
            let mut row = vec![(0.0, 0.0, bm)];
            if let Some(sm) = sup_m {
                let dx = max_w + gap;
                let w = sm.width;
                asc = asc.max(-sup_dy + sm.ascent);
                max_w = max_w.max(dx + w);
                row.push((dx, sup_dy, sm));
            }
            if let Some(bm_sub) = sub_m {
                let dx = row[0].2.width + gap;
                let w = bm_sub.width;
                desc = desc.max(sub_dy + bm_sub.descent);
                max_w = max_w.max(dx + w);
                row.push((dx, sub_dy, bm_sub));
            }
            MeasuredBox {
                width: max_w,
                ascent: asc,
                descent: desc,
                item: DrawnItem::Row(row),
            }
        }
        MathNode::BigOp { kind, sub, sup } => {
            let script_fs = (font_size * 0.66).max(8.5);
            let sub_m = sub.as_ref().map(|s| measure_node(ui, s, script_fs));
            let sup_m = sup.as_ref().map(|s| measure_node(ui, s, script_fs));

            match kind {
                BigOpKind::Int | BigOpKind::Oint | BigOpKind::Iint => {
                    let op_w = match kind {
                        BigOpKind::Iint => font_size * 0.92,
                        _ => font_size * 0.56,
                    };
                    let op_asc = font_size * 1.12;
                    let op_desc = font_size * 0.56;
                    let mut total_w = op_w + font_size * 0.08;
                    let mut asc = op_asc;
                    let mut desc = op_desc;
                    let sup_placed = sup_m.map(|sm| {
                        let dx = op_w + font_size * 0.04;
                        let dy = -(op_asc * 0.66);
                        total_w = total_w.max(dx + sm.width + font_size * 0.08);
                        asc = asc.max(-dy + sm.ascent);
                        (dx, dy, Box::new(sm))
                    });
                    let sub_placed = sub_m.map(|sm| {
                        let dx = op_w * 0.85;
                        let dy = op_desc * 0.72 + sm.ascent * 0.35;
                        total_w = total_w.max(dx + sm.width + font_size * 0.08);
                        desc = desc.max(dy + sm.descent);
                        (dx, dy, Box::new(sm))
                    });
                    MeasuredBox {
                        width: total_w,
                        ascent: asc,
                        descent: desc,
                        item: DrawnItem::BigOp {
                            kind: *kind,
                            font_size,
                            op_w,
                            op_ascent: op_asc,
                            op_descent: op_desc,
                            op_dx: 0.0,
                            op_dy: 0.0,
                            sub: sub_placed,
                            sup: sup_placed,
                        },
                    }
                }
                BigOpKind::Sum | BigOpKind::Prod => {
                    let op_w = font_size * 0.95;
                    let op_asc = font_size * 0.95;
                    let op_desc = font_size * 0.34;
                    let limit_gap = font_size * 0.10;
                    let mut max_w = op_w;
                    if let Some(ref sm) = sub_m {
                        max_w = max_w.max(sm.width);
                    }
                    if let Some(ref sm) = sup_m {
                        max_w = max_w.max(sm.width);
                    }
                    let op_dx = (max_w - op_w) * 0.5;
                    let mut asc = op_asc;
                    let mut desc = op_desc;
                    let sup_placed = sup_m.map(|sm| {
                        let dx = (max_w - sm.width) * 0.5;
                        let dy = -(op_asc + limit_gap + sm.descent);
                        asc = -dy + sm.ascent;
                        (dx, dy, Box::new(sm))
                    });
                    let sub_placed = sub_m.map(|sm| {
                        let dx = (max_w - sm.width) * 0.5;
                        let dy = op_desc + limit_gap + sm.ascent;
                        desc = dy + sm.descent;
                        (dx, dy, Box::new(sm))
                    });
                    MeasuredBox {
                        width: max_w + font_size * 0.12,
                        ascent: asc,
                        descent: desc,
                        item: DrawnItem::BigOp {
                            kind: *kind,
                            font_size,
                            op_w,
                            op_ascent: op_asc,
                            op_descent: op_desc,
                            op_dx,
                            op_dy: 0.0,
                            sub: sub_placed,
                            sup: sup_placed,
                        },
                    }
                }
                BigOpKind::Lim | BigOpKind::Argmax | BigOpKind::Max | BigOpKind::Min => {
                    let label = match kind {
                        BigOpKind::Lim => "lim",
                        BigOpKind::Argmax => "argmax",
                        BigOpKind::Max => "max",
                        BigOpKind::Min => "min",
                        _ => "op",
                    };
                    let sz = measure_text(ui, label, font_size, false);
                    let op_w = sz.x;
                    let op_asc = sz.y * 0.78;
                    let op_desc = sz.y * 0.22;
                    let mut max_w = op_w;
                    if let Some(ref sm) = sub_m {
                        max_w = max_w.max(sm.width);
                    }
                    if let Some(ref sm) = sup_m {
                        max_w = max_w.max(sm.width);
                    }
                    let op_dx = (max_w - op_w) * 0.5;
                    let limit_gap = font_size * 0.08;
                    let mut asc = op_asc;
                    let mut desc = op_desc;
                    let sup_placed = sup_m.map(|sm| {
                        let dx = (max_w - sm.width) * 0.5;
                        let dy = -(op_asc + limit_gap + sm.descent);
                        asc = -dy + sm.ascent;
                        (dx, dy, Box::new(sm))
                    });
                    let sub_placed = sub_m.map(|sm| {
                        let dx = (max_w - sm.width) * 0.5;
                        let dy = op_desc + limit_gap + sm.ascent;
                        desc = dy + sm.descent;
                        (dx, dy, Box::new(sm))
                    });
                    MeasuredBox {
                        width: max_w + font_size * 0.16,
                        ascent: asc,
                        descent: desc,
                        item: DrawnItem::BigOp {
                            kind: *kind,
                            font_size,
                            op_w,
                            op_ascent: op_asc,
                            op_descent: op_desc,
                            op_dx,
                            op_dy: 0.0,
                            sub: sub_placed,
                            sup: sup_placed,
                        },
                    }
                }
            }
        }
        MathNode::Delim { left, right, inner } => {
            let im = measure_node(ui, inner, font_size);
            let pad_v = font_size * 0.08;
            let top_y = -(im.ascent + pad_v);
            let bot_y = im.descent + pad_v;
            let delim_w = |d: DelimKind| -> f32 {
                match d {
                    DelimKind::Dot => 0.0,
                    DelimKind::Pipe => font_size * 0.24,
                    DelimKind::DoublePipe => font_size * 0.36,
                    DelimKind::BraceL | DelimKind::BraceR => font_size * 0.42,
                    DelimKind::AngleL | DelimKind::AngleR => font_size * 0.40,
                    _ => font_size * 0.34,
                }
            };
            let lw = delim_w(*left);
            let rw = delim_w(*right);
            let total_w = lw + im.width + rw;
            MeasuredBox {
                width: total_w,
                ascent: -top_y,
                descent: bot_y,
                item: DrawnItem::Delim {
                    left: *left,
                    right: *right,
                    lw,
                    rw,
                    top_y,
                    bot_y,
                    inner_dx: lw,
                    inner: Box::new(im),
                    stroke_w,
                },
            }
        }
        MathNode::Accent { kind, inner } => {
            let im = measure_node(ui, inner, font_size);
            let accent_h = font_size * 0.24;
            let accent_y = -(im.ascent + accent_h * 0.45);
            let asc = im.ascent + accent_h;
            MeasuredBox {
                width: im.width,
                ascent: asc,
                descent: im.descent,
                item: DrawnItem::Accent {
                    kind: *kind,
                    w: im.width,
                    inner: Box::new(im),
                    inner_dy: 0.0,
                    accent_y,
                    stroke_w,
                },
            }
        }
        MathNode::Matrix { style, rows } => {
            let cell_fs = (font_size * 0.88).max(10.0);
            let num_rows = rows.len().max(1);
            let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
            let mut measured_rows = Vec::with_capacity(num_rows);
            let mut col_widths = vec![0.0f32; num_cols];
            let mut row_ascents = vec![0.0f32; num_rows];
            let mut row_descents = vec![0.0f32; num_rows];

            for (r_idx, r) in rows.iter().enumerate() {
                let mut mrow = Vec::with_capacity(r.len());
                for (c_idx, cell) in r.iter().enumerate() {
                    let m = measure_node(ui, cell, cell_fs);
                    col_widths[c_idx] = col_widths[c_idx].max(m.width);
                    row_ascents[r_idx] = row_ascents[r_idx].max(m.ascent);
                    row_descents[r_idx] = row_descents[r_idx].max(m.descent);
                    mrow.push(m);
                }
                measured_rows.push(mrow);
            }

            let col_gap = font_size * 0.75;
            let row_gap = font_size * 0.34;
            let mut content_h = 0.0f32;
            for r in 0..num_rows {
                content_h += row_ascents[r] + row_descents[r];
                if r + 1 < num_rows {
                    content_h += row_gap;
                }
            }
            let center_shift = -axis;
            let top_y = center_shift - content_h * 0.5 - font_size * 0.10;
            let bot_y = center_shift + content_h * 0.5 + font_size * 0.10;

            let bracket_w = match style {
                MatrixStyle::Plain => 0.0,
                MatrixStyle::Cases => font_size * 0.45,
                _ => font_size * 0.36,
            };
            let right_w = match style {
                MatrixStyle::Plain | MatrixStyle::Cases => font_size * 0.12,
                _ => bracket_w,
            };

            let mut placed = Vec::with_capacity(num_rows);
            let mut cur_y = center_shift - content_h * 0.5;
            for (r_idx, mrow) in measured_rows.into_iter().enumerate() {
                let baseline_y = cur_y + row_ascents[r_idx];
                let mut cur_x = bracket_w + font_size * 0.12;
                let mut prow = Vec::with_capacity(mrow.len());
                for (c_idx, m) in mrow.into_iter().enumerate() {
                    let dx = if *style == MatrixStyle::Cases {
                        cur_x
                    } else {
                        cur_x + (col_widths[c_idx] - m.width) * 0.5
                    };
                    prow.push((dx, baseline_y, m));
                    cur_x += col_widths[c_idx];
                    if c_idx + 1 < num_cols {
                        cur_x += col_gap;
                    }
                }
                placed.push(prow);
                cur_y += row_ascents[r_idx] + row_descents[r_idx] + row_gap;
            }

            let cols_sum: f32 = col_widths.iter().sum::<f32>()
                + col_gap * (num_cols.saturating_sub(1) as f32);
            let total_w = bracket_w + font_size * 0.12 + cols_sum + right_w + font_size * 0.10;

            MeasuredBox {
                width: total_w,
                ascent: -top_y,
                descent: bot_y,
                item: DrawnItem::Matrix {
                    style: *style,
                    cells: placed,
                    top_y,
                    bot_y,
                    total_w,
                    bracket_w,
                    stroke_w,
                },
            }
        }
    }
}

// =========================================================================
// PAINTER GEOMETRY PRIMITIVES
// =========================================================================

fn cubic_bezier_points(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, steps: usize) -> Vec<Pos2> {
    let mut pts = Vec::with_capacity(steps + 1);
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * u * p0.x
            + 3.0 * u * u * t * p1.x
            + 3.0 * u * t * t * p2.x
            + t * t * t * p3.x;
        let y = u * u * u * p0.y
            + 3.0 * u * u * t * p1.y
            + 3.0 * u * t * t * p2.y
            + t * t * t * p3.y;
        pts.push(Pos2::new(x, y));
    }
    pts
}

fn draw_delim(
    painter: &egui::Painter,
    kind: DelimKind,
    x0: f32,
    w: f32,
    y_top: f32,
    y_bot: f32,
    stroke: Stroke,
) {
    let mid_y = (y_top + y_bot) * 0.5;
    let h = (y_bot - y_top).max(1.0);
    match kind {
        DelimKind::Dot => {}
        DelimKind::Pipe => {
            let x = x0 + w * 0.5;
            painter.line_segment([Pos2::new(x, y_top), Pos2::new(x, y_bot)], stroke);
        }
        DelimKind::DoublePipe => {
            let x1 = x0 + w * 0.34;
            let x2 = x0 + w * 0.66;
            painter.line_segment([Pos2::new(x1, y_top), Pos2::new(x1, y_bot)], stroke);
            painter.line_segment([Pos2::new(x2, y_top), Pos2::new(x2, y_bot)], stroke);
        }
        DelimKind::ParenL => {
            let rx = x0 + w * 0.85;
            let cx = x0 + w * 0.15;
            let pts = cubic_bezier_points(
                Pos2::new(rx, y_top),
                Pos2::new(cx, y_top + h * 0.25),
                Pos2::new(cx, y_bot - h * 0.25),
                Pos2::new(rx, y_bot),
                14,
            );
            painter.add(egui::Shape::line(pts, stroke));
        }
        DelimKind::ParenR => {
            let lx = x0 + w * 0.15;
            let cx = x0 + w * 0.85;
            let pts = cubic_bezier_points(
                Pos2::new(lx, y_top),
                Pos2::new(cx, y_top + h * 0.25),
                Pos2::new(cx, y_bot - h * 0.25),
                Pos2::new(lx, y_bot),
                14,
            );
            painter.add(egui::Shape::line(pts, stroke));
        }
        DelimKind::BracketL => {
            let x_l = x0 + w * 0.32;
            let x_r = x0 + w * 0.88;
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(x_r, y_top),
                    Pos2::new(x_l, y_top),
                    Pos2::new(x_l, y_bot),
                    Pos2::new(x_r, y_bot),
                ],
                stroke,
            ));
        }
        DelimKind::BracketR => {
            let x_l = x0 + w * 0.12;
            let x_r = x0 + w * 0.68;
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(x_l, y_top),
                    Pos2::new(x_r, y_top),
                    Pos2::new(x_r, y_bot),
                    Pos2::new(x_l, y_bot),
                ],
                stroke,
            ));
        }
        DelimKind::BraceL => {
            let x_r = x0 + w * 0.84;
            let x_m = x0 + w * 0.50;
            let x_tip = x0 + w * 0.16;
            let top_pts = cubic_bezier_points(
                Pos2::new(x_r, y_top),
                Pos2::new(x_m, y_top),
                Pos2::new(x_m, mid_y - h * 0.08),
                Pos2::new(x_tip, mid_y),
                10,
            );
            let bot_pts = cubic_bezier_points(
                Pos2::new(x_tip, mid_y),
                Pos2::new(x_m, mid_y + h * 0.08),
                Pos2::new(x_m, y_bot),
                Pos2::new(x_r, y_bot),
                10,
            );
            painter.add(egui::Shape::line(top_pts, stroke));
            painter.add(egui::Shape::line(bot_pts, stroke));
        }
        DelimKind::BraceR => {
            let x_l = x0 + w * 0.16;
            let x_m = x0 + w * 0.50;
            let x_tip = x0 + w * 0.84;
            let top_pts = cubic_bezier_points(
                Pos2::new(x_l, y_top),
                Pos2::new(x_m, y_top),
                Pos2::new(x_m, mid_y - h * 0.08),
                Pos2::new(x_tip, mid_y),
                10,
            );
            let bot_pts = cubic_bezier_points(
                Pos2::new(x_tip, mid_y),
                Pos2::new(x_m, mid_y + h * 0.08),
                Pos2::new(x_m, y_bot),
                Pos2::new(x_l, y_bot),
                10,
            );
            painter.add(egui::Shape::line(top_pts, stroke));
            painter.add(egui::Shape::line(bot_pts, stroke));
        }
        DelimKind::AngleL => {
            let x_l = x0 + w * 0.20;
            let x_r = x0 + w * 0.86;
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(x_r, y_top),
                    Pos2::new(x_l, mid_y),
                    Pos2::new(x_r, y_bot),
                ],
                stroke,
            ));
        }
        DelimKind::AngleR => {
            let x_l = x0 + w * 0.14;
            let x_r = x0 + w * 0.80;
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(x_l, y_top),
                    Pos2::new(x_r, mid_y),
                    Pos2::new(x_l, y_bot),
                ],
                stroke,
            ));
        }
    }
}

fn draw_integral_curve(
    painter: &egui::Painter,
    cx: f32,
    y_top: f32,
    y_bot: f32,
    fs: f32,
    color: Color32,
    closed_loop: bool,
) {
    let sw = (fs * 0.095).clamp(1.2, 2.5);
    let st = Stroke::new(sw, color);
    let h = y_bot - y_top;
    let dx = fs * 0.18;
    let upper = cubic_bezier_points(
        Pos2::new(cx + dx * 0.9, y_top + h * 0.06),
        Pos2::new(cx + dx * 0.25, y_top - h * 0.02),
        Pos2::new(cx - dx * 0.02, y_top + h * 0.25),
        Pos2::new(cx, (y_top + y_bot) * 0.5),
        12,
    );
    let lower = cubic_bezier_points(
        Pos2::new(cx, (y_top + y_bot) * 0.5),
        Pos2::new(cx + dx * 0.02, y_bot - h * 0.25),
        Pos2::new(cx - dx * 0.25, y_bot + h * 0.02),
        Pos2::new(cx - dx * 0.9, y_bot - h * 0.06),
        12,
    );
    painter.add(egui::Shape::line(upper, st));
    painter.add(egui::Shape::line(lower, st));
    painter.circle_filled(Pos2::new(cx + dx * 0.9, y_top + h * 0.07), sw * 0.8, color);
    painter.circle_filled(Pos2::new(cx - dx * 0.9, y_bot - h * 0.07), sw * 0.8, color);
    if closed_loop {
        painter.circle_stroke(
            Pos2::new(cx, (y_top + y_bot) * 0.5),
            fs * 0.21,
            Stroke::new(sw * 0.8, color),
        );
    }
}

fn draw_vector_glyph(
    painter: &egui::Painter,
    kind: GlyphKind,
    x: f32,
    baseline_y: f32,
    w: f32,
    fs: f32,
    color: Color32,
) {
    let sw = (fs * 0.075).clamp(1.05, 2.2);
    let st = Stroke::new(sw, color);
    let top = baseline_y - fs * 0.68;
    let mid = baseline_y - fs * 0.28;
    let bot = baseline_y + fs * 0.06;
    let lx = x + w * 0.10;
    let rx = x + w * 0.90;
    let cx = x + w * 0.50;

    let draw_text_fallback = |glyph: &str, italic: bool| {
        let txt_pos = Pos2::new(x + w * 0.5, baseline_y + fs * 0.18);
        let fid = FontId::new(fs, FontFamily::Proportional);
        if italic {
            let g = painter.layout_no_wrap(glyph.to_string(), fid, color);
            painter.galley(
                Pos2::new(cx - g.size().x * 0.5, baseline_y - g.size().y * 0.78),
                g,
                color,
            );
        } else {
            painter.text(txt_pos, Align2::CENTER_BOTTOM, glyph, fid, color);
        }
    };

    match kind {
        GlyphKind::Forall => {
            painter.line_segment([Pos2::new(lx, top), Pos2::new(cx, bot)], st);
            painter.line_segment([Pos2::new(rx, top), Pos2::new(cx, bot)], st);
            let bar_y = top + (bot - top) * 0.42;
            let bar_inset = (rx - lx) * 0.20;
            painter.line_segment(
                [Pos2::new(lx + bar_inset, bar_y), Pos2::new(rx - bar_inset, bar_y)],
                st,
            );
        }
        GlyphKind::Exists => {
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(lx, top),
                    Pos2::new(rx, top),
                    Pos2::new(rx, bot),
                    Pos2::new(lx, bot),
                ],
                st,
            ));
            painter.line_segment(
                [Pos2::new(lx + w * 0.12, mid), Pos2::new(rx, mid)],
                st,
            );
        }
        GlyphKind::InSet => {
            let r_top = top + fs * 0.10;
            let r_bot = bot - fs * 0.04;
            let r_mid = (r_top + r_bot) * 0.5;
            let c_pts = cubic_bezier_points(
                Pos2::new(rx, r_top),
                Pos2::new(lx, r_top),
                Pos2::new(lx, r_bot),
                Pos2::new(rx, r_bot),
                12,
            );
            painter.add(egui::Shape::line(c_pts, st));
            painter.line_segment([Pos2::new(lx + w * 0.14, r_mid), Pos2::new(rx, r_mid)], st);
        }
        GlyphKind::SubsetEq => {
            let r_top = top + fs * 0.10;
            let r_bot = bot - fs * 0.14;
            let c_pts = cubic_bezier_points(
                Pos2::new(rx, r_top),
                Pos2::new(lx, r_top),
                Pos2::new(lx, r_bot),
                Pos2::new(rx, r_bot),
                12,
            );
            painter.add(egui::Shape::line(c_pts, st));
            painter.line_segment(
                [Pos2::new(lx + w * 0.04, bot + fs * 0.02), Pos2::new(rx, bot + fs * 0.02)],
                st,
            );
        }
        GlyphKind::MathbbR => {
            let s1 = lx + w * 0.08;
            let s2 = lx + w * 0.25;
            let rb = bot - fs * 0.06;
            painter.line_segment([Pos2::new(s1, top), Pos2::new(s1, rb)], st);
            painter.line_segment([Pos2::new(s2, top), Pos2::new(s2, rb)], st);
            painter.line_segment([Pos2::new(lx, top), Pos2::new(cx, top)], st);
            let b_mid = (top + rb) * 0.50;
            let bowl = cubic_bezier_points(
                Pos2::new(cx, top),
                Pos2::new(rx, top),
                Pos2::new(rx, b_mid),
                Pos2::new(s2, b_mid),
                10,
            );
            painter.add(egui::Shape::line(bowl, st));
            painter.line_segment([Pos2::new(cx - w * 0.04, b_mid), Pos2::new(rx, rb)], st);
        }
        GlyphKind::Nabla => {
            painter.add(egui::Shape::closed_line(
                vec![Pos2::new(lx, top), Pos2::new(rx, top), Pos2::new(cx, bot)],
                st,
            ));
        }
        GlyphKind::DeltaCap => {
            painter.add(egui::Shape::closed_line(
                vec![Pos2::new(cx, top), Pos2::new(rx, bot - fs * 0.06), Pos2::new(lx, bot - fs * 0.06)],
                st,
            ));
        }
        GlyphKind::OmegaCap => {
            let b_y = bot - fs * 0.06;
            let c_pts = cubic_bezier_points(
                Pos2::new(cx - w * 0.16, b_y),
                Pos2::new(lx - w * 0.10, top),
                Pos2::new(rx + w * 0.10, top),
                Pos2::new(cx + w * 0.16, b_y),
                16,
            );
            painter.add(egui::Shape::line(c_pts, st));
            painter.line_segment([Pos2::new(lx, b_y), Pos2::new(cx - w * 0.16, b_y)], st);
            painter.line_segment([Pos2::new(cx + w * 0.16, b_y), Pos2::new(rx, b_y)], st);
        }
        GlyphKind::To => {
            let head = fs * 0.14;
            painter.line_segment([Pos2::new(lx, mid), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid - head * 0.8), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid + head * 0.8), Pos2::new(rx, mid)], st);
        }
        GlyphKind::Mapsto => {
            let head = fs * 0.14;
            painter.line_segment([Pos2::new(lx, mid - head), Pos2::new(lx, mid + head)], st);
            painter.line_segment([Pos2::new(lx, mid), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid - head * 0.8), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid + head * 0.8), Pos2::new(rx, mid)], st);
        }
        GlyphKind::Rightarrow | GlyphKind::Implies => {
            let head = fs * 0.17;
            let dy = fs * 0.075;
            painter.line_segment([Pos2::new(lx, mid - dy), Pos2::new(rx - head * 0.7, mid - dy)], st);
            painter.line_segment([Pos2::new(lx, mid + dy), Pos2::new(rx - head * 0.7, mid + dy)], st);
            painter.line_segment([Pos2::new(rx - head, mid - head * 0.9), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid + head * 0.9), Pos2::new(rx, mid)], st);
        }
        GlyphKind::Iff => {
            let head = fs * 0.16;
            let dy = fs * 0.075;
            painter.line_segment([Pos2::new(lx + head * 0.7, mid - dy), Pos2::new(rx - head * 0.7, mid - dy)], st);
            painter.line_segment([Pos2::new(lx + head * 0.7, mid + dy), Pos2::new(rx - head * 0.7, mid + dy)], st);
            painter.line_segment([Pos2::new(lx + head, mid - head * 0.85), Pos2::new(lx, mid)], st);
            painter.line_segment([Pos2::new(lx + head, mid + head * 0.85), Pos2::new(lx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid - head * 0.85), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(rx - head, mid + head * 0.85), Pos2::new(rx, mid)], st);
        }
        GlyphKind::Le => {
            let t = top + fs * 0.14;
            let b = mid + fs * 0.18;
            let m = (t + b) * 0.5;
            painter.add(egui::Shape::line(
                vec![Pos2::new(rx, t), Pos2::new(lx, m), Pos2::new(rx, b)],
                st,
            ));
            painter.line_segment([Pos2::new(lx, b + fs * 0.10), Pos2::new(rx, b + fs * 0.10)], st);
        }
        GlyphKind::Ge => {
            let t = top + fs * 0.14;
            let b = mid + fs * 0.18;
            let m = (t + b) * 0.5;
            painter.add(egui::Shape::line(
                vec![Pos2::new(lx, t), Pos2::new(rx, m), Pos2::new(lx, b)],
                st,
            ));
            painter.line_segment([Pos2::new(lx, b + fs * 0.10), Pos2::new(rx, b + fs * 0.10)], st);
        }
        GlyphKind::Ne => {
            let dy = fs * 0.085;
            painter.line_segment([Pos2::new(lx, mid - dy), Pos2::new(rx, mid - dy)], st);
            painter.line_segment([Pos2::new(lx, mid + dy), Pos2::new(rx, mid + dy)], st);
            painter.line_segment(
                [Pos2::new(cx + fs * 0.14, top + fs * 0.12), Pos2::new(cx - fs * 0.14, bot - fs * 0.04)],
                st,
            );
        }
        GlyphKind::Approx => {
            let dy = fs * 0.085;
            for sgn in [-1.0, 1.0] {
                let y0 = mid + sgn * dy;
                let pts = cubic_bezier_points(
                    Pos2::new(lx, y0 + fs * 0.04),
                    Pos2::new(lx + w * 0.25, y0 - fs * 0.12),
                    Pos2::new(lx + w * 0.55, y0 + fs * 0.12),
                    Pos2::new(rx, y0 - fs * 0.04),
                    10,
                );
                painter.add(egui::Shape::line(pts, st));
            }
        }
        GlyphKind::Equiv => {
            let dy = fs * 0.12;
            painter.line_segment([Pos2::new(lx, mid - dy), Pos2::new(rx, mid - dy)], st);
            painter.line_segment([Pos2::new(lx, mid), Pos2::new(rx, mid)], st);
            painter.line_segment([Pos2::new(lx, mid + dy), Pos2::new(rx, mid + dy)], st);
        }
        GlyphKind::Cdot => {
            painter.circle_filled(Pos2::new(cx, mid), (fs * 0.09).max(1.4), color);
        }
        GlyphKind::Circ => {
            painter.circle_stroke(Pos2::new(cx, mid), (fs * 0.14).max(2.0), st);
        }
        GlyphKind::Pm => {
            let r = fs * 0.22;
            painter.line_segment([Pos2::new(cx - r, mid - r * 0.2), Pos2::new(cx + r, mid - r * 0.2)], st);
            painter.line_segment([Pos2::new(cx, mid - r * 1.15), Pos2::new(cx, mid + r * 0.75)], st);
            painter.line_segment([Pos2::new(cx - r, mid + r * 1.15), Pos2::new(cx + r, mid + r * 1.15)], st);
        }
        GlyphKind::Times => {
            let r = fs * 0.19;
            painter.line_segment([Pos2::new(cx - r, mid - r), Pos2::new(cx + r, mid + r)], st);
            painter.line_segment([Pos2::new(cx + r, mid - r), Pos2::new(cx - r, mid + r)], st);
        }
        GlyphKind::Neg => {
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(lx, mid),
                    Pos2::new(rx, mid),
                    Pos2::new(rx, mid + fs * 0.18),
                ],
                st,
            ));
        }
        GlyphKind::Sim => {
            let pts = cubic_bezier_points(
                Pos2::new(lx, mid + fs * 0.04),
                Pos2::new(lx + w * 0.25, mid - fs * 0.16),
                Pos2::new(lx + w * 0.55, mid + fs * 0.16),
                Pos2::new(rx, mid - fs * 0.04),
                10,
            );
            painter.add(egui::Shape::line(pts, st));
        }
        GlyphKind::Infty => {
            let r = fs * 0.17;
            let dy = fs * 0.17;
            let left = cubic_bezier_points(
                Pos2::new(cx, mid),
                Pos2::new(cx - r * 1.3, mid - dy * 1.3),
                Pos2::new(cx - r * 1.3, mid + dy * 1.3),
                Pos2::new(cx, mid),
                14,
            );
            let right = cubic_bezier_points(
                Pos2::new(cx, mid),
                Pos2::new(cx + r * 1.3, mid - dy * 1.3),
                Pos2::new(cx + r * 1.3, mid + dy * 1.3),
                Pos2::new(cx, mid),
                14,
            );
            painter.add(egui::Shape::line(left, st));
            painter.add(egui::Shape::line(right, st));
        }
        GlyphKind::Ell => {
            let pts = cubic_bezier_points(
                Pos2::new(lx + w * 0.15, mid + fs * 0.12),
                Pos2::new(rx + w * 0.12, top - fs * 0.04),
                Pos2::new(lx + w * 0.05, top),
                Pos2::new(cx + w * 0.15, bot - fs * 0.05),
                16,
            );
            painter.add(egui::Shape::line(pts, st));
        }
        GlyphKind::CalL => {
            let pts = cubic_bezier_points(
                Pos2::new(lx + w * 0.25, top + fs * 0.18),
                Pos2::new(rx, top - fs * 0.10),
                Pos2::new(lx + w * 0.15, bot),
                Pos2::new(rx, bot - fs * 0.06),
                16,
            );
            painter.add(egui::Shape::line(pts, st));
        }
        GlyphKind::AngleL => draw_delim(painter, DelimKind::AngleL, x, w, top, bot, st),
        GlyphKind::AngleR => draw_delim(painter, DelimKind::AngleR, x, w, top, bot, st),
        GlyphKind::DoublePipe => draw_delim(painter, DelimKind::DoublePipe, x, w, top, bot, st),
        GlyphKind::Pi => draw_text_fallback("π", true),
        GlyphKind::Theta => draw_text_fallback("θ", true),
        GlyphKind::Lambda => draw_text_fallback("λ", true),
        GlyphKind::Alpha => draw_text_fallback("α", true),
        GlyphKind::Beta => draw_text_fallback("β", true),
        GlyphKind::Gamma => draw_text_fallback("γ", true),
        GlyphKind::Delta => draw_text_fallback("δ", true),
        GlyphKind::Varepsilon => draw_text_fallback("ε", true),
        GlyphKind::Mu => draw_text_fallback("μ", true),
        GlyphKind::Sigma => draw_text_fallback("σ", true),
        GlyphKind::Phi => draw_text_fallback("φ", true),
        GlyphKind::Omega => draw_text_fallback("ω", true),
        GlyphKind::Partial => draw_text_fallback("∂", false),
    }
}

fn paint_box(painter: &egui::Painter, m: &MeasuredBox, x: f32, baseline_y: f32, color: Color32) {
    match &m.item {
        DrawnItem::Empty => {}
        DrawnItem::Row(items) => {
            for (dx, dy, child) in items {
                paint_box(painter, child, x + *dx, baseline_y + *dy, color);
            }
        }
        DrawnItem::Text {
            text,
            font_size,
            italic,
            bold,
        } => {
            let fid = FontId::new(*font_size, FontFamily::Proportional);
            let galley = painter.layout_no_wrap(text.clone(), fid, color);
            let pad = (m.width - galley.size().x) * 0.5;
            let top_y = baseline_y - galley.size().y * 0.78;
            let pos = Pos2::new(x + pad, top_y);
            if *italic && text.chars().all(|c| c.is_ascii_alphabetic()) {
                // Slight italic shear or subtle slant cue via clean italic-style offset if bold/regular
                painter.galley(pos, galley.clone(), color);
            } else if *bold {
                painter.galley(pos, galley.clone(), color);
                painter.galley(Pos2::new(pos.x + 0.55, pos.y), galley, color);
            } else {
                painter.galley(pos, galley, color);
            }
        }
        DrawnItem::Glyph { kind, font_size } => {
            draw_vector_glyph(painter, *kind, x, baseline_y, m.width, *font_size, color);
        }
        DrawnItem::Frac {
            num,
            den,
            num_dx,
            num_dy,
            den_dx,
            den_dy,
            bar_y,
            bar_w,
            stroke_w,
        } => {
            paint_box(painter, num, x + *num_dx, baseline_y + *num_dy, color);
            paint_box(painter, den, x + *den_dx, baseline_y + *den_dy, color);
            let y = baseline_y + *bar_y;
            painter.line_segment(
                [Pos2::new(x + 1.5, y), Pos2::new(x + *bar_w - 1.5, y)],
                Stroke::new(*stroke_w, color),
            );
        }
        DrawnItem::Sqrt {
            index,
            idx_dx,
            idx_dy,
            inner,
            inner_dx,
            radical_x0,
            radical_w,
            top_y,
            bot_y,
            stroke_w,
        } => {
            if let Some(ix) = index {
                paint_box(painter, ix, x + *idx_dx, baseline_y + *idx_dy, color);
            }
            paint_box(painter, inner, x + *inner_dx, baseline_y, color);
            let st = Stroke::new(*stroke_w, color);
            let rx0 = x + *radical_x0;
            let rx_mid = rx0 + radical_w * 0.48;
            let rx_top = rx0 + radical_w;
            let y_top = baseline_y + *top_y;
            let y_bot = baseline_y + *bot_y;
            let y_hook = (y_top + y_bot) * 0.52;
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(rx0 + radical_w * 0.08, y_hook),
                    Pos2::new(rx0 + radical_w * 0.25, y_hook - stroke_w * 1.4),
                    Pos2::new(rx_mid, y_bot),
                    Pos2::new(rx_top, y_top),
                    Pos2::new(x + m.width - 1.0, y_top),
                ],
                st,
            ));
        }
        DrawnItem::BigOp {
            kind,
            font_size,
            op_w,
            op_ascent,
            op_descent,
            op_dx,
            op_dy,
            sub,
            sup,
        } => {
            let ox = x + *op_dx;
            let oy = baseline_y + *op_dy;
            let y_top = oy - *op_ascent;
            let y_bot = oy + *op_descent;
            let sw = (font_size * 0.085).clamp(1.2, 2.4);
            let st = Stroke::new(sw, color);
            match kind {
                BigOpKind::Int => {
                    draw_integral_curve(painter, ox + op_w * 0.5, y_top, y_bot, *font_size, color, false);
                }
                BigOpKind::Oint => {
                    draw_integral_curve(painter, ox + op_w * 0.5, y_top, y_bot, *font_size, color, true);
                }
                BigOpKind::Iint => {
                    draw_integral_curve(painter, ox + op_w * 0.30, y_top, y_bot, *font_size, color, false);
                    draw_integral_curve(painter, ox + op_w * 0.70, y_top, y_bot, *font_size, color, false);
                }
                BigOpKind::Sum => {
                    let lx = ox + op_w * 0.12;
                    let rx = ox + op_w * 0.88;
                    let mx = ox + op_w * 0.52;
                    let my = (y_top + y_bot) * 0.5;
                    painter.add(egui::Shape::line(
                        vec![
                            Pos2::new(rx, y_top + sw * 1.2),
                            Pos2::new(rx, y_top),
                            Pos2::new(lx, y_top),
                            Pos2::new(mx, my),
                            Pos2::new(lx, y_bot),
                            Pos2::new(rx, y_bot),
                            Pos2::new(rx, y_bot - sw * 1.2),
                        ],
                        st,
                    ));
                }
                BigOpKind::Prod => {
                    let lx = ox + op_w * 0.12;
                    let rx = ox + op_w * 0.88;
                    let leg_l = ox + op_w * 0.28;
                    let leg_r = ox + op_w * 0.72;
                    painter.line_segment([Pos2::new(lx, y_top), Pos2::new(rx, y_top)], st);
                    painter.line_segment([Pos2::new(leg_l, y_top), Pos2::new(leg_l, y_bot)], st);
                    painter.line_segment([Pos2::new(leg_r, y_top), Pos2::new(leg_r, y_bot)], st);
                    painter.line_segment([Pos2::new(lx, y_bot), Pos2::new(leg_l + op_w * 0.10, y_bot)], st);
                    painter.line_segment([Pos2::new(leg_r - op_w * 0.10, y_bot), Pos2::new(rx, y_bot)], st);
                }
                BigOpKind::Lim | BigOpKind::Argmax | BigOpKind::Max | BigOpKind::Min => {
                    let label = match kind {
                        BigOpKind::Lim => "lim",
                        BigOpKind::Argmax => "argmax",
                        BigOpKind::Max => "max",
                        BigOpKind::Min => "min",
                        _ => "",
                    };
                    let fid = FontId::new(*font_size, FontFamily::Proportional);
                    let galley = painter.layout_no_wrap(label.to_string(), fid, color);
                    painter.galley(Pos2::new(ox, oy - galley.size().y * 0.78), galley, color);
                }
            }
            if let Some((dx, dy, sbox)) = sup {
                paint_box(painter, sbox, x + *dx, baseline_y + *dy, color);
            }
            if let Some((dx, dy, sbox)) = sub {
                paint_box(painter, sbox, x + *dx, baseline_y + *dy, color);
            }
        }
        DrawnItem::Delim {
            left,
            right,
            lw,
            rw,
            top_y,
            bot_y,
            inner_dx,
            inner,
            stroke_w,
        } => {
            let st = Stroke::new(*stroke_w, color);
            draw_delim(painter, *left, x, *lw, baseline_y + *top_y, baseline_y + *bot_y, st);
            paint_box(painter, inner, x + *inner_dx, baseline_y, color);
            draw_delim(
                painter,
                *right,
                x + m.width - *rw,
                *rw,
                baseline_y + *top_y,
                baseline_y + *bot_y,
                st,
            );
        }
        DrawnItem::Accent {
            kind,
            inner,
            inner_dy,
            accent_y,
            w,
            stroke_w,
        } => {
            paint_box(painter, inner, x, baseline_y + *inner_dy, color);
            let st = Stroke::new(*stroke_w, color);
            let ay = baseline_y + *accent_y;
            let cx = x + *w * 0.5;
            match kind {
                AccentKind::Hat => {
                    let hw = (*w * 0.36).max(3.5);
                    let hh = (*w * 0.22).max(2.4);
                    painter.add(egui::Shape::line(
                        vec![
                            Pos2::new(cx - hw, ay + hh * 0.4),
                            Pos2::new(cx, ay - hh * 0.6),
                            Pos2::new(cx + hw, ay + hh * 0.4),
                        ],
                        st,
                    ));
                }
                AccentKind::Bar => {
                    painter.line_segment(
                        [Pos2::new(x + 1.0, ay), Pos2::new(x + *w - 1.0, ay)],
                        st,
                    );
                }
                AccentKind::Vec => {
                    let x0 = x + 1.0;
                    let x1 = x + *w - 1.0;
                    painter.line_segment([Pos2::new(x0, ay), Pos2::new(x1, ay)], st);
                    painter.line_segment([Pos2::new(x1 - 3.0, ay - 2.2), Pos2::new(x1, ay)], st);
                }
                AccentKind::Dot => {
                    painter.circle_filled(Pos2::new(cx, ay), (*stroke_w * 0.95).max(1.3), color);
                }
                AccentKind::Ddot => {
                    painter.circle_filled(Pos2::new(cx - 2.2, ay), (*stroke_w * 0.85).max(1.2), color);
                    painter.circle_filled(Pos2::new(cx + 2.2, ay), (*stroke_w * 0.85).max(1.2), color);
                }
            }
        }
        DrawnItem::Matrix {
            style,
            cells,
            top_y,
            bot_y,
            total_w,
            bracket_w,
            stroke_w,
        } => {
            let st = Stroke::new(*stroke_w, color);
            let y_top = baseline_y + *top_y;
            let y_bot = baseline_y + *bot_y;
            match style {
                MatrixStyle::Bracket => {
                    draw_delim(painter, DelimKind::BracketL, x, *bracket_w, y_top, y_bot, st);
                    draw_delim(painter, DelimKind::BracketR, x + *total_w - *bracket_w, *bracket_w, y_top, y_bot, st);
                }
                MatrixStyle::Paren => {
                    draw_delim(painter, DelimKind::ParenL, x, *bracket_w, y_top, y_bot, st);
                    draw_delim(painter, DelimKind::ParenR, x + *total_w - *bracket_w, *bracket_w, y_top, y_bot, st);
                }
                MatrixStyle::Vbar => {
                    draw_delim(painter, DelimKind::Pipe, x, *bracket_w, y_top, y_bot, st);
                    draw_delim(painter, DelimKind::Pipe, x + *total_w - *bracket_w, *bracket_w, y_top, y_bot, st);
                }
                MatrixStyle::Cases => {
                    draw_delim(painter, DelimKind::BraceL, x, *bracket_w, y_top, y_bot, st);
                }
                MatrixStyle::Plain => {}
            }
            for row in cells {
                for (dx, dy, cbox) in row {
                    paint_box(painter, cbox, x + *dx, baseline_y + *dy, color);
                }
            }
        }
    }
}

// =========================================================================
// PUBLIC WIDGET API
// =========================================================================

#[allow(dead_code)]
pub fn latex_size(ui: &Ui, latex: &str, font_size: f32) -> Vec2 {
    let ast = parse_latex(latex);
    let m = measure_node(ui, &ast, font_size);
    Vec2::new(m.width.max(4.0), m.height().max(font_size))
}

pub fn paint_latex(ui: &mut Ui, latex: &str, font_size: f32, color: Color32) -> Response {
    let ast = parse_latex(latex);
    let mut m = measure_node(ui, &ast, font_size);
    let avail = ui.available_width().max(80.0);
    // Shrink gracefully if a long expression exceeds narrow column width so nothing gets clipped
    if m.width > avail && avail > 120.0 {
        let ratio = (avail / m.width).clamp(0.68, 1.0);
        let scaled_fs = (font_size * ratio).max(10.0);
        m = measure_node(ui, &ast, scaled_fs);
    }
    let pad_y = 3.5;
    let size = Vec2::new(m.width.max(4.0), m.height() + 2.0 * pad_y);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let baseline_y = rect.min.y + pad_y + m.ascent;
        paint_box(ui.painter(), &m, rect.min.x, baseline_y, color);
    }
    resp
}

pub fn paint_latex_boxed(
    ui: &mut Ui,
    latex: &str,
    font_size: f32,
    text_color: Color32,
    bg: Color32,
    border: Stroke,
) -> Response {
    egui::Frame::none()
        .fill(bg)
        .stroke(border)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| paint_latex(ui, latex, font_size, text_color))
        .inner
}

pub fn paint_transformation_arrow(ui: &mut Ui, font_size: f32, color: Color32) -> Response {
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(2.0, 8.0))
        .show(ui, |ui| paint_latex(ui, r"\implies", font_size, color))
        .inner
}
