const KB: f64 = 1.380649e-23; // Boltzmann constant (joule per kelvin)
const C: f64 = 299792458.0;
pub fn lambda(frequency: f64) -> f64 {
    C / frequency
}

pub fn thermal_noise_power(temperature: f64, bandwidth: f64) -> f64 {
    KB * temperature * bandwidth
}

pub fn thermal_noise_temperature(power: f64, bandwidth: f64) -> f64 {
    power / bandwidth / KB
}

pub fn milliwatt_to_dbm(power: f64) -> f64 {
    10.0 * f64::log10(power)
}

pub fn watt_to_dbm(power: f64) -> f64 {
    10.0 * f64::log10(power * 1000.0)
}

pub fn dbm_to_milliwat(dbm: f64) -> f64 {
    f64::powf(10.0, dbm / 10.)
}

pub fn dbm_to_watt(dbm: f64) -> f64 {
    f64::powf(10.0, dbm / 10.) / 1000.0
}
pub mod friis {
    
    

    pub fn path_loss(distance: f64, d_break: f64, frequency: f64, break_exponent: f64) -> f64 {
        let one_meter_one_ghz = 32.0; // dB
        let freq_loss = 20.0 * f64::log10(frequency / 1e9);
        let path_loss = one_meter_one_ghz + freq_loss + if distance < d_break {
            20.0 * f64::log10(distance / 1.0)
        } else {
            20.0 * f64::log10(d_break / 1.0) + break_exponent * 10.0 * f64::log10(distance / d_break)
        };

        return path_loss;
    }

    pub fn distance(path_loss: f64, d_break: f64, frequency: f64, break_exponent: f64) -> f64 {
        let one_meter_one_ghz = 32.0; // dB
        let freq_loss = 20.0 * f64::log10(frequency / 1e9);
        let path_loss = path_loss - one_meter_one_ghz - freq_loss;
        let loss_at_break = 20.0 * f64::log10(d_break / 1.0);

        let distance = if path_loss <= loss_at_break {
            10f64.powf(path_loss / 20.0)
        } else {
            10f64.powf((path_loss - loss_at_break) / break_exponent / 10.0) * d_break
        };
        return distance;
    }
}
