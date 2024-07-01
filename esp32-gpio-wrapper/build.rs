use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ADC {
    ADC1(i32),
    ADC2(i32),
    NOADC,
}

macro_rules! print {
    ($file:ident, $to_write:expr) => {
        $file.write(format!($to_write).as_bytes()).unwrap()
    };
}

fn main() {
    embuild::espidf::sysenv::output();

    let esp32_pins: Vec<(i32, ADC)> = vec![
        (0, ADC::ADC2(1)),
        (1, ADC::NOADC),
        (2, ADC::ADC2(2)),
        (3, ADC::NOADC),
        (4, ADC::ADC2(0)),
        (5, ADC::NOADC),
        (6, ADC::NOADC),
        (7, ADC::NOADC),
        (8, ADC::NOADC),
        (9, ADC::NOADC),
        (10, ADC::NOADC),
        (11, ADC::NOADC),
        (12, ADC::ADC2(5)),
        (13, ADC::ADC2(4)),
        (14, ADC::ADC2(6)),
        (15, ADC::ADC2(3)),
        (16, ADC::NOADC),
        (17, ADC::NOADC),
        (18, ADC::NOADC),
        (19, ADC::NOADC),
        (21, ADC::NOADC),
        (22, ADC::NOADC),
        (23, ADC::NOADC),
        (25, ADC::ADC2(8)),
        (26, ADC::ADC2(9)),
        (27, ADC::ADC2(7)),
        (32, ADC::ADC1(4)),
        (33, ADC::ADC1(5)),
        (34, ADC::ADC1(6)),
        (35, ADC::ADC1(7)),
        (36, ADC::ADC1(0)),
        (37, ADC::ADC1(1)),
        (38, ADC::ADC1(2)),
        (39, ADC::ADC1(3)),
    ];

    let esp32s2_pins: Vec<(i32, ADC)> = vec![
        (0, ADC::NOADC),
        (1, ADC::ADC1(0)),
        (2, ADC::ADC1(1)),
        (3, ADC::ADC1(2)),
        (4, ADC::ADC1(3)),
        (5, ADC::ADC1(4)),
        (6, ADC::ADC1(5)),
        (7, ADC::ADC1(6)),
        (8, ADC::ADC1(7)),
        (9, ADC::ADC1(8)),
        (10, ADC::ADC1(9)),
        (11, ADC::ADC2(0)),
        (12, ADC::ADC2(1)),
        (13, ADC::ADC2(2)),
        (14, ADC::ADC2(3)),
        (15, ADC::ADC2(4)),
        (16, ADC::ADC2(5)),
        (17, ADC::ADC2(6)),
        (18, ADC::ADC2(7)),
        (19, ADC::ADC2(8)),
        (20, ADC::ADC2(9)),
        (21, ADC::NOADC),
        (26, ADC::NOADC),
        (27, ADC::NOADC),
        (28, ADC::NOADC),
        (29, ADC::NOADC),
        (30, ADC::NOADC),
        (31, ADC::NOADC),
        (32, ADC::NOADC),
        (33, ADC::NOADC),
        (34, ADC::NOADC),
        (35, ADC::NOADC),
        (36, ADC::NOADC),
        (37, ADC::NOADC),
        (38, ADC::NOADC),
        (39, ADC::NOADC),
        (40, ADC::NOADC),
        (41, ADC::NOADC),
        (42, ADC::NOADC),
        (43, ADC::NOADC),
        (44, ADC::NOADC),
        (45, ADC::NOADC),
        (46, ADC::NOADC),
        (47, ADC::NOADC),
        (48, ADC::NOADC),
    ];

    let mut pins_opt = None;

    let args = embuild::espidf::sysenv::cfg_args().unwrap().args;

    if args.contains(&"esp32".to_string()) {
        pins_opt = Some(esp32_pins);
    } else if args.contains(&"esp32s3".to_string()) || args.contains(&"esp32s3".to_string()) {
        pins_opt = Some(esp32s2_pins);
    }

    let pins = pins_opt.expect("SoC not supported");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("adc.rs");

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dest_path.clone())
        .unwrap();

    for (pin, adc) in pins.iter() {
        let adc_driver = match adc {
            ADC::ADC1(_) => "adc1_driver",
            ADC::ADC2(_) => "adc2_driver",
            ADC::NOADC => "None",
        };
        let adc_str = match adc {
            ADC::ADC1(_) => "ADC1",
            ADC::ADC2(_) => "ADC2",
            ADC::NOADC => "None",
        };
        print!(
            file,
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
impl GpioPin for GpioPin{pin}<'_> {{"
        );

        if *adc != ADC::NOADC {
            print!(file, "
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError> {{
        if self.pin.is_some() {{
            let pin = self.pin.as_mut().unwrap();
            let mut adc_driver = self.{adc_driver}.lock().await;
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
    }}

    async fn get_adc_averaged( &mut self, measurement: MeasurementConfig) -> Result<f32, GpioWrapperError> {{
        if measurement.to_measure == 0 {{
            return Ok(f32::NAN);
        }}
        if self.pin.is_some() {{
            let pin = self.pin.as_mut().unwrap();
            let mut adc_driver = self.{adc_driver}.lock().await;
            if adc_driver.is_some() {{
                fn helper<const N: u32>(
                    pin: &mut Gpio{pin},
                    adc_driver: &mut tokio::sync::MutexGuard<Option<AdcDriver<{adc_str}>>>,
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
    }}
"
            );
        } else {
            print!(file, "
    async fn get_adc(&mut self) -> Result<u16, GpioWrapperError> {{
        Err(GpioWrapperError::NotAnAdcPin)
    }}

    async fn get_adc_averaged( &mut self, measurement: MeasurementConfig) -> Result<f32, GpioWrapperError> {{
        Err(GpioWrapperError::NotAnAdcPin)
    }}
");
        }
        print!(
            file,
            "
}}
"
        );
    }
    print!(file, "
fn pins_vec(adc1: Option<ADC1>, adc2: Option<ADC2>, pins: Pins) -> Vec<Arc<Mutex<Option<Box<dyn GpioPin>>>>> {{
        let adc1_driver = adc1.map(|adc1| {{
            AdcDriver::new(adc1, &hal::adc::config::Config::new().calibration(true)).unwrap()
        }});
        let adc2_driver = adc2.map(|adc2| {{
            AdcDriver::new(adc2, &hal::adc::config::Config::new().calibration(true)).unwrap()
        }});
        let adc1_ref: Arc<Mutex<Option<AdcDriver<ADC1>>>> = Arc::new(Mutex::new(adc1_driver));
        let adc2_ref = Arc::new(Mutex::new(adc2_driver));
    vec![
    ");
    for i in 0..pins.last().unwrap().0 {
        if pins.iter().any(|(pin, _)| *pin == i) {
            print!(
                file,
                "
        Arc::new(Mutex::new(Some(Box::new(GpioPin{i}::new(
                pins.gpio{i},
                adc1_ref.clone(),
                adc2_ref.clone(),
))))),"
            );
        } else {
            print!(
                file,
                "
        Arc::new(Mutex::new(None)),"
            );
        }
    }
    print!(
        file,
        "
    ]
}}
    "
    );
    println!("cargo::rerun-if-changed=build.rs");
}
