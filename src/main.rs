use std::error::Error;
use eframe::{App, CreationContext, Frame};
use egui::{Context, DragValue, Slider, TextEdit, Ui};
use egui::CentralPanel;
use egui_extras::{Column, TableBuilder};
use number_prefix::{NumberPrefix, Prefix};
use uom::ConstantOp::Add;
use uom::num::Num;

mod calc;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Link Budget Calculator", native_options, Box::new(|cc| LinkBudgetApp::new(cc))).unwrap();
}

#[derive(Default)]
struct LinkBudgetApp {
    temperature: f64, // Kelvin
    bandwidth: f64, // Hertz
    snr_min: f64, // dB
    noise_figure: f64, // dB

    tx_gain: f64, // dB
    rx_gain: f64, // dB

    distance: f64, // meter
    d_break: f64, // meter
    break_exponent: f64,

    additional_losses: Vec<AdditionalLoss>,
    loss_name: String,
    loss_db: f64,
}

struct AdditionalLoss {
    name: String,
    loss: f64,
}

impl LinkBudgetApp {
    pub fn new(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
        Ok(Box::new(Self::default()))
    }
}

impl eframe::App for LinkBudgetApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Minimal transmission power");
            frame_styled(ui).show(ui, |ui| {
                ui.heading("Base Info");
                egui::Grid::new("base_data").num_columns(2).show(ui, |ui| {
                    ui.label("Temperature");
                    ui.add(DragValue::new(&mut self.temperature).suffix(" K"));
                    ui.end_row();

                    ui.label("Bandwidth");
                    ui.add(prefix_drag_value(&mut self.bandwidth).suffix("Hz").range(0.0..=f64::MAX).speed(1e6));
                    ui.end_row();

                    ui.label("Min. SNR");
                    ui.add(DragValue::new(&mut self.snr_min).suffix(" dB"));
                    ui.end_row();

                    ui.label("Noise figure");
                    ui.add(DragValue::new(&mut self.noise_figure).suffix(" dB"));
                    ui.end_row();

                    ui.label("TX Gain");
                    ui.add(DragValue::new(&mut self.tx_gain).suffix(" dBm"));
                    ui.end_row();

                    ui.label("RX Gain");
                    ui.add(DragValue::new(&mut self.rx_gain).suffix(" dBm"));
                    ui.end_row();
                })
            });
            frame_styled(ui).show(ui, |ui| {
                ui.heading("Additional Losses");
                ui.horizontal(|ui| {
                    ui.label("Add loss: ");
                    let name_response = ui.add(TextEdit::singleline(&mut self.loss_name).hint_text("Loss Name"));
                    ui.add(DragValue::new(&mut self.loss_db).suffix(" dB"));
                    if ui.button("Add").clicked() || (name_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
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
        });
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
        .custom_formatter(|value, range| {
            match number_prefix::NumberPrefix::decimal(value) {
                NumberPrefix::Standalone(num) => {
                    format!("{num} ")
                }
                NumberPrefix::Prefixed(prefix, num) => {
                    format!("{:.1} {}", num, prefix)
                }
            }
        }).custom_parser(|value| {
        let Ok(number_prefix) = value.parse::<NumberPrefix<f64>>() else {
            return None;
        };

        return match number_prefix {
            NumberPrefix::Standalone(number) => { Some(number) }
            NumberPrefix::Prefixed(prefix, number) => {
                let factor = match prefix {
                    Prefix::Kilo => { 1e3 }
                    Prefix::Mega => { 1e6 }
                    Prefix::Giga => { 1e9 }
                    Prefix::Tera => { 1e12 }
                    Prefix::Peta => { 1e15 }
                    Prefix::Exa => { 1e18 }
                    Prefix::Zetta => { 1e21 }
                    Prefix::Yotta => { 1e24 }
                    _ => return None
                };

                Some(factor * number)
            }
        };
    })
}