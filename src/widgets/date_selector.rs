use chrono::{Datelike, Days, Months, NaiveDate};
use egui::{Button, Label};
use egui_flex::{Flex, item};

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
        Flex::horizontal()
            .w_full()
            .show(ui, |flex| {
                flex.add_flex(item().grow(1.0), Flex::vertical(), |flex| {
                    if flex.add(item(), Button::new("+")).clicked() {
                        if let Some(date) = self.date.checked_add_days(Days::new(1)) {
                            *self.date = date;
                        }
                    }
                    flex.add(item(), Label::new(format!("{:02}", self.date.day())));
                    if flex.add(item(), Button::new("-")).clicked() {
                        if let Some(date) = self.date.checked_sub_days(Days::new(1)) {
                            *self.date = date;
                        }
                    }
                });
                flex.add_flex(item().grow(1.0), Flex::vertical(), |flex| {
                    if flex.add(item(), Button::new("+")).clicked() {
                        if let Some(date) = self.date.checked_add_months(Months::new(1)) {
                            *self.date = date;
                        }
                    }
                    flex.add(item(), Label::new(format!("{:02}", self.date.month())));
                    if flex.add(item(), Button::new("-")).clicked() {
                        if let Some(date) = self.date.checked_sub_months(Months::new(1)) {
                            *self.date = date;
                        }
                    }
                });
                flex.add_flex(item().grow(1.0), Flex::vertical(), |flex| {
                    if flex.add(item(), Button::new("+")).clicked() {
                        let (flag, years) = self.date.year_ce();
                        if flag {
                            if let Some(date) = self.date.with_year((years + 1).try_into().unwrap())
                            {
                                *self.date = date;
                            }
                        }
                    }
                    flex.add(item(), Label::new(format!("{:02}", self.date.year_ce().1)));
                    if flex.add(item(), Button::new("-")).clicked() {
                        let (flag, years) = self.date.year_ce();
                        if flag {
                            if let Some(date) = self.date.with_year((years - 1).try_into().unwrap())
                            {
                                *self.date = date;
                            }
                        }
                    }
                });
            })
            .response
    }
}
