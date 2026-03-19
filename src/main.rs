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

    // --- PWM для RGB LED (TIM3) ---
    // RED   → PA7 → TIM3_CH2
    // GREEN → PC6 → TIM3_CH1
    // BLUE  → PC9 → TIM3_CH4
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

    let mut rgb = pwm3.split();
    rgb.ch1.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch2.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch4.set_polarity(OutputPolarity::ActiveLow);
    rgb.ch1.enable();
    rgb.ch2.enable();
    rgb.ch4.enable();

    // --- PWM для сервомотора (TIM2) ---
    // SIG → PB3 → TIM2_CH2 → D3
    let servo_pin: PwmPin<_, Ch2> = PwmPin::new(p.PB3, OutputType::PushPull);

    let mut pwm2 = SimplePwm::new(
        p.TIM2,
        None,
        Some(servo_pin),
        None,
        None,
        embassy_stm32::time::hz(50), // 50 Hz = 20ms период для сервы
        Default::default(),
    );

    let mut servo = pwm2.split();
    servo.ch2.enable();

    // =========================================================
    // ЗАДАНИЕ 1a: RED LED на 25% яркости
    // =========================================================
    // rgb.ch2.set_duty_cycle_percent(25);
    // loop { Timer::after_secs(1).await; }

    // =========================================================
    // ЗАДАНИЕ 1b: яркость от 0% до 100% шагом 10%
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
    // ЗАДАНИЕ 2: потенциометр (ADC A0=PA0) → яркость RED LED
    // =========================================================
    // let mut adc = adc::Adc::new(p.ADC1);
    // adc.set_resolution(adc::Resolution::BITS12);
    // adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    // let mut adc_pin = p.PA0;
    // const MAX_ADC: u32 = 4095;
    // loop {
    //     let level: u16 = adc.blocking_read(&mut adc_pin);
    //     let duty = (level as u32 * 100 / MAX_ADC) as u8;
    //     rgb.ch2.set_duty_cycle_percent(duty);
    //     defmt::info!("ADC: {}, duty: {}%", level, duty);
    //     Timer::after_millis(100).await;
    // }

    // =========================================================
    // ЗАДАНИЕ 3: RGB LED red → yellow → blue по кнопке S4 (PA8)
    // =========================================================
    // let button = Input::new(p.PA8, Pull::Up);
    // let mut color: u8 = 0;
    // rgb.ch2.set_duty_cycle_percent(100);
    // rgb.ch1.set_duty_cycle_percent(0);
    // rgb.ch4.set_duty_cycle_percent(0);
    // let mut last_pressed = false;
    // loop {
    //     let pressed = button.is_low();
    //     if pressed && !last_pressed {
    //         color = (color + 1) % 3;
    //         match color {
    //             0 => {
    //                 rgb.ch2.set_duty_cycle_percent(100);
    //                 rgb.ch1.set_duty_cycle_percent(0);
    //                 rgb.ch4.set_duty_cycle_percent(0);
    //                 defmt::info!("Color: RED");
    //             }
    //             1 => {
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
    //     Timer::after_millis(20).await;
    // }

    // =========================================================
    // ЗАДАНИЕ 4: фоторезистор (ADC A2=PA4) → цвет RGB LED
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
    // ЗАДАНИЕ 5: сервомотор 0° → 180° → 0° в цикле
    // SIG → PB3 → TIM2_CH2 (D3)
    // 50 Hz → период 20ms
    // 0°   = 0.5ms pulse → set_duty_cycle_fraction(25, 1000)
    // 180° = 2.5ms pulse → set_duty_cycle_fraction(125, 1000)
    // =========================================================
    loop {
        // 0 градусов
        servo.ch2.set_duty_cycle_fraction(25, 1000);
        defmt::info!("Servo: 0 degrees");
        Timer::after_secs(1).await;

        // 180 градусов
        servo.ch2.set_duty_cycle_fraction(125, 1000);
        defmt::info!("Servo: 180 degrees");
        Timer::after_secs(1).await;
    }
}
