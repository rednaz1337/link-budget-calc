use eframe::{App, CreationContext, Frame};
use egui::CentralPanel;
use egui::{Context, DragValue, Slider, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};
use number_prefix::{NumberPrefix, Prefix};
use std::error::Error;

mod asynch;
mod calc;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Link Budget Calculator",
        native_options,
        Box::new(|cc| LinkBudgetApp::new(cc)),
    )
    .unwrap();
}

#[derive(Default, PartialEq, Eq)]
enum CalculationTarget {
    Temperature,
    Bandwidth,
    #[default]
    Snr,
    NoiseFigure,
    TxGain,
    RxGain,
    Distance,
    AdditionalLosses,
    TxPower,
    RxPower,
}

struct LinkBudgetApp {
    temperature: f64,  // Kelvin
    frequency: f64,    // Hertz
    bandwidth: f64,    // Hertz
    snr_min: f64,      // dB
    noise_figure: f64, // dB

    tx_power: f64, // dBm
    rx_power: f64, // dBm
    tx_gain: f64,  // dB
    rx_gain: f64,  // dB

    distance: f64, // meter
    d_break: f64,  // meter
    break_exponent: f64,

    additional_losses: Vec<AdditionalLoss>,
    loss_name: String,
    loss_db: f64,

    calculation_target: CalculationTarget,
}

impl Default for LinkBudgetApp {
    fn default() -> Self {
        Self {
            temperature: 290.0,
            bandwidth: 20e6,
            snr_min: 10.0,
            frequency: 2.4e9,
            noise_figure: 10.0,
            tx_gain: 5.0,
            rx_gain: 5.0,
            tx_power: 30.0,
            rx_power: 1.0,
            distance: 2000.0,
            d_break: 500.0,
            break_exponent: 4.3,
            additional_losses: Vec::default(),
            loss_name: String::default(),
            loss_db: 10.0,
            calculation_target: CalculationTarget::default(),
        }
    }
}

struct AdditionalLoss {
    name: String,
    loss: f64,
}

impl LinkBudgetApp {
    pub fn new(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
        Ok(Box::new(Self::default()))
    }

    pub fn additional_losses(&self) -> f64 {
        self.additional_losses.iter().map(|l| l.loss).sum()
    }

    pub fn total_sum(&self) -> f64 {
        let thermal =
            calc::watt_to_dbm(calc::thermal_noise_power(self.temperature, self.bandwidth));
        let additional = self.additional_losses();
        let path = calc::friis::path_loss(self.distance, self.d_break, self.frequency, self.break_exponent);

        let negative =
             thermal
            + additional
            + path
            + self.snr_min
            + self.noise_figure
            + self.rx_power;
        let positive = self.tx_gain + self.tx_power + self.rx_gain;

        return positive - negative;
    }
}

impl eframe::App for LinkBudgetApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let total_db = self.total_sum();
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Link Budget Calculator");
            frame_styled(ui).show(ui, |ui| {
                ui.heading("Base Info");
                egui::Grid::new("base_data").num_columns(2).show(ui, |ui| {
                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::Temperature,
                        "Temperature",
                    );
                    ui.add(DragValue::new(&mut self.temperature).suffix(" K"));
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
                        CalculationTarget::Bandwidth,
                        "Bandwidth",
                    );
                    ui.add(
                        prefix_drag_value(&mut self.bandwidth)
                            .suffix("Hz")
                            .range(0.0..=f64::MAX)
                            .speed(1e6),
                    );
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::Snr,
                        "Min. SNR",
                    );
                    ui.add(DragValue::new(&mut self.snr_min).suffix(" dB"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::TxPower,
                        "Tx Power",
                    );
                    ui.add(DragValue::new(&mut self.tx_power).suffix(" dBm"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::RxPower,
                        "Rx Power (sensitivity)",
                    );
                    ui.add(DragValue::new(&mut self.rx_power).suffix(" dBm"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::NoiseFigure,
                        "Noise Figure",
                    );
                    ui.add(DragValue::new(&mut self.noise_figure).suffix(" dB"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::TxGain,
                        "Tx Gain",
                    );
                    ui.add(DragValue::new(&mut self.tx_gain).suffix(" dB"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::RxGain,
                        "Rx Gain",
                    );
                    ui.add(DragValue::new(&mut self.rx_gain).suffix(" dB"));
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.calculation_target,
                        CalculationTarget::AdditionalLosses,
                        "AdditionalLosses",
                    );
                    let loss_sum = self.additional_losses.iter().map(|l| l.loss).sum::<f64>();
                    ui.label(format!("{:.2} dB", loss_sum));
                    ui.end_row();
                })
            });
            frame_styled(ui).show(ui, |ui| {
                if matches!(self.calculation_target, CalculationTarget::AdditionalLosses) {
                    ui.disable();
                }
                ui.heading("Additional Losses");
                ui.horizontal(|ui| {
                    ui.label("Add loss: ");
                    let name_response =
                        ui.add(TextEdit::singleline(&mut self.loss_name).hint_text("Loss Name"));
                    ui.add(DragValue::new(&mut self.loss_db).suffix(" dB"));
                    if ui.button("Add").clicked()
                        || (name_response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.loss_name.trim().is_empty() {
                            self.additional_losses.push(AdditionalLoss {
                                name: self.loss_name.clone(),
                                loss: self.loss_db,
                            });
                            self.loss_name.clear();
                            self.loss_db = 0.0;
                        }
                    }
                });
                ui.separator();
                let mut delete_loss: Option<usize> = None;
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(20.0))
                    .column(Column::remainder())
                    .column(Column::exact(100.0))
                    .header(20., |mut header| {
                        header.col(|ui| {
                            ui.label(" ");
                        });
                        header.col(|ui| {
                            ui.heading("Loss Name");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        for (index, loss) in self.additional_losses.iter_mut().enumerate() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("X").clicked() {
                                        delete_loss = Some(index);
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(loss.name.as_str());
                                });
                                row.col(|ui| {
                                    ui.add(DragValue::new(&mut loss.loss).suffix(" dB"));
                                });
                            })
                        }
                    });
                if let Some(index) = delete_loss {
                    self.additional_losses.remove(index);
                }
            });
            frame_styled(&ui).show(ui, |ui| {
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
                });
            });
        });

        match self.calculation_target {
            CalculationTarget::Temperature => {}
            CalculationTarget::Bandwidth => {}
            CalculationTarget::Snr => {
                self.snr_min += total_db;
            }
            CalculationTarget::NoiseFigure => {}
            CalculationTarget::TxGain => {}
            CalculationTarget::RxGain => {}
            CalculationTarget::Distance => {}
            CalculationTarget::AdditionalLosses => {}
            CalculationTarget::TxPower => {}
            CalculationTarget::RxPower => {}
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
