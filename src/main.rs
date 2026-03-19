#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::adc;
use embassy_stm32::gpio::{Input, OutputType, Pull};
use embassy_stm32::time::khz;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Ch1, Ch2, Ch3, Ch4};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Set up PWM for the RGB LED using Timer 3
    // Each color channel maps to a specific STM32 pin and timer channel:
    //   RED   → PA7 → TIM3_CH2
    //   GREEN → PC6 → TIM3_CH1
    //   BLUE  → PC9 → TIM3_CH4
    let red: PwmPin<_, Ch2> = PwmPin::new(p.PA7, OutputType::PushPull);
    let green: PwmPin<_, Ch1> = PwmPin::new(p.PC6, OutputType::PushPull);
    let blue: PwmPin<_, Ch4> = PwmPin::new(p.PC9, OutputType::PushPull);

    let mut pwm3 = SimplePwm::new(
        p.TIM3,
        Some(green),
        Some(red),
        None,
        Some(blue),
        khz(1),
        Default::default(),
    );

    // Split into individual channels and configure polarity.
    // The RGB LED on this board is common-anode, so we invert the signal:
    // 0% duty = fully ON, 100% duty = fully OFF
    let mut rgb = pwm3.split();
    rgb.ch1.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch2.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch4.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch1.enable();
    rgb.ch2.enable();
    rgb.ch4.enable();

    // Set up PWM for the servo motor using Timer 2
    // The signal wire connects to: PB3 → TIM2_CH2 → Arduino pin D3
    // Servo expects 50 Hz (one pulse every 20 ms)
    let servo_pin: PwmPin<_, Ch2> = PwmPin::new(p.PB3, OutputType::PushPull);

    let mut pwm2 = SimplePwm::new(
        p.TIM2,
        None,
        Some(servo_pin),
        None,
        None,
        embassy_stm32::time::hz(50),
        Default::default(),
    );

    let mut servo = pwm2.split();
    servo.ch2.enable();

    // =========================================================
    // TASK 1a: Light up the RED LED at 25% brightness
    // =========================================================
    // rgb.ch2.set_duty_cycle_percent(25);
    // loop { Timer::after_secs(1).await; }

    // =========================================================
    // TASK 1b: Fade the RED LED from 0% to 100% in 10% steps,
    // changing every second
    // =========================================================
    // loop {
    //     let mut duty: u8 = 0;
    //     while duty <= 100 {
    //         rgb.ch2.set_duty_cycle_percent(duty);
    //         Timer::after_secs(1).await;
    //         duty += 10;
    //     }
    // }

    // =========================================================
    // TASK 2: Read the potentiometer via ADC (A0 = PA0) and
    // use it to control the brightness of the RED LED
    // =========================================================
    // let mut adc = adc::Adc::new(p.ADC1);
    // adc.set_resolution(adc::Resolution::BITS12);
    // adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    // let mut adc_pin = p.PA0;
    // const MAX_ADC: u32 = 4095; // 12-bit ADC max value
    // loop {
    //     let level: u16 = adc.blocking_read(&mut adc_pin);
    //     let duty = (level as u32 * 100 / MAX_ADC) as u8;
    //     rgb.ch2.set_duty_cycle_percent(duty);
    //     defmt::info!("ADC: {}, duty: {}%", level, duty);
    //     Timer::after_millis(100).await;
    // }

    // =========================================================
    // TASK 3: Cycle RGB LED through red → yellow → blue
    // each time button S4 (PA8 = D7) is pressed
    // =========================================================
    // let button = Input::new(p.PA8, Pull::Up);
    // let mut color: u8 = 0;
    // rgb.ch2.set_duty_cycle_percent(100); // start with red
    // rgb.ch1.set_duty_cycle_percent(0);
    // rgb.ch4.set_duty_cycle_percent(0);
    // let mut last_pressed = false;
    // loop {
    //     let pressed = button.is_low(); // button pulls low when pressed
    //     if pressed && !last_pressed {  // detect the moment of press
    //         color = (color + 1) % 3;
    //         match color {
    //             0 => {
    //                 rgb.ch2.set_duty_cycle_percent(100);
    //                 rgb.ch1.set_duty_cycle_percent(0);
    //                 rgb.ch4.set_duty_cycle_percent(0);
    //                 defmt::info!("Color: RED");
    //             }
    //             1 => {
    //                 // Yellow = red + green mixed together
    //                 rgb.ch2.set_duty_cycle_percent(100);
    //                 rgb.ch1.set_duty_cycle_percent(100);
    //                 rgb.ch4.set_duty_cycle_percent(0);
    //                 defmt::info!("Color: YELLOW");
    //             }
    //             2 => {
    //                 rgb.ch2.set_duty_cycle_percent(0);
    //                 rgb.ch1.set_duty_cycle_percent(0);
    //                 rgb.ch4.set_duty_cycle_percent(100);
    //                 defmt::info!("Color: BLUE");
    //             }
    //             _ => {}
    //         }
    //     }
    //     last_pressed = pressed;
    //     Timer::after_millis(20).await; // small delay to debounce the button
    // }

    // =========================================================
    // TASK 4: Read light intensity from the photoresistor (A2 = PA4)
    // and change the RGB LED color based on how bright it is:
    //   dark  (0..1365)    → RED
    //   medium (1366..2730) → GREEN
    //   bright (2731..4095) → BLUE
    // =========================================================
    // let mut adc = adc::Adc::new(p.ADC1);
    // adc.set_resolution(adc::Resolution::BITS12);
    // adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    // let mut photo_pin = p.PA4;
    // loop {
    //     let level: u16 = adc.blocking_read(&mut photo_pin);
    //     defmt::info!("Photoresistor: {}", level);
    //     if level < 1365 {
    //         rgb.ch2.set_duty_cycle_percent(100);
    //         rgb.ch1.set_duty_cycle_percent(0);
    //         rgb.ch4.set_duty_cycle_percent(0);
    //         defmt::info!("Color: RED (low light)");
    //     } else if level < 2731 {
    //         rgb.ch2.set_duty_cycle_percent(0);
    //         rgb.ch1.set_duty_cycle_percent(100);
    //         rgb.ch4.set_duty_cycle_percent(0);
    //         defmt::info!("Color: GREEN (mid light)");
    //     } else {
    //         rgb.ch2.set_duty_cycle_percent(0);
    //         rgb.ch1.set_duty_cycle_percent(0);
    //         rgb.ch4.set_duty_cycle_percent(100);
    //         defmt::info!("Color: BLUE (high light)");
    //     }
    //     Timer::after_millis(200).await;
    // }

    // =========================================================
    // TASK 5: Move the servo motor smoothly between 0° and 180°
    // in a continuous loop.
    //
    // Servo is connected to: PB3 → TIM2_CH2 → D3
    // At 50 Hz, the period is 20 ms. Pulse width controls angle:
    //   0°   = 0.5 ms pulse → duty fraction 25/1000
    //   180° = 2.5 ms pulse → duty fraction 125/1000
    // =========================================================
    loop {
        servo.ch2.set_duty_cycle_fraction(25, 1000);
        defmt::info!("Servo: 0 degrees");
        Timer::after_secs(1).await;

        servo.ch2.set_duty_cycle_fraction(125, 1000);
        defmt::info!("Servo: 180 degrees");
        Timer::after_secs(1).await;
    }
}
