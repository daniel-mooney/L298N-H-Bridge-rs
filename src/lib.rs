#![deny(unsafe_code)]
#![no_std]

use embedded_hal::{digital, pwm};
use core::convert::Infallible;

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
    P1: digital::OutputPin<Error = Infallible>,
    P2: digital::OutputPin<Error = Infallible>,
    EN: pwm::SetDutyCycle<Error = Infallible>,
{
    dir1: P1,
    dir2: P2,
    enable: EN,
    throttle: u16,
}

/// A `Command` sent to a motor driver
pub enum Command {
    Drive { direction: Direction, throttle: u16 },
    Stop(StopMode),
}

/// The direction of the H-Bridge
pub enum Direction { Forward, Reverse }

/// Each `StopMode` variant maps to a stop mode specified in the datasheet:
/// - Brake -> Fast Motor Stop
/// - Coast -> Free Running Motor Stop
pub enum StopMode { Brake, Coast }

impl<P1, P2, EN> L298NHBridge<P1, P2, EN>
where 
    P1: digital::OutputPin<Error = Infallible>,
    P2: digital::OutputPin<Error = Infallible>,
    EN: pwm::SetDutyCycle<Error = Infallible>,
{

    pub fn new(dir1: P1, dir2: P2, enable: EN) -> Result<Self,Infallible> {
        let mut  handle = Self { dir1, dir2, enable, throttle: 0u16 };
        handle.enable.set_duty_cycle(0u16)?;

        Ok(handle)
    }

    pub fn set(&mut self, cmd: Command) -> Result<(), Infallible> {
        match cmd {
            Command::Drive { direction, throttle } => {
                match direction {
                    Direction::Forward => self.forward()?,
                    Direction::Reverse => self.reverse()?,
                }

                self.set_throttle(throttle)?;
            },
            Command::Stop(stop_mode) => {
                match stop_mode {
                    StopMode::Brake => self.fast_motor_stop()?,
                    StopMode::Coast => self.free_running_motor_stop()?,
                }
            }
        }
        Ok(())
    }

    pub fn get_throttle(&self) -> u16 {
        self.throttle
    }

    fn set_throttle(&mut self, throttle: u16) -> Result<(), Infallible> {
        self.throttle = throttle;

        let duty = self.duty_from_fullscale(throttle);
        self.enable.set_duty_cycle(duty)?;

        Ok(())
    }

    fn duty_from_fullscale(&self, throttle: u16) -> u16 {
        let max = self.enable.max_duty_cycle() as u32;
        let throttle = throttle as u32;

        ((max * throttle + 0x8000) / 0xFFFF) as u16
    }
    
    /// Sets the L298 into forward mode
    fn forward(&mut self) -> Result<(), Infallible> {
        self.dir1.set_high()?;
        self.dir2.set_low()?;

        Ok(())
    }

    /// Sets the L298 into reverse mode
    fn reverse(&mut self) -> Result<(), Infallible> {
        self.dir1.set_low()?;
        self.dir2.set_high()?;

        Ok(())
    }

    /// Sets the L298 into fast motor stop mode
    fn fast_motor_stop(&mut self) -> Result<(), Infallible> {
        self.dir1.set_high()?;
        self.dir2.set_high()?;
        self.set_throttle(u16::MAX)?;

        Ok(())
    }

    /// Sets the L298 into free running motor stop mode
    fn free_running_motor_stop(&mut self) -> Result<(), Infallible> {
        self.set_throttle(0u16)?;
        Ok(())
    }
}
