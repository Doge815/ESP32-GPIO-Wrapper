/*fn main() {
    embuild::espidf::sysenv::output();
}*/
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("adc.rs");

let mut file = OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(dest_path.clone())
        .unwrap();

    for adc1_pin in 32..40 { //pins 37 and 38 don't exist
        file.write(
            format!("
struct Adc1Gpio{pin}<'a> {{
    adc_pin: AdcChannelDriver<'a, {{ esp_idf_svc::hal::adc::attenuation::DB_11 }}, Gpio{pin}>,
}}

impl Adc1Gpio{pin}<'_> {{
    pub fn new(pin: Gpio{pin}) -> Self {{
        let adc_pin: AdcChannelDriver<{{ esp_idf_svc::hal::adc::attenuation::DB_11 }}, Gpio{pin}> =
            esp_idf_svc::hal::adc::AdcChannelDriver::new(pin).unwrap();
        Adc1Gpio{pin} {{ adc_pin }}
    }}
}}

unsafe impl Send for Adc1Gpio{pin}<'_> {{}}
unsafe impl Sync for Adc1Gpio{pin}<'_> {{}}

impl<'a> Adc1PinWrapper for Adc1Gpio{pin}<'a> {{
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC1>) -> u16 {{
        driver.read(&mut self.adc_pin).unwrap()
    }}
}}

", pin = adc1_pin).as_bytes(),
        )
        .unwrap();
    }

    for adc2_pin in [2, 4, 12, 13, 14, 15, 25, 26, 27] {
        file.write(
            format!("
struct Adc2Gpio{pin}<'a> {{
    adc_pin: AdcChannelDriver<'a, {{ esp_idf_svc::hal::adc::attenuation::DB_11 }}, Gpio{pin}>,
}}

impl Adc2Gpio{pin}<'_> {{
    pub fn new(pin: Gpio{pin}) -> Self {{
        let adc_pin: AdcChannelDriver<{{ esp_idf_svc::hal::adc::attenuation::DB_11 }}, Gpio{pin}> =
            esp_idf_svc::hal::adc::AdcChannelDriver::new(pin).unwrap();
        Adc2Gpio{pin} {{ adc_pin }}
    }}
}}

unsafe impl Send for Adc2Gpio{pin}<'_> {{}}
unsafe impl Sync for Adc2Gpio{pin}<'_> {{}}

impl<'a> Adc2PinWrapper for Adc2Gpio{pin}<'a> {{
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC2>) -> u16 {{
        driver.read(&mut self.adc_pin).unwrap()
    }}
}}

", pin = adc2_pin).as_bytes(),
        )
        .unwrap();
    }
    println!("cargo::rerun-if-changed=build.rs");
}
