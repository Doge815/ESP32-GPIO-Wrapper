#![allow(dead_code, unused_variables)]
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};
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
use std::{fmt::Debug, sync::Arc};
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
trait GpioPin: Send + Sync + Downcast + Debug {
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError>;
    async fn get_adc_averaged(
        &mut self,
        measurement: MeasurementConfig,
    ) -> Result<f32, GpioWrapperError>;
}
impl_downcast!(GpioPin);

#[derive(Clone, Debug)]
pub enum Attenuation {
    DB0,
    DB2_5,
    DB6,
    DB11,
}
#[derive(Clone, Debug)]
pub struct MeasurementConfig {
    pub to_measure: u32,
    pub attenuation: Attenuation,
}

include!(concat!(env!("OUT_DIR"), "/adc.rs"));

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
    pub async fn get_adc_averaged(
        &self,
        measurement: MeasurementConfig,
    ) -> Result<f32, GpioWrapperError> {
        let mut pin = self.pin.lock().await;
        return if pin.is_some() {
            pin.as_mut().unwrap().get_adc_averaged(measurement).await
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
        GpioWrapper {
            pins: pins_vec(adc1, adc2, pins),
        }
    }

    pub fn get_pin(&self, pin: usize) -> Result<PinWrapper, GpioWrapperError> {
        if pin >= self.pins.len() {
            return Err(GpioWrapperError::PinDoesNotExist);
        }

        Ok(PinWrapper {
            pin: self.pins.get(pin).unwrap().clone(),
        })
    }
}

/*impl GpioWrapper {
    pub async fn release_pin0(&mut self) -> Result<Gpio0, GpioWrapperError> {
        let wrapper = self.get_pin(0).unwrap();
        let mut pin = wrapper.pin.lock().await;
        return if let Some(boxed) = pin.take() {
            let x: Box<GpioPin0> = boxed.downcast().unwrap();
            Ok(x.pin)
        } else {
            Err(GpioWrapperError::PinNotOwned)
        }
    }
}*/
