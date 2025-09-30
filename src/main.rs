mod db;
mod views;
mod widgets;

fn main() -> eframe::Result {
    env_logger::init();

    let db = crate::db::DeckDBv::open("./deck.db").unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Deckview",
        options,
        Box::new(|_cc| Ok(Box::new(views::Viewer::build(db)))),
    )
}
