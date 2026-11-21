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

use calc_core::Stage;
use egui::{Color32, Context, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcademicTheme {
    SlateDark,
    PaperLight,
}

impl AcademicTheme {
    pub fn apply(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(14.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);

        let mut visuals = match self {
            AcademicTheme::SlateDark => Visuals::dark(),
            AcademicTheme::PaperLight => Visuals::light(),
        };

        match self {
            AcademicTheme::SlateDark => {
                visuals.panel_fill = Color32::from_rgb(22, 25, 31);
                visuals.window_fill = Color32::from_rgb(26, 30, 38);
                visuals.extreme_bg_color = Color32::from_rgb(15, 18, 23);
                visuals.faint_bg_color = Color32::from_rgb(30, 35, 44);
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 33, 42);
                visuals.widgets.noninteractive.bg_stroke =
                    Stroke::new(1.0_f32, Color32::from_rgb(52, 60, 74));
                visuals.widgets.inactive.rounding = Rounding::same(5.0);
                visuals.widgets.hovered.rounding = Rounding::same(5.0);
                visuals.widgets.active.rounding = Rounding::same(5.0);
            }
            AcademicTheme::PaperLight => {
                visuals.panel_fill = Color32::from_rgb(246, 245, 240);
                visuals.window_fill = Color32::from_rgb(252, 251, 248);
                visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
                visuals.faint_bg_color = Color32::from_rgb(237, 235, 228);
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(242, 240, 234);
                visuals.widgets.noninteractive.bg_stroke =
                    Stroke::new(1.0_f32, Color32::from_rgb(206, 203, 194));
                visuals.widgets.inactive.rounding = Rounding::same(5.0);
                visuals.widgets.hovered.rounding = Rounding::same(5.0);
                visuals.widgets.active.rounding = Rounding::same(5.0);
            }
        }

        style.visuals = visuals;
        ctx.set_style(style);
    }

    pub fn stage_accent(&self, stage: Stage) -> Color32 {
        match (self, stage) {
            (AcademicTheme::SlateDark, Stage::Notation) => Color32::from_rgb(232, 196, 92),
            (AcademicTheme::SlateDark, Stage::Algebra) => Color32::from_rgb(92, 164, 230),
            (AcademicTheme::SlateDark, Stage::Trigonometry) => Color32::from_rgb(82, 190, 152),
            (AcademicTheme::SlateDark, Stage::Calculus) => Color32::from_rgb(224, 166, 78),
            (AcademicTheme::SlateDark, Stage::LinearAlgebra) => Color32::from_rgb(76, 196, 208),
            (AcademicTheme::SlateDark, Stage::AdvancedCalculus) => Color32::from_rgb(198, 126, 214),
            (AcademicTheme::SlateDark, Stage::DifferentialEquations) => {
                Color32::from_rgb(230, 122, 108)
            }
            (AcademicTheme::SlateDark, Stage::ProbabilityStats) => Color32::from_rgb(154, 208, 96),
            (AcademicTheme::PaperLight, Stage::Notation) => Color32::from_rgb(142, 106, 14),
            (AcademicTheme::PaperLight, Stage::Algebra) => Color32::from_rgb(33, 97, 160),
            (AcademicTheme::PaperLight, Stage::Trigonometry) => Color32::from_rgb(28, 124, 92),
            (AcademicTheme::PaperLight, Stage::Calculus) => Color32::from_rgb(168, 102, 20),
            (AcademicTheme::PaperLight, Stage::LinearAlgebra) => Color32::from_rgb(18, 118, 132),
            (AcademicTheme::PaperLight, Stage::AdvancedCalculus) => Color32::from_rgb(126, 56, 145),
            (AcademicTheme::PaperLight, Stage::DifferentialEquations) => {
                Color32::from_rgb(172, 58, 44)
            }
            (AcademicTheme::PaperLight, Stage::ProbabilityStats) => Color32::from_rgb(72, 126, 28),
        }
    }

    pub fn ok_color(&self) -> Color32 {
        match self {
            AcademicTheme::SlateDark => Color32::from_rgb(84, 196, 132),
            AcademicTheme::PaperLight => Color32::from_rgb(26, 128, 72),
        }
    }

    pub fn warn_color(&self) -> Color32 {
        match self {
            AcademicTheme::SlateDark => Color32::from_rgb(228, 168, 72),
            AcademicTheme::PaperLight => Color32::from_rgb(172, 104, 16),
        }
    }

    pub fn err_color(&self) -> Color32 {
        match self {
            AcademicTheme::SlateDark => Color32::from_rgb(224, 98, 98),
            AcademicTheme::PaperLight => Color32::from_rgb(176, 42, 42),
        }
    }

    pub fn card_fill(&self) -> Color32 {
        match self {
            AcademicTheme::SlateDark => Color32::from_rgb(30, 35, 45),
            AcademicTheme::PaperLight => Color32::from_rgb(252, 251, 248),
        }
    }

    pub fn card_stroke(&self) -> Stroke {
        match self {
            AcademicTheme::SlateDark => Stroke::new(1.0_f32, Color32::from_rgb(55, 64, 80)),
            AcademicTheme::PaperLight => Stroke::new(1.0_f32, Color32::from_rgb(212, 209, 200)),
        }
    }
}
