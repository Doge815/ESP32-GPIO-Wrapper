use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("adc.rs");

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dest_path.clone())
        .unwrap();

    const ADC1_PINS: [i32; 8] = [32, 33, 34, 35, 36, 37, 38, 39];
    const ADC2_PINS: [i32; 9] = [2, 4, 12, 13, 14, 15, 25, 26, 27];
    const NON_EXISTENT_PINS: [i32; 6] = [20, 24, 28, 29, 30, 31];

    for pin in 0..40 {
        if NON_EXISTENT_PINS.contains(&pin) {
            continue;
        }

        let adc = if ADC1_PINS.contains(&pin) {
            1
        } else if ADC2_PINS.contains(&pin) {
            2
        } else {
            0
        };

        let used_adc = match adc {
            1 => "ADC1",
            2 => "ADC2",
            _ => "",
        };

        let used_adc_driver = match adc {
            1 => "adc1_driver",
            2 => "adc2_driver",
            _ => "",
        };

        file.write(
            format!(
                "
struct GpioPin{pin}<'a> {{
    pin: Option<Gpio{pin}>,
    adc1_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC1>>>>,
    adc2_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC2>>>>,
}}

impl<'a> GpioPin{pin}<'a> {{
    fn new(
        pin: Gpio{pin},
        adc1_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC1>>>>,
        adc2_driver: Arc<Mutex<Option<AdcDriver<'a, hal::adc::ADC2>>>>,
    ) -> Self {{
        GpioPin{pin} {{
            pin: Some(pin),
            adc1_driver,
            adc2_driver,
        }}
    }}
}}

unsafe impl Send for GpioPin{pin}<'_> {{}}
unsafe impl Sync for GpioPin{pin}<'_> {{}}

#[async_trait]
impl GpioPin for GpioPin{pin}<'_> {{
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError> {{
"
            )
            .as_bytes(),
        )
        .unwrap();

        if adc != 0 {
            file.write(
                format!(
                    "
        if self.pin.is_some() {{
            let pin = self.pin.as_mut().unwrap();
            let mut adc_driver = self.{used_adc_driver}.lock().await;
            if adc_driver.is_some() {{
                let mut adc_channel_driver: AdcChannelDriver<
                    {{ hal::adc::attenuation::DB_11 }},
                    Gpio{pin},
                > = AdcChannelDriver::new((pin).into_ref()).unwrap();
                let readout = adc_driver
                    .as_mut()
                    .unwrap()
                    .read(&mut adc_channel_driver)
                    .map_err(GpioWrapperError::from);
                return readout;
            }}
            return Err(GpioWrapperError::AdcNotOwned);
        }}
        Err(GpioWrapperError::PinNotOwned)
",
                )
                .as_bytes(),
            )
            .unwrap();
        } else {
            file.write(
                format!(
                    "
        Err(GpioWrapperError::NotAnAdcPin)
",
                )
                .as_bytes(),
            )
            .unwrap();
        }
        file.write(
            format!(
                "
    }}
    async fn get_adc_averaged( &mut self, measurement: MeasurementConfig) -> Result<f32, GpioWrapperError> {{
",
            )
            .as_bytes(),
        )
        .unwrap();
        if adc != 0 {
            file.write(
                format!(
                    "
        if measurement.to_measure == 0 {{
            return Ok(f32::NAN);
        }}
        if self.pin.is_some() {{
            let pin = self.pin.as_mut().unwrap();
            let mut adc_driver = self.{used_adc_driver}.lock().await;
            if adc_driver.is_some() {{
                fn helper<const N: u32>(
                    pin: &mut Gpio{pin},
                    adc_driver: &mut tokio::sync::MutexGuard<Option<AdcDriver<{used_adc}>>>,
                    measurement: MeasurementConfig,
                ) -> Result<f32, GpioWrapperError> {{
                    let mut adc_channel_driver: AdcChannelDriver<{{ N }}, Gpio{pin}> =
                        AdcChannelDriver::new((pin).into_ref()).unwrap();
                    let reader = adc_driver.as_mut().unwrap();
                    let mut sum = 0.0;
                    for _ in 0..measurement.to_measure {{
                        sum += reader
                            .read(&mut adc_channel_driver)
                            .map_err(GpioWrapperError::from)? as f32;
                    }}
                    return Ok(sum as f32 / measurement.to_measure as f32);
                }}
                const DB_0: u32 = hal::sys::adc_atten_t_ADC_ATTEN_DB_0;
                const DB_2_5: u32 = hal::sys::adc_atten_t_ADC_ATTEN_DB_2_5;
                const DB_6: u32 = hal::sys::adc_atten_t_ADC_ATTEN_DB_6;
                const DB_11: u32 = hal::sys::adc_atten_t_ADC_ATTEN_DB_11;
                const DB_12: u32 = hal::sys::adc_atten_t_ADC_ATTEN_DB_12; //not used

                match measurement.attenuation {{
                    Attenuation::DB0 => {{
                        return helper::<DB_0>(pin, &mut adc_driver, measurement);
                        //cannot use the constant directly, wtf rust???
                    }}
                    Attenuation::DB2_5 => {{
                        return helper::<DB_2_5>(pin, &mut adc_driver, measurement);
                    }}
                    Attenuation::DB6 => {{
                        return helper::<DB_6>(pin, &mut adc_driver, measurement);
                    }}
                    Attenuation::DB11 => {{
                        return helper::<DB_11>(pin, &mut adc_driver, measurement);
                    }}
                }}
            }}
            return Err(GpioWrapperError::AdcNotOwned);
        }}
        Err(GpioWrapperError::PinNotOwned)
",
                )
                .as_bytes(),
            )
            .unwrap();
        } else {
            file.write(
                format!(
                    "
        Err(GpioWrapperError::NotAnAdcPin)
",
                )
                .as_bytes(),
            )
            .unwrap();
        }
        file.write(
            format!(
                "
    }}
}}
",
            )
            .as_bytes(),
        )
        .unwrap();
    }
    println!("cargo::rerun-if-changed=build.rs");
}
