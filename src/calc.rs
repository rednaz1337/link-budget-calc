use uom::si::f64::*;
use uom::si::velocity::meter_per_second;


pub fn c() -> Velocity {
    Velocity::new::<meter_per_second>(299792458.0)
}
pub fn lambda(frequency: Frequency) -> Length {
    c() / frequency
}
mod friis {
    use std::f64::consts::PI;
    use uom::si::DimensionOne;
    use uom::si::f64::*;
    use crate::calc::lambda;
    use uom::typenum::P2;

    pub fn path_loss(distance: Length, frequency: Frequency) -> Ratio {
        ((lambda(frequency) /(4.0 * PI * distance)).powi(P2::new()))
    }
    pub fn rx_power(tx_power: Power, tx_gain: f64, rx_gain: f64, distance: Length, frequency: Frequency) -> Power {
        tx_power * tx_gain * rx_gain * path_loss(distance, frequency)
    }
}