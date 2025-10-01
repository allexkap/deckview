use crate::{
    db::{DeckDBv, Sessions},
    views::{View, ViewParams},
};
use egui::{Color32, Painter, Pos2, Rect, Stroke, emath::RectTransform, pos2};
use std::{cell::RefCell, rc::Rc};

pub struct LineView {
    db: Rc<RefCell<DeckDBv>>,
    view_params: ViewParams,
    foreground: Segments,
    background: Segments,
}

impl View for LineView {
    fn build(db: Rc<RefCell<DeckDBv>>, view_params: ViewParams) -> Self {
        let mut view = Self {
            db,
            view_params,
            foreground: Default::default(),
            background: Default::default(),
        };
        view.load_data();
        view
    }

    fn update(&mut self, view_params: ViewParams) {
        self.view_params = view_params;
        self.load_data();
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let (_id, rect) = ui.allocate_space(ui.available_size());
        let to_screen =
            RectTransform::from_to(Rect::from_x_y_ranges(-0.04..=1.02, -0.02..=1.02), rect);
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
                egui::FontId::default(),
                Color32::GRAY,
            );
        }

        let total_days: usize = (self.view_params.range[1] - self.view_params.range[0])
            .num_days()
            .try_into()
            .unwrap();
        let step = total_days / 40;
        for (i, day) in self.view_params.range[0]
            .iter_days()
            .take(total_days)
            .enumerate()
        {
            if step != 0 && i % (step + 1) != 0 {
                continue;
            }
            painter.text(
                to_screen * pos2(0.0, i as f32 / (total_days - 1) as f32),
                egui::Align2::RIGHT_CENTER,
                day.format("%d.%m  "),
                egui::FontId::default(),
                Color32::GRAY,
            );
        }
    }
}

impl LineView {
    fn load_data(&mut self) {
        let [start_ts, stop_ts] = self
            .view_params
            .range
            .map(|x| x.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64);
        let x_size = 24 * 60 * 60;

        let sessions = self
            .db
            .borrow_mut()
            .load_sessions(self.view_params.selected_app_id, start_ts, stop_ts)
            .unwrap();
        self.foreground = Segments::build(sessions, start_ts, stop_ts, x_size);

        let dy = 1.0 / ((stop_ts - start_ts) as f32 / x_size as f32 - 1.0).ceil();
        self.background = Segments::from_points(pos2(0.0, 0.0), pos2(1.0, 1.0), dy);
    }
}

#[derive(Default, derive_more::Deref, derive_more::DerefMut)]
struct Segments(Vec<[Pos2; 2]>);

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
