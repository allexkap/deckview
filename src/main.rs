use egui::{emath, pos2, Color32, Painter, Pos2, Rect, Stroke};
use rusqlite::{types, Connection};

const NAMES: [&str; 3] = ["asd", "qwe", "zxc"];

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Deckview",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

enum EventType {
    Running = 0,
    Started,
    Stopped,
    Suspended,
    Resumed,
}

impl types::FromSql for EventType {
    fn column_result(value: types::ValueRef<'_>) -> types::FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(EventType::Running),
            1 => Ok(EventType::Started),
            2 => Ok(EventType::Stopped),
            3 => Ok(EventType::Suspended),
            4 => Ok(EventType::Resumed),
            i => Err(rusqlite::types::FromSqlError::OutOfRange(i)),
        } // ._.
    }
}

struct Segments(Vec<[Pos2; 2]>);

impl std::ops::Deref for Segments {
    type Target = Vec<[Pos2; 2]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Segments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Segments {
    fn build<I>(iter: I, start_ts: u64, stop_ts: u64, x_size: u64) -> Segments
    where
        I: IntoIterator<Item = (u64, EventType)>,
    {
        let y_step = 1.0 / ((stop_ts - start_ts) as f32 / x_size as f32 - 1.0).ceil();
        let mut prev_ts = Default::default();
        let vec = iter
            .into_iter()
            .map(|(ts, ev)| {
                let pos = (ts - start_ts) as f32 / x_size as f32;
                (pos2(pos.fract(), pos.trunc() * y_step), ev)
            })
            .filter_map(|(ts, ev)| match ev {
                EventType::Running => None,
                EventType::Started | EventType::Resumed => {
                    prev_ts = ts;
                    None
                }
                EventType::Stopped | EventType::Suspended => Some([prev_ts, ts]),
            })
            .flat_map(Regen::dy(y_step))
            .collect();
        Segments { 0: vec }
    }

    fn from_points(a: Pos2, b: Pos2, dy: f32) -> Segments {
        Segments {
            0: Regen::dy(dy)([a, b]).collect(),
        }
    }
}

struct Regen {
    segment: [Pos2; 2],
    n: i32,
    i: i32,
}

impl Iterator for Regen {
    type Item = [Pos2; 2];

    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        match self.i {
            1 if self.n == 1 => Some(self.segment),
            1 => Some([self.segment[0], pos2(1.0, self.segment[0].y)]),
            i if i == self.n => Some([pos2(0.0, self.segment[1].y), self.segment[1]]),
            i if i < self.n => {
                let y =
                    (self.segment[1].y - self.segment[0].y) / (self.n - 1) as f32 * (i - 1) as f32;
                Some([pos2(0.0, y), pos2(1.0, y)])
            }
            _ => None,
        }
    }
}

impl Regen {
    fn dy(dy: f32) -> impl FnMut([Pos2; 2]) -> Regen {
        move |segment| Regen {
            segment,
            n: ((segment[1].y - segment[0].y) / dy).round() as i32 + 1,
            i: 0,
        }
    }
}

fn load_test_data() -> Segments {
    let conn = Connection::open("./deck.db").unwrap();
    let mut stmt = conn
        .prepare(
            "select timestamp, event_type from events \
            where object_id = ?1 and ?2 <= timestamp and timestamp < ?3",
        )
        .unwrap();

    let start_ts = 1738357200;
    let stop_ts = 1740776400;
    let x_size = 24 * 60 * 60;

    let iter = stmt
        .query_map((1, start_ts, stop_ts), |row: &rusqlite::Row<'_>| {
            Ok((row.get::<_, u64>(0)?, row.get::<_, EventType>(1)?))
        })
        .unwrap()
        .filter_map(Result::ok);
    Segments::build(iter, start_ts, stop_ts, x_size)
}

fn paint_segments(
    painter: &Painter,
    to_screen: &emath::RectTransform,
    segments: &Segments,
    stroke: Stroke,
) {
    segments.iter().for_each(|segment| {
        painter.line_segment(segment.map(|p| to_screen * p), stroke);
    })
}

struct MyApp {
    selected: &'static str,
    segments: Segments,
    segments_back: Segments,
}

impl Default for MyApp {
    fn default() -> Self {
        let segments = load_test_data();
        println!("{}", segments.len());
        Self {
            selected: NAMES[0],
            segments,
            segments_back: Segments::from_points(pos2(0.0, 0.0), pos2(1.0, 1.0), 1.0 / 27.0),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("my_left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("combo")
                    .selected_text(format!("{}", self.selected))
                    .show_ui(ui, |ui| {
                        for name in NAMES {
                            ui.selectable_value(&mut self.selected, name, name);
                        }
                    });
                if ui
                    .add(egui::Button::new("Close").min_size(egui::vec2(100.0, 0.0)))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_id, rect) = ui.allocate_space(ui.available_size());
            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(-0.02..=1.02, -0.02..=1.02),
                rect,
            );
            let painter = ui.painter_at(rect);

            paint_segments(
                &painter,
                &to_screen,
                &self.segments_back,
                Stroke::new(0.1, Color32::GRAY),
            );
            paint_segments(
                &painter,
                &to_screen,
                &self.segments,
                Stroke::new(5.0, Color32::RED),
            );
        });
    }
}
