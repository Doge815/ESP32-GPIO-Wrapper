#![allow(dead_code)]
use async_trait::async_trait;
use core::fmt;
use esp_idf_svc::{
    hal::{
        self,
        adc::{AdcChannelDriver, AdcDriver, ADC1, ADC2},
        gpio::*,
        peripheral::Peripheral,
    },
    sys::EspError,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum GpioWrapperError {
    EspError(EspError),
    PinDoesNotExist,
    AdcNotOwned,
    PinNotOwned,
    NotAnAdcPin,
}

impl fmt::Display for GpioWrapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpioWrapperError::EspError(e) => write!(f, "EspError: {}", e),
            GpioWrapperError::AdcNotOwned => {
                write!(f, "The wrapper does not own the required ADC.")
            }
            GpioWrapperError::PinNotOwned => write!(f, "The wrapper does not own the pin."),
            GpioWrapperError::PinDoesNotExist => write!(f, "The pin does not exist."),
            GpioWrapperError::NotAnAdcPin => {
                write!(f, "Cannot perform ADC mesurements on an non ADC pin")
            }
        }
    }
}

impl std::error::Error for GpioWrapperError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GpioWrapperError::EspError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EspError> for GpioWrapperError {
    fn from(e: EspError) -> Self {
        GpioWrapperError::EspError(e)
    }
}

#[async_trait]
pub trait GpioPin: Send + Sync {
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError>;
}

include!(concat!(env!("OUT_DIR"), "/adc.rs"));

/*struct GpioPin32<'a> {
    pin: Option<Gpio32>,
    adc1_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC1>>>>,
    adc2_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC2>>>>,
}

impl<'a> GpioPin32<'a> {
    fn new(
        pin: Gpio32,
        adc1_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC1>>>>,
        adc2_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC2>>>>,
    ) -> Self {
        GpioPin32 {
            pin: Some(pin),
            adc1_driver,
            adc2_driver,
        }
    }
}

#[async_trait]
impl GpioPin for GpioPin32<'_> {
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError> {
        if self.pin.is_some() {
            let pin = self.pin.as_mut().unwrap();
            let mut adc_driver = self.adc1_driver.lock().await;
            if adc_driver.is_some() {
                let mut adc_channel_driver: AdcChannelDriver<
                    { hal::adc::attenuation::DB_11 },
                    Gpio32,
                > = AdcChannelDriver::new((pin).into_ref()).unwrap();
                let readout = adc_driver
                    .as_mut()
                    .unwrap()
                    .read(&mut adc_channel_driver)
                    .map_err(GpioWrapperError::from);
                return readout;
            }
            return Err(GpioWrapperError::AdcNotOwned);
        }
        Err(GpioWrapperError::PinNotOwned)
    }
}

unsafe impl Send for GpioPin32<'_> {}
unsafe impl Sync for GpioPin32<'_> {}*/

pub struct PinWrapper {
    pin: Arc<Mutex<Option<Box<dyn GpioPin>>>>,
}

impl PinWrapper {
    pub async fn get_adc(&self) -> Result<u16, GpioWrapperError> {
        let mut pin = self.pin.lock().await;
        return if pin.is_some() {
            pin.as_mut().unwrap().get_adc().await
        } else {
            Err(GpioWrapperError::PinNotOwned)
        };
    }
}

#[derive(Clone)]
pub struct GpioWrapper {
    pins: Vec<Arc<Mutex<Option<Box<dyn GpioPin>>>>>,
}

impl GpioWrapper {
    pub fn new(adc1: Option<ADC1>, adc2: Option<ADC2>, pins: Pins) -> Self {
        let adc1_driver = adc1.map(|adc1| {
            AdcDriver::new(adc1, &hal::adc::config::Config::new().calibration(true)).unwrap()
        });
        let adc2_driver = adc2.map(|adc2| {
            AdcDriver::new(adc2, &hal::adc::config::Config::new().calibration(true)).unwrap()
        });
        let adc1_ref = Arc::new(Mutex::new(adc1_driver));
        let adc2_ref = Arc::new(Mutex::new(adc2_driver));

        let pins: Vec<Arc<Mutex<Option<Box<dyn GpioPin>>>>> = vec![
            Arc::new(Mutex::new(Some(Box::new(GpioPin0::new( pins.gpio0, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin1::new( pins.gpio1, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin2::new( pins.gpio2, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin3::new( pins.gpio3, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin4::new( pins.gpio4, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin5::new( pins.gpio5, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin6::new( pins.gpio6, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin7::new( pins.gpio7, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin8::new( pins.gpio8, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin9::new( pins.gpio9, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin10::new(pins.gpio10, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin11::new(pins.gpio11, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin12::new(pins.gpio12, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin13::new(pins.gpio13, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin14::new(pins.gpio14, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin15::new(pins.gpio15, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin16::new(pins.gpio16, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin17::new(pins.gpio17, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin18::new(pins.gpio18, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin19::new(pins.gpio19, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin20::new(pins.gpio20, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin21::new(pins.gpio21, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin22::new(pins.gpio22, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin23::new(pins.gpio23, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin24::new(pins.gpio24, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin25::new(pins.gpio25, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin26::new(pins.gpio26, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin27::new(pins.gpio27, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin28::new(pins.gpio28, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin29::new(pins.gpio29, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin30::new(pins.gpio30, adc1_ref.clone(), adc2_ref.clone()))))),
            //Arc::new(Mutex::new(Some(Box::new(GpioPin31::new(pins.gpio31, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin32::new(pins.gpio32, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin33::new(pins.gpio33, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin34::new(pins.gpio34, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin35::new(pins.gpio35, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin36::new(pins.gpio36, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin37::new(pins.gpio37, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin38::new(pins.gpio38, adc1_ref.clone(), adc2_ref.clone()))))),
            Arc::new(Mutex::new(Some(Box::new(GpioPin39::new(pins.gpio39, adc1_ref.clone(), adc2_ref.clone()))))),
        ];
        GpioWrapper { pins }
    }

    pub fn get_pin(&self, pin: usize) -> Result<PinWrapper, GpioWrapperError> {
        const NON_EXISTENT_PINS: [usize; 6] = [20, 24, 28, 29, 30, 31];
        let mut index = pin as usize;

        if NON_EXISTENT_PINS.contains(&index) || index >= 40 {
            return Err(GpioWrapperError::PinDoesNotExist);
        }

        for i in NON_EXISTENT_PINS.iter() {
            if pin >= *i {
                index -= 1;
            } else {
                break;
            }
        }

        Ok(PinWrapper { pin: self.pins.get(index).unwrap().clone()})
    }
}
