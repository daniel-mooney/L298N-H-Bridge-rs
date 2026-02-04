//! A basic example demonstrating the full functionality of the L298N H-Bridge
//! rust driver for the STM32F4. Tested on an STM32F411CEU6.
//!
//! A robot running this example is expected to initially drive forward,
//! before stopping, then spinning around on the spot.
//!
//! # Wiring
//! The L298N should be wired to the STM32 ports using the following mappings:
//! =============================
//! |   L298N   |   STM32F4     |
//! |-----------|---------------|
//! |   En A    |   PB5         |
//! |   I2      |   PB4         |
//! |   I1      |   PB3         |
//! |   En B    |   PA9         |
//! |   I4      |   PA8         |
//! |   I3      |   B15         |
//! =============================

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{pac, prelude::*};

use l298_hbridge::{L298NHBridge, Command, Direction, StopMode};

#[entry]
fn main() -> ! {
    // === Periperal setup ==================================================
    let dp = pac::Peripherals::take().unwrap(); 
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    // Configure GPIO
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let left_dir1 = gpiob.pb3.into_push_pull_output();
    let left_dir2 = gpiob.pb4.into_push_pull_output();

    let right_dir1 = gpioa.pa8.into_push_pull_output();
    let right_dir2 = gpiob.pb15.into_push_pull_output();

    // Setup PWM
    let (_, (_, tim1_ch2, ..)) = dp.TIM1.pwm_us(100.micros(), &mut rcc);
    let (_, (_, tim3_ch2, ..)) = dp.TIM3.pwm_us(100.micros(), &mut rcc);

    let mut left_enable = tim3_ch2.with(gpiob.pb5);
    left_enable.enable();

    let mut right_enable = tim1_ch2.with(gpioa.pa9);
    right_enable.enable();

    // === L298N setup ======================================================
    let mut left_motor = L298NHBridge::new(left_dir1, left_dir2, left_enable).unwrap();
    let mut right_motor = L298NHBridge::new(right_dir1, right_dir2, right_enable).unwrap();

    let mut delay = cp.SYST.delay(&rcc.clocks);
    delay.delay_ms(1000);

    // === Program Logic ====================================================
    left_motor.set(Command::Drive { direction: Direction::Forward, throttle: u16::MAX });
    right_motor.set(Command::Drive { direction: Direction::Forward, throttle: u16::MAX });

    delay.delay_ms(2000);

    left_motor.set(Command::Stop(StopMode::Coast));
    right_motor.set(Command::Stop(StopMode::Coast));

    delay.delay_ms(2000);
    
    // Spin right
    left_motor.set(Command::Drive { direction: Direction::Forward, throttle: 55000u16 });
    right_motor.set(Command::Drive { direction: Direction::Reverse, throttle: 55000u16 });

    delay.delay_ms(2000);

    // Spin left
    left_motor.set(Command::Drive { direction: Direction::Reverse, throttle: 55000u16 });
    right_motor.set(Command::Drive { direction: Direction::Forward, throttle: 55000u16 });

    delay.delay_ms(2000);

    left_motor.set(Command::Stop(StopMode::Brake));
    right_motor.set(Command::Stop(StopMode::Brake));

    // Loop forever
    loop { }
}
