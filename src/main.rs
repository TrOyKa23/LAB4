#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, OutputType, Pull};
use embassy_stm32::time::khz;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Ch1, Ch2, Ch4};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // --- PWM для RGB LED ---
    // RED   → PA7 → TIM3_CH2
    // GREEN → PC6 → TIM3_CH1
    // BLUE  → PC9 → TIM3_CH4
    let red: PwmPin<_, Ch2> = PwmPin::new(p.PA7, OutputType::PushPull);
    let green: PwmPin<_, Ch1> = PwmPin::new(p.PC6, OutputType::PushPull);
    let blue: PwmPin<_, Ch4> = PwmPin::new(p.PC9, OutputType::PushPull);

    let mut pwm = SimplePwm::new(
        p.TIM3,
        Some(green), // CH1 = GREEN
        Some(red),   // CH2 = RED
        None,
        Some(blue), // CH4 = BLUE
        khz(1),
        Default::default(),
    );

    let mut channels = pwm.split();

    channels.ch1.set_polarity(OutputPolarity::ActiveLow); // GREEN
    channels.ch2.set_polarity(OutputPolarity::ActiveLow); // RED
    channels.ch4.set_polarity(OutputPolarity::ActiveLow); // BLUE

    channels.ch1.enable();
    channels.ch2.enable();
    channels.ch4.enable();

    // =========================================================
    // ЗАДАНИЕ 1a: RED LED на 25% яркости
    // RED → ch2
    // =========================================================
    // channels.ch2.set_duty_cycle_percent(25);
    // loop {
    //     Timer::after_secs(1).await;
    // }

    // =========================================================
    // ЗАДАНИЕ 1b: яркость от 0% до 100% шагом 10% каждую секунду
    // =========================================================
    // loop {
    //     let mut duty: u8 = 0;
    //     while duty <= 100 {
    //         channels.ch2.set_duty_cycle_percent(duty);
    //         Timer::after_secs(1).await;
    //         duty += 10;
    //     }
    // }

    // =========================================================
    // ЗАДАНИЕ 2: потенциометр (ADC PA0) → яркость RED LED
    // =========================================================
    // use embassy_stm32::adc;
    // let mut adc = adc::Adc::new(p.ADC1);
    // adc.set_resolution(adc::Resolution::BITS12);
    // adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    // let mut adc_pin = p.PA0;
    // const MAX_ADC: u32 = 4095;
    // loop {
    //     let level: u16 = adc.blocking_read(&mut adc_pin);
    //     let duty = (level as u32 * 100 / MAX_ADC) as u8;
    //     channels.ch2.set_duty_cycle_percent(duty);
    //     defmt::info!("ADC: {}, duty: {}%", level, duty);
    //     Timer::after_millis(100).await;
    // }

    // =========================================================
    // ЗАДАНИЕ 3: RGB LED red → yellow → blue по кнопке S4 (PA8)
    // =========================================================
    let button = Input::new(p.PA8, Pull::Up);

    let mut color: u8 = 0;

    // Начальный цвет — Red
    channels.ch2.set_duty_cycle_percent(100); // RED on
    channels.ch1.set_duty_cycle_percent(0); // GREEN off
    channels.ch4.set_duty_cycle_percent(0); // BLUE off

    let mut last_pressed = false;

    loop {
        let pressed = button.is_low();

        if pressed && !last_pressed {
            color = (color + 1) % 3;

            match color {
                0 => {
                    // Red
                    channels.ch2.set_duty_cycle_percent(100);
                    channels.ch1.set_duty_cycle_percent(0);
                    channels.ch4.set_duty_cycle_percent(0);
                    defmt::info!("Color: RED");
                }
                1 => {
                    // Yellow = Red + Green
                    channels.ch2.set_duty_cycle_percent(100);
                    channels.ch1.set_duty_cycle_percent(100);
                    channels.ch4.set_duty_cycle_percent(0);
                    defmt::info!("Color: YELLOW");
                }
                2 => {
                    // Blue
                    channels.ch2.set_duty_cycle_percent(0);
                    channels.ch1.set_duty_cycle_percent(0);
                    channels.ch4.set_duty_cycle_percent(100);
                    defmt::info!("Color: BLUE");
                }
                _ => {}
            }
        }

        last_pressed = pressed;
        Timer::after_millis(20).await;
    }
}
