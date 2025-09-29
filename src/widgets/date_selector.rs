use chrono::{Datelike, Days, Months, NaiveDate};

pub struct DateSelector<'a> {
    date: &'a mut NaiveDate,
}

impl<'a> DateSelector<'a> {
    pub fn new(date: &'a mut NaiveDate) -> DateSelector<'a> {
        DateSelector { date }
    }
}

impl<'a> egui::Widget for DateSelector<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.monospace("fuck");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button(egui::RichText::new("+")).clicked() {
                    if let Some(date) = self.date.checked_add_days(Days::new(1)) {
                        *self.date = date;
                    }
                };
                ui.label(self.date.day().to_string());
                if ui.button(egui::RichText::new("-")).clicked() {
                    if let Some(date) = self.date.checked_sub_days(Days::new(1)) {
                        *self.date = date;
                    }
                };
            });
            ui.vertical(|ui| {
                if ui.button(egui::RichText::new("+")).clicked() {
                    if let Some(date) = self.date.checked_add_months(Months::new(1)) {
                        *self.date = date;
                    }
                };
                ui.label(self.date.month().to_string());
                if ui.button(egui::RichText::new("-")).clicked() {
                    if let Some(date) = self.date.checked_sub_months(Months::new(1)) {
                        *self.date = date;
                    }
                };
            });
            ui.vertical(|ui| {
                if ui.button(egui::RichText::new("+")).clicked() {
                    if let Some(date) = self
                        .date
                        .with_year((self.date.year_ce().1 + 1).try_into().unwrap())
                    {
                        *self.date = date;
                    }
                };
                ui.label(self.date.year_ce().1.to_string());
                if ui.button(egui::RichText::new("-")).clicked() {
                    if let Some(date) = self
                        .date
                        .with_year((self.date.year_ce().1 - 1).try_into().unwrap())
                    {
                        *self.date = date;
                    }
                }
            });
        })
        .response
    }
}
