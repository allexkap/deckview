use crate::{db::DeckDBv, widgets::DateSelector};
use chrono::NaiveDate;
use log::info;
use std::{cell::RefCell, rc::Rc};
mod grid;
mod line;

pub use grid::GridView;
pub use line::LineView;

const PADDING: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewParams {
    pub range: [NaiveDate; 2],
    pub selected_app: usize,
    pub selected_app_id: u32, // :<
}

pub trait View {
    fn build(db: Rc<RefCell<DeckDBv>>, view_params: ViewParams) -> Self
    where
        Self: Sized;
    fn update(&mut self, view_params: ViewParams);
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct Viewer {
    db: Rc<RefCell<DeckDBv>>,

    view: Box<dyn View>,
    view_type_id: usize,
    view_params: ViewParams,

    date_mode_id: usize,

    apps: Vec<(u32, String)>,
}

impl Viewer {
    pub fn build(db: DeckDBv) -> Self {
        let ref_db = Rc::new(RefCell::new(db));

        let end = chrono::Local::now().date_naive();
        let start = end - chrono::Duration::days(14);

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
            date_mode_id: 0,
            apps,
        }
    }

    fn select_view_type(&mut self, ui: &mut egui::Ui) {
        const VIEW_TITLES: [&str; 2] = ["Grid", "Lines"];

        ui.heading("View type");
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
        ui.heading("App");
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

    fn select_date(&mut self, ui: &mut egui::Ui) {
        const DATE_MODES: [&str; 3] = ["Day", "Week", "Month"];

        ui.heading("Date");

        ui.label("From");
        ui.add(DateSelector::new(&mut self.view_params.range[0]));
        ui.add_space(PADDING);

        ui.label("To");
        ui.add(DateSelector::new(&mut self.view_params.range[1]));
    }

    fn close_btn(&mut self, ui: &mut egui::Ui) {
        ui.heading("Actions");
        if ui
            .add(egui::Button::new("Update").min_size(egui::vec2(100.0, 0.0)))
            .clicked()
        {
            self.view.update(self.view_params);
        }
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
                ui.add_space(6.0);
                self.select_view_type(ui);
                ui.add_space(PADDING);
                self.select_date(ui);
                ui.add_space(PADDING);
                self.select_app(ui);
                ui.add_space(PADDING);
                self.close_btn(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| self.view.ui(ui));
    }
}
