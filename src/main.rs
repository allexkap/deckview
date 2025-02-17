use egui::{emath, pos2, Color32, Painter, Pos2, Rect, Stroke};

const NAMES: [&str; 3] = ["asd", "qwe", "zxc"];

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

fn load_test_data() -> Vec<[u64; 2]> {
    std::fs::read_to_string("data.in")
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect::<Vec<u64>>()
        .chunks(2)
        .map(|c| [c[0], c[1]])
        .collect()
}

type Segments = Vec<[Pos2; 2]>;

fn ts_to_segments(timestamps: Vec<[u64; 2]>, start_ts: u64, x_size: u32, y_count: u32) -> Segments {
    let mut lines_segments: Segments = Vec::new();

    let max_ts = start_ts + x_size as u64 * y_count as u64 - 1;

    for segment in timestamps {
        if segment[1] < segment[0] || segment[1] < start_ts || segment[0] > max_ts {
            continue;
        }

        let points = [
            std::cmp::max(segment[0], start_ts),
            std::cmp::min(segment[1], max_ts),
        ]
        .map(|ts| {
            let pos = (ts - start_ts) as f32 / x_size as f32;
            pos2(pos.fract(), pos.floor() / (y_count - 1) as f32)
        });

        if points[0].y == points[1].y {
            lines_segments.push([points[0], points[1]]);
        } else {
            lines_segments.push([points[0], pos2(1.0, points[0].y)]);
            for i in 1..((points[1].y - points[0].y) * (y_count - 1) as f32).round() as u32 {
                let y = points[0].y + i as f32 / (y_count - 1) as f32;
                lines_segments.push([pos2(0.0, y), pos2(1.0, y)]);
            }
            lines_segments.push([pos2(0.0, points[1].y), points[1]]);
        }
    }

    lines_segments
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
        let segments = ts_to_segments(load_test_data(), 1738357200, 24 * 60 * 60, 31)
            .into_iter()
            .filter(|[a, b]| a.distance_sq(*b) > 0.00001)
            .collect::<Segments>();
        println!("{}", segments.len());
        Self {
            selected: NAMES[0],
            segments,
            segments_back: ts_to_segments(vec![[0, 24 * 60 * 60 * 31]], 0, 24 * 60 * 60, 31),
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
