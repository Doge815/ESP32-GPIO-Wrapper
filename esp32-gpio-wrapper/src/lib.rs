use esp_idf_svc::hal::{
    adc::{AdcChannelDriver, AdcDriver, ADC1, ADC2},
    gpio::*,
};
use std::sync::Arc;
use tokio::sync::Mutex;

trait Adc1PinWrapper: Send + Sync {
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC1>) -> u16;
}

trait Adc2PinWrapper: Send + Sync {
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC2>) -> u16;
}

include!(concat!(env!("OUT_DIR"), "/adc.rs"));

/*struct Adc1Gpio32<'a> {
    adc_pin: AdcChannelDriver<'a, { esp_idf_svc::hal::adc::attenuation::DB_11 }, Gpio32>,
}

impl Adc1Gpio32<'_> {
    pub fn new(pin: Gpio32) -> Self {
        let adc_pin: AdcChannelDriver<{ esp_idf_svc::hal::adc::attenuation::DB_11 }, Gpio32> =
            esp_idf_svc::hal::adc::AdcChannelDriver::new(pin).unwrap();
        Adc1Gpio32 { adc_pin }
    }
}

unsafe impl Send for Adc1Gpio32<'_> {}
unsafe impl Sync for Adc1Gpio32<'_> {}

impl<'a> Adc1PinWrapper for Adc1Gpio32<'a> {
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC1>) -> u16 {
        driver.read(&mut self.adc_pin).unwrap()
    }
}

struct Adc1Gpio33<'a> {
    adc_pin: AdcChannelDriver<'a, { esp_idf_svc::hal::adc::attenuation::DB_11 }, Gpio33>,
}

impl Adc1Gpio33<'_> {
    pub fn new(pin: Gpio33) -> Self {
        let adc_pin: AdcChannelDriver<{ esp_idf_svc::hal::adc::attenuation::DB_11 }, Gpio33> =
            esp_idf_svc::hal::adc::AdcChannelDriver::new(pin).unwrap();
        Adc1Gpio33 { adc_pin }
    }
}

unsafe impl Send for Adc1Gpio33<'_> {}
unsafe impl Sync for Adc1Gpio33<'_> {}

impl<'a> Adc1PinWrapper for Adc1Gpio33<'a> {
    fn get_adc(&mut self, driver: &mut AdcDriver<'_, esp_idf_svc::hal::adc::ADC1>) -> u16 {
        driver.read(&mut self.adc_pin).unwrap()
    }
}*/

struct GpioPins<'a> {
    adc1_driver: AdcDriver<'a, esp_idf_svc::hal::adc::ADC1>,
    adc2_driver: Option<AdcDriver<'a, esp_idf_svc::hal::adc::ADC2>>,

    adc1_ports: Vec<Box<dyn Adc1PinWrapper>>,
    adc2_ports: Vec<Box<dyn Adc2PinWrapper>>,
}

impl GpioPins<'_> {
    fn get_adc(&mut self, index: usize) -> Option<u16> {
        if index >= 40 {
            return None;
        }
        if index >= 32 && index <= 39 {
            return Some(self.adc1_ports[index - 32].get_adc(&mut self.adc1_driver));
        }
        if self.adc2_driver.is_some() {
            let mut adc2_driver = self.adc2_driver.as_mut().unwrap();
            match index {
                2 => Some(self.adc2_ports[0].get_adc(&mut adc2_driver)),
                4 => Some(self.adc2_ports[1].get_adc(&mut adc2_driver)),
                12 => Some(self.adc2_ports[2].get_adc(&mut adc2_driver)),
                13 => Some(self.adc2_ports[3].get_adc(&mut adc2_driver)),
                14 => Some(self.adc2_ports[4].get_adc(&mut adc2_driver)),
                15 => Some(self.adc2_ports[5].get_adc(&mut adc2_driver)),
                25 => Some(self.adc2_ports[6].get_adc(&mut adc2_driver)),
                26 => Some(self.adc2_ports[7].get_adc(&mut adc2_driver)),
                27 => Some(self.adc2_ports[8].get_adc(&mut adc2_driver)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn new(adc1: ADC1, adc2: Option<ADC2>, pins: Pins) -> Self {
        let adc1_ports: Vec<Box<dyn Adc1PinWrapper>> = vec![
            Box::new(Adc1Gpio32::new(pins.gpio32)),
            Box::new(Adc1Gpio33::new(pins.gpio33)),
            Box::new(Adc1Gpio34::new(pins.gpio34)),
            Box::new(Adc1Gpio35::new(pins.gpio35)),
            Box::new(Adc1Gpio36::new(pins.gpio36)),
            Box::new(Adc1Gpio37::new(pins.gpio37)),
            Box::new(Adc1Gpio38::new(pins.gpio38)),
            Box::new(Adc1Gpio39::new(pins.gpio39)),
        ];

        let adc2_ports: Vec<Box<dyn Adc2PinWrapper>> = if adc2.is_some() {
            vec![
                Box::new(Adc2Gpio2::new(pins.gpio2)),
                Box::new(Adc2Gpio4::new(pins.gpio4)),
                Box::new(Adc2Gpio12::new(pins.gpio12)),
                Box::new(Adc2Gpio13::new(pins.gpio13)),
                Box::new(Adc2Gpio14::new(pins.gpio14)),
                Box::new(Adc2Gpio15::new(pins.gpio15)),
                Box::new(Adc2Gpio25::new(pins.gpio25)),
                Box::new(Adc2Gpio26::new(pins.gpio26)),
                Box::new(Adc2Gpio27::new(pins.gpio27)),
            ]
        } else {
            vec![]
        };

        let adc1_driver: AdcDriver<esp_idf_svc::hal::adc::ADC1> = AdcDriver::new(
            adc1,
            &esp_idf_svc::hal::adc::config::Config::new().calibration(true),
        )
        .unwrap();

        let adc2_driver: Option<AdcDriver<esp_idf_svc::hal::adc::ADC2>> = if let Some(adc2) = adc2 {
            Some(
                AdcDriver::new(
                    adc2,
                    &esp_idf_svc::hal::adc::config::Config::new().calibration(true),
                )
                .unwrap(),
            )
        } else {
            None
        };

        GpioPins {
            adc1_driver,
            adc2_driver,
            adc1_ports,
            adc2_ports,
        }
    }
}

#[derive(Clone)]
pub struct GpioWrapper<'a> {
    pins: Arc<Mutex<GpioPins<'a>>>,
}

unsafe impl Send for GpioWrapper<'_> {}
unsafe impl Sync for GpioWrapper<'_> {}

impl GpioWrapper<'_> {
    pub fn new(adc1: ADC1, adc2: Option<ADC2>, pins: Pins) -> Self {
        GpioWrapper {
            pins: Arc::new(Mutex::new(GpioPins::new(adc1, adc2, pins))),
        }
    }

    pub async fn get_adc(&self, index: usize) -> Option<u16> {
        let mut pins = self.pins.lock().await;
        pins.get_adc(index)
    }
}
