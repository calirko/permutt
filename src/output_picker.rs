use gtk::prelude::*;
use gtk::{Align, Box as GBox, CheckButton, Label, Orientation};

use crate::converter::ImageOutputFormat;

#[derive(Clone)]
pub struct OutputPicker {
    container: GBox,
    buttons: Vec<(ImageOutputFormat, CheckButton)>,
}

impl OutputPicker {
    pub fn new() -> Self {
        let container = GBox::new(Orientation::Vertical, 4);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(12);
        container.set_margin_bottom(12);

        let section_label = Label::new(Some("Convert to"));
        section_label.set_halign(Align::Start);
        section_label.add_css_class("section-label");
        container.append(&section_label);

        let grid = GBox::new(Orientation::Horizontal, 8);
        grid.set_margin_top(6);

        let mut buttons = Vec::new();
        let mut first_btn: Option<CheckButton> = None;

        for fmt in ImageOutputFormat::all() {
            let btn = if let Some(ref group) = first_btn {
                CheckButton::builder()
                    .label(fmt.to_string().as_str())
                    .group(group)
                    .build()
            } else {
                CheckButton::builder()
                    .label(fmt.to_string().as_str())
                    .build()
            };

            btn.add_css_class("format-chip");

            if first_btn.is_none() {
                first_btn = Some(btn.clone());
            }

            grid.append(&btn);
            buttons.push((*fmt, btn));
        }

        container.append(&grid);

        Self { container, buttons }
    }

    pub fn widget(&self) -> &GBox {
        &self.container
    }

    pub fn selected_format(&self) -> Option<ImageOutputFormat> {
        self.buttons
            .iter()
            .find(|(_, btn)| btn.is_active())
            .map(|(fmt, _)| *fmt)
    }
}
