use std::collections::HashMap;

use raqote::SolidSource;

use crate::parser::{ColorRgb, StyleExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    pub on_color: ColorRgb,
    pub color: ColorRgb,
    pub bold: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: (255, 255, 255),
            on_color: (255, 0, 0),
            bold: false,
        }
    }
}

impl Style {
    pub fn on(&self) -> SolidSource {
        SolidSource::from_unpremultiplied_argb(
            255,
            self.on_color.0,
            self.on_color.1,
            self.on_color.2,
        )
    }

    pub fn off(&self) -> SolidSource {
        SolidSource::from_unpremultiplied_argb(255, self.color.0, self.color.1, self.color.2)
    }

    pub fn extend(&mut self, other: &Style) {
        self.on_color = other.on_color;
        self.color = other.color;
        self.bold = other.bold;
    }

    pub fn decode<'a>(styles: &HashMap<String, Style>, exprs: &Vec<StyleExpr<'a>>) -> Self {
        let mut me = Self::default();

        for expr in exprs {
            match expr {
                StyleExpr::Extend { name } => {
                    if let Some(s) = styles.get(*name) {
                        me.extend(s);
                    } else if *name != "default" {
                        // TODO: error handling
                    }
                }

                StyleExpr::Color { value } => me.color = *value,
                StyleExpr::OnColor { value } => me.on_color = *value,
                StyleExpr::Bold { value } => me.bold = *value,
                StyleExpr::Comment => {}
            }
        }

        me
    }
}
