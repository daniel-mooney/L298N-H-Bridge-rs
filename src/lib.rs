#![deny(unsafe_code)]
#![no_std]

use embedded_hal::{digital, pwm};

/// Driver wrapper for one half of an **L298N** Dual Full-Bridge.
///
/// ## Wiring
/// - `dir1`/`dir2`: direction inputs (e.g. In1/In2) as GPIO push-pull outputs.
/// - `enable`: PWM output driving the enable pin (e.g. EnA).
///
/// The driver assumes when the enable pin is high, the direction inputs are wired such that:
/// |===================================|
/// | dir1  | dir2  | Function          |
/// |-------|-------|-------------------|
/// | H     | L     | Forward           |
/// | L     | H     | Reverse           |
/// | H     | H     | Fast Motor Stop   |
/// | L     | L     | Fast Motor Stop   |
/// |===================================|
///
/// A enable pin set to low result in a Free Running Motor Stop.
///
/// ## Type Parameters
/// - `P1, N1`: GPIO port letter and pin number for `dir1`.
/// - `P2, N2`: GPIO port letter and pin number for `dir2`.
/// - `TIM`: timer peripheral used to generate PWM.
/// - `C`: timer channel used for the PWM output.
pub struct L298NHBridge<P1, P2, EN>
where 
    P1: digital::OutputPin,
    P2: digital::OutputPin,
    EN: pwm::SetDutyCycle,
{
    dir1: P1,
    dir2: P2,
    enable: EN,
    throtte: u16,
}

/// Command sent to a motor driver.
///
/// The motor throttle is set using an `u16`: 
/// `0u16`      = stopped
/// `65535u16`  = maximum throttle
pub enum MotorFunction {
    /// Set motor throttle in a forward state
    Forward(u16),

    /// Set motor throttle in a reverse state
    Reverse(u16),

    /// Set motor to FastMotorStop mode
    FastMotorStop,

    /// Set motor to FreeRunningMotorStop mode
    FreeRunningMotorStop,
}

impl<P1, P2, EN> L298NHBridge<P1, P2, EN>
where 
    P1: digital::OutputPin,
    P2: digital::OutputPin,
    EN: pwm::SetDutyCycle,
{
    const STOP_THROTTLE: u16 = 0u16;

    pub fn new(dir1: P1, dir2: P2, enable: EN) -> Self {
        let mut  handle = Self { dir1, dir2, enable, throtte: 0u16 };
        handle.enable.set_duty_cycle(0u16).unwrap();

        handle
    }

    pub fn set_function(&mut self, function: MotorFunction) {
        // Set the mode
        match function {
             MotorFunction::Forward { .. } => self.forward(),
             MotorFunction::Reverse { .. } => self.reverse(),
             MotorFunction::FreeRunningMotorStop => self.free_running_motor_stop(),
             MotorFunction::FastMotorStop => self.fast_motor_stop(),
        } 
        
        // Set duty cycle
        let throttle = match function {
            MotorFunction::Forward ( throttle ) | MotorFunction::Reverse ( throttle ) => throttle,
            _ => Self::STOP_THROTTLE,
        };

        self.set_throttle(throttle);
    }

    pub fn get_throttle(&self) -> u16 {
        self.throtte
    }

    fn set_throttle(&mut self, throtte: u16) {
        self.throtte = throtte;

        let duty = self.duty_from_freescale(throtte);
        self.enable.set_duty_cycle(duty).ok();
    }

    fn duty_from_freescale(&self, throttle: u16) -> u16 {
        let max = self.enable.max_duty_cycle() as u32;
        let throttle = throttle as u32;

        ((max * throttle + 0x8000) / 0xFFFF) as u16
    }
    
    /// Sets the L298 into forward mode
    fn forward(&mut self) {
        self.dir1.set_high().unwrap();
        self.dir2.set_low().unwrap();
    }

    /// Sets the L298 into reverse mode
    fn reverse(&mut self) {
        self.dir1.set_low().unwrap();
        self.dir2.set_high().unwrap();
    }

    /// Sets the L298 into fast motor stop mode
    fn fast_motor_stop(&mut self) {
        self.dir1.set_high().unwrap();
        self.dir2.set_high().unwrap();
    }

    /// Sets the L298 into free running motor stop mode
    fn free_running_motor_stop(&mut self) {
        self.set_throttle(0u16);
    }
}
