use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, pos2};

use crate::db::{AppId, DeckDBv, Sessions};

pub struct Viewer {
    db: DeckDBv,
    apps: Vec<(AppId, String)>,
    loaded: usize,
    selected: usize,
    range: [u64; 3],
    foreground: Segments,
    background: Segments,
}

impl Default for Viewer {
    fn default() -> Self {
        let db = DeckDBv::open("./deck.db").unwrap();
        let apps = db.load_apps().unwrap();

        Self {
            db,
            apps,
            loaded: 1, // != selected
            selected: 0,
            range: [1738357200, 1740776400, 24 * 60 * 60],
            foreground: Default::default(),
            background: Default::default(),
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.loaded != self.selected {
            self.loaded = self.selected;
            self.load_data();
        }

        egui::SidePanel::right("my_left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("combo")
                    .selected_text(&self.apps[self.selected].1)
                    .truncate()
                    .show_index(ui, &mut self.selected, self.apps.len(), |i| &self.apps[i].1);
                if ui
                    .add(egui::Button::new("Close").min_size(egui::vec2(100.0, 0.0)))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (_id, rect) = ui.allocate_space(ui.available_size());
            let to_screen = egui::emath::RectTransform::from_to(
                Rect::from_x_y_ranges(-0.02..=1.02, -0.02..=1.02),
                rect,
            );
            let painter = ui.painter_at(rect);

            self.foreground
                .paint(&painter, &to_screen, Stroke::new(5.0, Color32::RED));
            self.background
                .paint(&painter, &to_screen, Stroke::new(0.1, Color32::GRAY));

            // paint vertical lines
            let stroke = Stroke::new(0.1, Color32::GRAY);
            for i in [6, 12, 18] {
                let x = i as f32 / 24.0;
                dashed_line_segment(
                    &painter,
                    [1.0, 0.0].map(|y| to_screen * pos2(x, y)),
                    stroke,
                    4.0,
                    4.0,
                );

                painter.text(
                    to_screen * pos2(x, 1.0),
                    egui::Align2::CENTER_TOP,
                    format!("{i}"),
                    FontId::default(),
                    Color32::GRAY,
                );
            }

            // print day numbers
            let n = 28; // todo
            for i in 0..n {
                painter.text(
                    to_screen * pos2(0.0, i as f32 / (n - 1) as f32),
                    egui::Align2::RIGHT_CENTER,
                    format!("{}  ", i + 1),
                    FontId::default(),
                    Color32::GRAY,
                );
            }
        });
    }
}

impl Viewer {
    fn load_data(&mut self) {
        let [start_ts, stop_ts, x_size] = self.range;

        let sessions = self
            .db
            .load_sessions(self.apps[self.loaded as usize].0, start_ts, stop_ts)
            .unwrap();
        self.foreground = Segments::build(sessions, start_ts, stop_ts, x_size);

        let dy = 1.0 / ((stop_ts - start_ts) as f32 / x_size as f32 - 1.0).ceil();
        self.background = Segments::from_points(pos2(0.0, 0.0), pos2(1.0, 1.0), dy);
    }
}

// ----------------------- PRIMITIVES ----------------------- //

struct Segments(Vec<[Pos2; 2]>);

impl Default for Segments {
    fn default() -> Self {
        Self(Default::default())
    }
}

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
    fn build(sessions: Sessions, start_ts: u64, stop_ts: u64, x_size: u64) -> Segments {
        let y_step = 1.0 / ((stop_ts - start_ts) as f32 / x_size as f32 - 1.0).ceil();

        let segments = sessions
            .iter()
            .map(|session| {
                session.map(|ts| {
                    let pos = (ts - start_ts) as f32 / x_size as f32;
                    pos2(pos.fract(), pos.trunc() * y_step)
                })
            })
            .flat_map(Regen::dy(y_step))
            .collect();

        Segments { 0: segments }
    }

    fn from_points(a: Pos2, b: Pos2, dy: f32) -> Segments {
        Segments {
            0: Regen::dy(dy)([a, b]).collect(),
        }
    }

    fn paint(&self, painter: &Painter, to_screen: &egui::emath::RectTransform, stroke: Stroke) {
        self.0.iter().for_each(|segment| {
            painter.line_segment(segment.map(|p| to_screen * p), stroke);
        })
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

fn dashed_line_segment(
    painter: &Painter,
    points: [Pos2; 2],
    stroke: impl Into<Stroke>,
    dash_length: f32,
    gap_length: f32,
) {
    let line_vec = points[1] - points[0];
    let length = line_vec.length();
    let dir = line_vec.normalized();
    let stroke = stroke.into();

    let dash = dir * dash_length;
    let gap = dir * gap_length;

    let mut pos = points[0];
    for _ in 0..(length / (dash_length + gap_length)) as i64 {
        let dash_end_pos = pos + dash;
        painter.line_segment([pos, dash_end_pos], stroke);
        pos = dash_end_pos + gap;
    }
    painter.line_segment([pos, points[1]], stroke);
}
