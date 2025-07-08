use std::{cell::RefCell, rc::Rc};

use rand::Rng;

struct Cell {
    value: f32,
    tooltip: String,
}

pub struct GridView {
    db: Rc<RefCell<crate::db::DeckDBv>>,
    view_params: crate::gui::ViewParams,
    content: Vec<Vec<Cell>>,
    cell_size: f32,
}

impl Default for GridView {
    fn default() -> Self {
        let cell_size = 40.0;
        let mut rng = rand::rng();

        Self {
            db: Default::default(),
            view_params: Default::default(),
            cell_size,
            content: (0..20)
                .map(|x| {
                    (0..7)
                        .map(|y| Cell {
                            value: rng.random_range(0.0..1.0),
                            tooltip: format!("x={x}, y={y}"),
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

impl crate::gui::View for GridView {
    fn build(db: Rc<RefCell<crate::db::DeckDBv>>, view_params: crate::gui::ViewParams) -> Self {
        Default::default()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for row in self.content.iter() {
                ui.vertical(|ui| {
                    for cell in row.iter() {
                        ui.horizontal(|ui| {
                            let desired_size = [self.cell_size, self.cell_size].into();
                            let response = ui.allocate_response(desired_size, egui::Sense::hover());

                            let color = egui::Color32::GREEN.gamma_multiply(cell.value);

                            if response.hovered() {
                                response.show_tooltip_text(&cell.tooltip);
                            }

                            let painter = ui.painter_at(response.rect);
                            painter.rect_filled(response.rect, 10.0, color);
                        });
                    }
                });
            }
        });
    }
}
