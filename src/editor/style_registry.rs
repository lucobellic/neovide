use std::{collections::HashMap, sync::Arc};

use super::{style::Style, ColorOpacity};

type StyleId = u64;
type ColorId = u64;

/// The `StyleRegistry` struct is responsible for keeping styles updated with corresponding opacity settings.
/// Styles and opacities are associated with background and foreground colors.
#[derive(Default)]
pub struct StyleRegistry {
    /// Maps style IDs (neovim highlight table id) to their corresponding styles
    defined_styles: HashMap<StyleId, Arc<Style>>,

    /// Associates each color with opacity settings.
    /// This is used to update the opacity of all styles when the global opacity changes.
    defined_opacities: HashMap<ColorId, ColorOpacity>,

    /// Maps background colors to their corresponding style IDs
    background_color_style_map: HashMap<ColorId, Vec<StyleId>>,

    /// Maps foreground colors to their corresponding style IDs
    foreground_color_style_map: HashMap<ColorId, Vec<StyleId>>,
}

impl StyleRegistry {
    pub fn new() -> Self {
        Self {
            defined_opacities: HashMap::new(),
            defined_styles: HashMap::new(),
            background_color_style_map: HashMap::new(),
            foreground_color_style_map: HashMap::new(),
        }
    }

    pub fn default_style(&self) -> Option<Style> {
        self.defined_styles.get(&0).map(|style| (**style).clone())
    }

    pub fn defined_styles(&self) -> &HashMap<u64, Arc<Style>> {
        &self.defined_styles
    }

    pub fn set_style(&mut self, mut style: Style, id: u64, default_opacity: f32) {
        self.update_style_opacities_from_existing_mapping(&mut style, default_opacity);
        self.update_color_to_style_mapping(&style, id);
        self.defined_styles.insert(id, Arc::new(style));
    }

    /// Set the foreground and background opacity of a color and update all styles that use this color
    pub fn set_opacity(
        &mut self,
        color: ColorId,
        color_opacity: ColorOpacity,
        default_opacity: f32,
    ) {
        // Update the opacity of all styles that use this color
        let mut update_opacity =
            |styles_map: &HashMap<ColorId, Vec<StyleId>>,
             set_opacity_fn: fn(&mut Style, &ColorOpacity, f32)| {
                if let Some(styles_id) = styles_map.get(&color) {
                    styles_id.iter().for_each(|id| {
                        if let Some(arc) = self.defined_styles.get(id) {
                            let mut style = (**arc).to_owned();
                            set_opacity_fn(&mut style, &color_opacity, default_opacity);
                            self.defined_styles.insert(*id, Arc::new(style));
                        }
                    });
                }
            };

        update_opacity(
            &self.background_color_style_map,
            Style::set_background_opacity,
        );
        update_opacity(
            &self.foreground_color_style_map,
            Style::set_foreground_opacity,
        );

        self.defined_opacities.insert(color, color_opacity);
    }

    /// Update all styles with existing color opacity settings with updated global opacity
    pub fn update_all_styles_opacity(&mut self, default_opacity: f32) {
        let get_updated_styles =
            |color_style_map: &HashMap<ColorId, Vec<StyleId>>,
             set_opacity_fn: fn(&mut Style, &ColorOpacity, f32)| {
                color_style_map
                    .iter()
                    .filter_map(|(color, style_ids)| {
                        self.defined_opacities.get(color).map(|color_opacity| {
                            style_ids
                                .iter()
                                .filter_map(|id| self.defined_styles.get_key_value(id))
                                .map(|(id, arc)| {
                                    let mut style = (**arc).to_owned();
                                    set_opacity_fn(&mut style, color_opacity, default_opacity);
                                    (*id, style)
                                })
                        })
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            };

        let new_background_styles = get_updated_styles(
            &self.background_color_style_map,
            Style::set_background_opacity,
        );

        let new_foreground_styles = get_updated_styles(
            &self.foreground_color_style_map,
            Style::set_foreground_opacity,
        );

        new_background_styles.iter().for_each(|(id, style)| {
            self.defined_styles.insert(*id, Arc::new(style.clone()));
        });

        new_foreground_styles.iter().for_each(|(id, style)| {
            self.defined_styles.insert(*id, Arc::new(style.clone()));
        });
    }

    /// Updates the opacity of the background and foreground style based on an existing opacity mapping.
    /// This function should be called when a style is defined or opacity changes.
    fn update_style_opacities_from_existing_mapping(&mut self, style: &mut Style, opacity: f32) {
        if let Some(o) = style.bg().and_then(|bg| self.defined_opacities.get(&bg)) {
            style.set_background_opacity(o, opacity);
        }

        if let Some(o) = style.fg().and_then(|fg| self.defined_opacities.get(&fg)) {
            style.set_foreground_opacity(o, opacity);
        }
    }

    /// Add style id in the background and foreground mapping with corresponding color.
    /// Should be called when a new style is defined
    fn update_color_to_style_mapping(&mut self, style: &Style, id: StyleId) {
        if let Some(color) = style.bg() {
            self.background_color_style_map
                .entry(color)
                .or_default()
                .push(id);
        }

        if let Some(color) = style.fg() {
            self.foreground_color_style_map
                .entry(color)
                .or_default()
                .push(id);
        }
    }
}
