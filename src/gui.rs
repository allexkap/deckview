use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewParams {
    pub range: [chrono::NaiveDate; 2],
    pub selected_app: usize,
    pub selected_app_id: u32, // :<
}

pub trait View {
    fn build(db: Rc<RefCell<crate::db::DeckDBv>>, view_params: ViewParams) -> Self
    where
        Self: Sized;
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct Viewer {
    db: Rc<RefCell<crate::db::DeckDBv>>,
    view: Box<dyn View>,
    view_params: ViewParams,
    view_params_new: ViewParams,
    apps: Vec<(u32, String)>,
}

impl Viewer {
    pub fn build(db: crate::db::DeckDBv) -> Self {
        let ref_db = Rc::new(RefCell::new(db));

        let end = chrono::Local::now().date_naive();

        let start = end - chrono::Duration::days(30);

        let params = ViewParams {
            range: [start, end],
            selected_app: 0,
            selected_app_id: 0,
        };

        let view = Box::new(crate::gui_lines::LineView::build(ref_db.clone(), params));

        Self {
            db: ref_db,
            view,
            view_params: params,
            view_params_new: params,
            apps: Vec::default(),
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::right("right")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                if self.apps.len() != 0 {
                    egui::ComboBox::from_id_salt("combo")
                        .selected_text(&self.apps[self.view_params_new.selected_app].1)
                        .truncate()
                        .show_index(
                            ui,
                            &mut self.view_params_new.selected_app,
                            self.apps.len(),
                            |i| &self.apps[i].1,
                        );
                    self.view_params_new.selected_app_id =
                        self.apps[self.view_params_new.selected_app].0;
                }
                if ui
                    .add(egui::Button::new("Close").min_size(egui::vec2(100.0, 0.0)))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| self.view.ui(ui));

        if self.view_params != self.view_params_new {
            self.view_params = self.view_params_new;
        }
    }
}
