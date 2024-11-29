use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use eframe::{App, CreationContext, Frame, Storage};
use egui::{CentralPanel, Context, DragValue, TextEdit, ThemePreference, Ui};
use egui_extras::{Column, TableBuilder};
use number_prefix::{NumberPrefix, Prefix};
use serde::{Deserialize, Serialize};
use crate::calc;

#[derive(Default, PartialEq, Eq, Serialize, Deserialize)]
enum CalculationTarget {
    #[default]
    Snr,
    Distance,
    TxPower,
}

#[derive(Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
enum PowerUnit {
    #[default]
    DbMilliwatt,
    DbWatt,
    Milliwatt,
    Watt,
}

impl Display for PowerUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerUnit::DbMilliwatt => { write!(f, "dBm") }
            PowerUnit::DbWatt => { write!(f, "dBW") }
            PowerUnit::Milliwatt => { write!(f, "mW") }
            PowerUnit::Watt => { write!(f, "W") }
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Power {
    pub val_dbm: f64,
    pub unit: PowerUnit,
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.val_dbm, self.unit)
    }
}

impl Power {

    pub fn value_selector_ui(&mut self, ui: &mut Ui) {
        let mut val_unit = self.get_in_unit();
        ui.add(DragValue::new(&mut val_unit));
        self.value_from_unit(val_unit);
    }

    pub fn unit_selector_ui(&mut self, id_salt: &str, ui: &mut Ui) {
            egui::ComboBox::new(id_salt, "").width(60.0)
                .selected_text(self.unit.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.unit, PowerUnit::DbMilliwatt, "dBm");
                    ui.selectable_value(&mut self.unit, PowerUnit::DbWatt, "dBW");
                    ui.selectable_value(&mut self.unit, PowerUnit::Milliwatt, "mW");
                    ui.selectable_value(&mut self.unit, PowerUnit::Watt, "W");
                });
    }

    pub fn get_in_unit(&self) -> f64 {
        match self.unit {
            PowerUnit::DbMilliwatt => { self.val_dbm }
            PowerUnit::DbWatt => { calc::dbm_to_dbw(self.val_dbm) }
            PowerUnit::Milliwatt => { calc::dbm_to_milliwat(self.val_dbm) }
            PowerUnit::Watt => { calc::dbm_to_watt(self.val_dbm) }
        }
    }

    pub fn value_from_unit(&mut self, val_unit: f64) {
        self.val_dbm = match self.unit {
            PowerUnit::DbMilliwatt => { val_unit }
            PowerUnit::DbWatt => { calc::dbw_to_dbm(val_unit) }
            PowerUnit::Milliwatt => { calc::milliwatt_to_dbm(val_unit) }
            PowerUnit::Watt => { calc::watt_to_dbm(val_unit) }
        };
    }
}

#[derive(Serialize, Deserialize)]
pub struct LinkBudgetApp {
    temperature: f64,  // Kelvin
    frequency: f64,    // Hertz
    bandwidth: f64,    // Hertz
    snr: f64,      // dB

    tx_power: Power,
    rx_power: Power,

    distance: f64, // meter
    d_break: f64,  // meter
    break_exponent: f64,

    losses: HashMap<String, f64>,
    loss_name: String,

    gains: HashMap<String, f64>,
    gain_name: String,

    calculation_target: CalculationTarget,
}

impl Default for LinkBudgetApp {
    fn default() -> Self {
        Self {
            temperature: 290.0,
            bandwidth: 20e6,
            snr: 10.0,
            frequency: 2.4e9,
            tx_power: Power::default(),
            rx_power: Power::default(),
            distance: 2000.0,
            d_break: 500.0,
            break_exponent: 4.3,
            losses: HashMap::default(),
            loss_name: String::default(),
            gains: HashMap::new(),
            gain_name: String::new(),
            calculation_target: CalculationTarget::default(),
        }
    }
}
impl LinkBudgetApp {
    pub fn new(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
        if let Some(storage) = cc.storage {
            return Ok(Box::new(eframe::get_value::<LinkBudgetApp>(storage, eframe::APP_KEY).unwrap_or_default()));
        }
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
        let positive = self.tx_power.val_dbm + gains;

        return positive - negative;
    }

    fn ui_parameters(&mut self, ui: &mut Ui) {
        frame_styled(ui).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Parameters");
                egui::Grid::new("base_data").num_columns(3).show(ui, |ui| {
                    ui.label("Temperature");
                    ui.add(DragValue::new(&mut self.temperature));
                    ui.label("K");
                    ui.end_row();

                    ui.label("Bandwidth");
                    ui.add(
                        prefix_drag_value(&mut self.bandwidth)
                            .range(0.0..=f64::MAX)
                            .speed(1e6),
                    );
                    ui.label("Hz");
                    ui.end_row();

                    let thermal_noise_floor = calc::watt_to_dbm(calc::thermal_noise_power(self.temperature, self.bandwidth));
                    ui.label("Thermal noise floor");
                    ui.label(format!("{thermal_noise_floor:.1}"));
                    ui.label("dBm");
                    ui.end_row();

                    ui.label("Frequency");
                    ui.add(
                        prefix_drag_value(&mut self.frequency)
                            .range(0.0..=f64::MAX)
                            .speed(1e6),
                    );
                    ui.label("Hz");
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::Snr,
                        "SNR",
                    );
                    ui.add(DragValue::new(&mut self.snr));
                    ui.label("dB");
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::TxPower,
                        "Tx Power",
                    );
                    self.tx_power.value_selector_ui( ui);
                    self.tx_power.unit_selector_ui("tx_power", ui);
                    ui.end_row();

                    ui.label("Rx Power");
                    self.rx_power.val_dbm = self.snr + thermal_noise_floor;
                    ui.label(format!("{:.2}", self.rx_power.get_in_unit()));
                    self.rx_power.unit_selector_ui("rx_power", ui);
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
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let total_db = self.total_sum();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Reset").clicked() {
                    *self = Self::default();
                }
                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.collapsing("How to use", |ui| {
                ui.set_max_width(400.0);
                ui.label("This tool calculates the link budget for a noise limited wireless transmission in free space. It can calculate the SNR, the required TX Power, or the achievable transmission distance. You can add Gains like TX or RX antenna gains, and losses like a fading margin or the noise figure.");
                ui.label("Frequencies can be entered in scientific notation (20e6) or with a suffix (20M)");
            });
            ui.horizontal(|ui| {
                self.ui_parameters(ui);
                self.ui_path_loss(ui);
            });
            frame_styled(ui)
                .show(ui, |ui| {
                    ui.heading("Gains");
                    ui.horizontal(|ui| {
                        let name_response =
                            ui.add(TextEdit::singleline(&mut self.gain_name).hint_text("Gain Name"));
                        if ui.button("Add").clicked()
                            || (name_response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            if !self.gain_name.trim().is_empty() {
                                self.gains.insert(self.gain_name.clone(), 10.0);
                                self.gain_name.clear();
                            }
                        }
                    });
                    TableBuilder::new(ui)
                        .id_salt("gain_table")
                        .striped(true)
                        .column(Column::exact(20.0))
                        .column(Column::exact(250.0))
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
                    if ui.button("Add").clicked()
                        || (name_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.loss_name.trim().is_empty() {
                            self.losses.insert(self.loss_name.clone(), 10.0);
                            self.loss_name.clear();
                        }
                    }
                });
                TableBuilder::new(ui)
                    .id_salt("loss_table")
                    .striped(true)
                    .column(Column::exact(20.0))
                    .column(Column::exact(250.0))
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
                self.tx_power.val_dbm -= total_db;
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
