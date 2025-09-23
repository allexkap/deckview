use log::info;
use std::{cell::RefCell, rc::Rc};

use crate::{gui_grid::GridView, gui_lines::LineView};

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
    fn update(&mut self, view_params: crate::gui::ViewParams);
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct Viewer {
    db: Rc<RefCell<crate::db::DeckDBv>>,

    view: Box<dyn View>,
    view_type_id: usize,

    view_params: ViewParams,

    apps: Vec<(u32, String)>,
}

impl Viewer {
    pub fn build(db: crate::db::DeckDBv) -> Self {
        let ref_db = Rc::new(RefCell::new(db));

        let end = chrono::Local::now().date_naive();
        let start = end - chrono::Duration::days(28);

        let apps = ref_db.borrow().load_apps().unwrap();
        let selected_app = 0;

        let params = ViewParams {
            range: [start, end],
            selected_app,
            selected_app_id: apps[selected_app].0,
        };

        let view_type_id = 0;
        let view = Box::new(GridView::build(ref_db.clone(), params));

        Self {
            db: ref_db,
            view,
            view_type_id,
            view_params: params,
            apps,
        }
    }

    fn select_view_type(&mut self, ui: &mut egui::Ui) {
        const VIEW_TITLES: [&str; 2] = ["Grid", "Lines"];
        if !egui::ComboBox::from_id_salt("select_view_type")
            .selected_text(VIEW_TITLES[self.view_type_id])
            .show_index(ui, &mut self.view_type_id, VIEW_TITLES.len(), |i| {
                VIEW_TITLES[i]
            })
            .changed()
        {
            return;
        }

        info!("reload view (id={})", self.view_type_id);
        self.view = match self.view_type_id {
            0 => Box::new(GridView::build(self.db.clone(), self.view_params)),
            1 => Box::new(LineView::build(self.db.clone(), self.view_params)),
            i => panic!("incorrect view_type_id={i}"),
        }
    }

    fn select_app(&mut self, ui: &mut egui::Ui) {
        if !egui::ComboBox::from_id_salt("select_app")
            .selected_text(&self.apps[self.view_params.selected_app].1)
            .truncate()
            .show_index(
                ui,
                &mut self.view_params.selected_app,
                self.apps.len(),
                |i| &self.apps[i].1,
            )
            .changed()
        {
            return;
        }
        self.view_params.selected_app_id = self.apps[self.view_params.selected_app].0;
        self.view.update(self.view_params);
    }

    fn close_btn(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(egui::Button::new("Close").min_size(egui::vec2(100.0, 0.0)))
            .clicked()
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("right")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                self.select_view_type(ui);
                self.select_app(ui);
                self.close_btn(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| self.view.ui(ui));
    }
}
