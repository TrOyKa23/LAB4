#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::adc;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Ch2;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // --- PWM (RED LED на PA7 → TIM3_CH2) ---
    let red: PwmPin<_, Ch2> = PwmPin::new(p.PA7, OutputType::PushPull);

    let mut pwm = SimplePwm::new(
        p.TIM3,
        None,
        Some(red),
        None,
        None,
        khz(1),
        Default::default(),
    );

    let mut ch = pwm.ch2();
    ch.set_polarity(OutputPolarity::ActiveLow);
    ch.enable();

    // =========================================================
    // ЗАДАНИЕ 1a: LED на 25% яркости
    // =========================================================
    // ch.set_duty_cycle_percent(25);
    // loop {
    //     Timer::after_secs(1).await;
    // }

    // =========================================================
    // ЗАДАНИЕ 1b: яркость от 0% до 100% шагом 10% каждую секунду
    // =========================================================
    // loop {
    //     let mut duty: u8 = 0;
    //     while duty <= 100 {
    //         ch.set_duty_cycle_percent(duty);
    //         Timer::after_secs(1).await;
    //         duty += 10;
    //     }
    // }

    // =========================================================
    // ЗАДАНИЕ 2: потенциометр (ADC PA0) → яркость LED
    // =========================================================
    let mut adc = adc::Adc::new(p.ADC1);
    adc.set_resolution(adc::Resolution::BITS12);
    adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    let mut adc_pin = p.PA0;

    const MAX_ADC: u32 = 4095; // 12 бит = 0..4095

    loop {
        let level: u16 = adc.blocking_read(&mut adc_pin);
        let duty = (level as u32 * 100 / MAX_ADC) as u8;

        ch.set_duty_cycle_percent(duty);

        defmt::info!("ADC: {}, duty: {}%", level, duty);

        Timer::after_millis(100).await;
    }
}
