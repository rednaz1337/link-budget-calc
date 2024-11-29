use std::collections::HashMap;
use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, Vec2, ViewportBuilder};
use egui::{Context, DragValue, Slider, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};
use number_prefix::{NumberPrefix, Prefix};
use std::error::Error;
use std::fmt::format;
use egui::UiKind::ScrollArea;

mod asynch;
mod calc;

fn main() {
    let viewport_builder = ViewportBuilder::default().with_inner_size(Vec2::new(420.0, 600.0));
    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Link Budget Calculator",
        native_options,
        Box::new(|cc| LinkBudgetApp::new(cc)),
    )
        .unwrap();
}

#[derive(Default, PartialEq, Eq)]
enum CalculationTarget {
    #[default]
    Snr,
    Distance,
    TxPower,
}

struct LinkBudgetApp {
    temperature: f64,  // Kelvin
    frequency: f64,    // Hertz
    bandwidth: f64,    // Hertz
    snr: f64,      // dB

    tx_power: f64, // dBm

    distance: f64, // meter
    d_break: f64,  // meter
    break_exponent: f64,

    losses: HashMap<String, f64>,
    loss_name: String,
    loss_db: f64,

    gains: HashMap<String, f64>,
    gain_name: String,
    gain_db: f64,

    calculation_target: CalculationTarget,
}

impl Default for LinkBudgetApp {
    fn default() -> Self {
        Self {
            temperature: 290.0,
            bandwidth: 20e6,
            snr: 10.0,
            frequency: 2.4e9,
            tx_power: 30.0,
            distance: 2000.0,
            d_break: 500.0,
            break_exponent: 4.3,
            losses: HashMap::default(),
            loss_name: String::default(),
            loss_db: 10.0,
            gains: HashMap::new(),
            gain_name: String::new(),
            gain_db: 10.0,
            calculation_target: CalculationTarget::default(),
        }
    }
}
impl LinkBudgetApp {
    pub fn new(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
        Ok(Box::new(Self::default()))
    }

    pub fn total_losses(&self) -> f64 {
        self.losses.iter().map(|(_, l)| *l).sum()
    }

    pub fn total_gains(&self) -> f64 {
        self.gains.iter().map(|(_, g)| *g).sum()
    }


    pub fn total_sum(&self) -> f64 {
        let thermal =
            calc::watt_to_dbm(calc::thermal_noise_power(self.temperature, self.bandwidth));
        let losses = self.total_losses();
        let gains = self.total_gains();
        let path = calc::friis::path_loss(self.distance, self.d_break, self.frequency, self.break_exponent);

        let negative =
            thermal
                + losses
                + path
                + self.snr;
        let positive = self.tx_power + gains;

        return positive - negative;
    }

    fn ui_base_info(&mut self, ui: &mut Ui) {
        frame_styled(ui).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Base Info");
                egui::Grid::new("base_data").num_columns(2).show(ui, |ui| {
                    ui.label("Temperature");
                    ui.add(DragValue::new(&mut self.temperature).suffix(" K"));
                    ui.end_row();

                    ui.label("Bandwidth");
                    ui.add(
                        prefix_drag_value(&mut self.bandwidth)
                            .suffix("Hz")
                            .range(0.0..=f64::MAX)
                            .speed(1e6),
                    );
                    ui.end_row();

                    let thermal_noise_floor = calc::watt_to_dbm(calc::thermal_noise_power(self.temperature, self.bandwidth));
                    ui.label("Thermal noise floor");
                    ui.label(format!("{thermal_noise_floor:.1} dBm"));
                    ui.end_row();

                    ui.label("Frequency");
                    ui.add(
                        prefix_drag_value(&mut self.frequency)
                            .suffix("Hz")
                            .range(0.0..=f64::MAX)
                            .speed(1e6),
                    );
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::Snr,
                        "SNR",
                    );
                    ui.add(DragValue::new(&mut self.snr).suffix(" dB"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::TxPower,
                        "Tx Power",
                    );
                    ui.add(DragValue::new(&mut self.tx_power).suffix(" dBm"));
                    ui.end_row();

                    ui.label("Rx Power");
                    let rx_power = self.snr + thermal_noise_floor;
                    ui.label(format!("{rx_power:.1} dBm"))
                })
            });
        });
    }

    fn ui_path_loss(&mut self, ui: &mut Ui) {
        frame_styled(&ui).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Free Space Path loss");
                egui::Grid::new("path_loss").show(ui, |ui| {
                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::Distance,
                        "Distance",
                    );
                    ui.add(DragValue::new(&mut self.distance).suffix(" m"));
                    ui.end_row();

                    ui.label("break distance");
                    ui.add(DragValue::new(&mut self.d_break).suffix(" m"));
                    ui.end_row();

                    ui.label("break exponent");
                    ui.add(DragValue::new(&mut self.break_exponent));
                    ui.end_row();

                    let path_loss = calc::friis::path_loss(self.distance, self.d_break, self.frequency, self.break_exponent);
                    ui.label("Path Loss");
                    ui.label(format!("{path_loss:.1} dBm"));
                    ui.end_row();
                });
            });
        });

    }
}

impl eframe::App for LinkBudgetApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let total_db = self.total_sum();
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.ui_base_info(ui);
                self.ui_path_loss(ui);
            });
            frame_styled(ui).show(ui, |ui| {
                ui.heading("Gains");
                ui.horizontal(|ui| {
                    let name_response =
                        ui.add(TextEdit::singleline(&mut self.gain_name).hint_text("Gain Name"));
                    ui.add(DragValue::new(&mut self.gain_db).suffix(" dB"));
                    if ui.button("Add").clicked()
                        || (name_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.gain_name.trim().is_empty() {
                            self.gains.insert(self.gain_name.clone(), self.gain_db);
                            self.gain_name.clear();
                        }
                    }
                });
                ui.separator();
                TableBuilder::new(ui)
                    .id_salt("gain_table")
                    .striped(true)
                    .column(Column::exact(20.0))
                    .column(Column::remainder())
                    .column(Column::exact(100.0))
                    .header(20., |mut header| {
                        header.col(|ui| {
                            ui.label(" ");
                        });
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        self.gains.retain(|name, gain| {
                            let mut retain = true;
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("X").clicked() {
                                        retain = false;
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(name.as_str());
                                });
                                row.col(|ui| {
                                    ui.add(DragValue::new(gain).suffix(" dB"));
                                });
                            });
                            retain
                        });
                    });
            });
            frame_styled(ui).show(ui, |ui| {
                ui.heading("Losses");
                ui.horizontal(|ui| {
                    let name_response =
                        ui.add(TextEdit::singleline(&mut self.loss_name).hint_text("Loss Name"));
                    ui.add(DragValue::new(&mut self.loss_db).suffix(" dB"));
                    if ui.button("Add").clicked()
                        || (name_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.loss_name.trim().is_empty() {
                            self.losses.insert(self.loss_name.clone(), self.loss_db);
                            self.loss_name.clear();
                        }
                    }
                });
                ui.separator();
                TableBuilder::new(ui)
                    .id_salt("loss_table")
                    .striped(true)
                    .column(Column::exact(20.0))
                    .column(Column::remainder())
                    .column(Column::exact(100.0))
                    .header(20., |mut header| {
                        header.col(|ui| {
                            ui.label(" ");
                        });
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        self.losses.retain(|name, loss| {
                            let mut retain = true;
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("X").clicked() {
                                        retain = false;
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(name.as_str());
                                });
                                row.col(|ui| {
                                    ui.add(DragValue::new(loss).suffix(" dB"));
                                });
                            });
                            retain
                        });
                    });
            });
        });

        if total_db.is_infinite() || total_db.is_nan() {
            return;
        }

        match self.calculation_target {
            CalculationTarget::Snr => {
                self.snr += total_db;
            }
            CalculationTarget::Distance => {
                let new_path_loss = calc::friis::path_loss(self.distance, self.d_break, self.frequency, self.break_exponent) + total_db;
                self.distance = calc::friis::distance(new_path_loss, self.d_break, self.frequency, self.break_exponent);
            }
            CalculationTarget::TxPower => {
                self.tx_power -= total_db;
            }
        }
    }
}

fn frame_styled(ui: &Ui) -> egui::Frame {
    egui::Frame::default()
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .rounding(ui.visuals().widgets.noninteractive.rounding)
        .inner_margin(5.0)
        .outer_margin(5.0)
}
fn prefix_drag_value(value: &mut f64) -> DragValue {
    DragValue::new(value)
        .custom_formatter(
            |value, range| match number_prefix::NumberPrefix::decimal(value) {
                NumberPrefix::Standalone(num) => {
                    format!("{num} ")
                }
                NumberPrefix::Prefixed(prefix, num) => {
                    format!("{:.1} {}", num, prefix)
                }
            },
        )
        .custom_parser(|value| {
            let Ok(number_prefix) = value.parse::<NumberPrefix<f64>>() else {
                return None;
            };

            return match number_prefix {
                NumberPrefix::Standalone(number) => Some(number),
                NumberPrefix::Prefixed(prefix, number) => {
                    let factor = match prefix {
                        Prefix::Kilo => 1e3,
                        Prefix::Mega => 1e6,
                        Prefix::Giga => 1e9,
                        Prefix::Tera => 1e12,
                        Prefix::Peta => 1e15,
                        Prefix::Exa => 1e18,
                        Prefix::Zetta => 1e21,
                        Prefix::Yotta => 1e24,
                        _ => return None,
                    };

                    Some(factor * number)
                }
            };
        })
}
