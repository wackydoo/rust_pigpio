#![crate_type = "lib"]
#![crate_name = "rust_pigpio"]
#![allow(dead_code)]
#![allow(non_snake_case)]

//! #Rust PiGPIO
//!
//! The Rust wrapper of the C library functions
pub mod pwm;
pub mod constants;

use std::string::String;

use constants::*;

const OK: i32 = 0;
const INIT_FAILED: i32 = -1;
const BAD_USER_GPIO: i32 = -2;
const BAD_GPIO: i32 = -3;
const BAD_MODE: i32 = -4;
const BAD_LEVEL: i32 = -5;
const BAD_PUD: i32 = -6;
const DEFAULT_ERROR: &str = "Unknown error.";

// wave tx mode 
const PI_WAVE_MODE_ONE_SHOT: u32 = 0;
const PI_WAVE_MODE_REPEAT: u32 = 1;
const PI_WAVE_MODE_ONE_SHOT_SYNC: u32 = 2;
const PI_WAVE_MODE_REPEAT_SYNC: u32 = 3;
// tx send results
const PI_BAD_WAVE_ID: i32 =   -66 ;// non existent wave id
const PI_BAD_WAVE_MODE:i32 =     -33; // waveform mode not 0-3


pub const INPUT: GpioMode = GpioMode::INPUT;
pub const OUTPUT: GpioMode = GpioMode::OUTPUT;

pub const ON: Level = Level::ON;
pub const OFF: Level = Level::OFF;

pub type GpioResult = Result<(), String>;
pub type GpioResponse = Result<u32, String>;

#[repr(C)]
#[derive(Debug)]
pub struct gpioPulse_t {
    pub gpioOn: u32,
    pub gpioOff: u32,
    pub usDelay: u32,
} 

pub type PulseTrain = Vec<gpioPulse_t>;

#[link(name = "pigpio", kind = "dylib")]
extern "C" {
    fn gpioInitialise() -> i32;
    fn gpioTerminate();

    fn gpioSetMode(gpio: u32, mode: u32) -> i32;
    fn gpioGetMode(gpio: u32) -> i32;
    fn gpioSetPullUpDown(gpio: u32, pud: u32) -> i32; //
    fn gpioRead(gpio: u32) -> i32;
    fn gpioWrite(gpio: u32, level: u32) -> i32;

    fn gpioDelay(micros: u32) -> u32;

    fn gpioSetAlertFunc(user_gpio: u32, alert_func: extern fn (u32, u32, u32)) -> i32;
    //    fn gpioSetAlertFuncEx(user_gpio: u32, f: gpioAlertFuncEx_t, void* userdata) -> i32;

    fn gpioTrigger(user_gpio: u32, pulseLen: u32, level: u32) -> i32; //
    fn gpioSetWatchdog(user_gpio: u32, timeout: u32) -> i32; //

    // ** waveform functions
    // clear all defined waveforms
    fn gpioWaveClear() -> i32; 
    // add waveform returning number of pulses
    // fn gpioWaveAddGeneric(numPulses: u32, gpioPulse_t *pulses) -> i32 ; 
    fn gpioWaveAddGeneric(numPulses: u32, pulses: *const gpioPulse_t) -> i32 ; 
    // create waveform loaded by WaveAddGeneric
    // returns waveid >= zero if successful
    fn gpioWaveCreate() -> i32; 
    //
    fn gpioWaveTxSend( wave_id: u32,  wave_mode: u32) -> i32;

    
}

pub fn wave_tx_send_once(wave_id: u32) -> GpioResult {
    let result: i32 = unsafe { gpioWaveTxSend(wave_id, PI_WAVE_MODE_ONE_SHOT) };
    match result {
        PI_BAD_WAVE_MODE => Err("Invalid mode in call to gpioWaveTxSend".to_string()),
        PI_BAD_WAVE_ID => Err("Invalide wave id in call to gpioWaveTxSend".to_string()),
        _ => Ok(())
    }
}

pub fn wave_create() -> GpioResponse {
    let result:i32 = unsafe { gpioWaveCreate() };
    match result {
        d if d >= 0 => Ok(result as u32),
        _ => Err(format!("Can't create wave ({})",result ))
    }
}

pub fn wave_clear() -> GpioResult {
    let result: i32 = unsafe {gpioWaveClear()};
    match result {
        0  => Ok(()),
        _ => Err("Error clearing in call to gpioWaveClear".to_string())
    }
}

pub fn wave_add_generic(numPulses: u32,  pulses: &PulseTrain) -> GpioResponse {
    const PI_TOO_MANY_PULSES:i32 =  -36;
    let result = unsafe {
        gpioWaveAddGeneric(numPulses, pulses.as_ptr()) };
    let expected_result = numPulses as i32;
    match result {
        PI_TOO_MANY_PULSES => Err("Initialize failed".to_string()),
        expected_result => Ok(result as u32),
        _ => Err(format!("Add waveform failed (unknown {}",result))
    }
}



/// Initializes the library.
///
/// Initialize must be called before using the other library functions with some exceptions not yet wrapped.
pub fn initialize() -> GpioResponse {
    let result = unsafe { gpioInitialise() };
    match result {
        INIT_FAILED => Err("Initialize failed".to_string()),
        _ => Ok(result as u32)
    }

}

/// Terminates the library.
///
/// Call before program exit.
/// This function resets the used DMA channels, releases memory, and terminates any running threads.
pub fn terminate() {
    unsafe { gpioTerminate() };
}

/// Sets the GPIO mode, typically input or output.
pub fn set_mode(gpio: u32, mode: GpioMode) -> GpioResult {
    match unsafe { gpioSetMode(gpio, mode as u32) } {
        OK => Ok(()),
        BAD_GPIO => Err("Bad gpio".to_string()),
        BAD_MODE => Err("Bad mode".to_string()),
        _ => Err(DEFAULT_ERROR.to_string()),
    }
}

/// Gets the GPIO mode.
pub fn get_mode(gpio: u32) -> GpioResponse {
    let response = unsafe { gpioGetMode(gpio) };
    match response {
        BAD_GPIO => Err("Bad gpio".to_string()),
        _ => Ok(response as u32),
    }
}

/// Sets or clears resistor pull ups or downs on the GPIO.
pub fn set_pull_up_down(gpio: u32, pud: Pud) -> GpioResult {
    match unsafe { gpioSetPullUpDown(gpio, pud as u32) } {
        OK => Ok(()),
        BAD_GPIO => Err("Bad gpio".to_string()),
        BAD_PUD => Err("Bad pud".to_string()),
        _ => Err(DEFAULT_ERROR.to_string())
    }
}

/// Reads the GPIO level, on or off.
pub fn read(gpio: u32) -> GpioResponse {
    match unsafe { gpioRead(gpio) } {
        1 => Ok(1),
        0 => Ok(0),
        BAD_GPIO => Err("Bad gpio".to_string()),
        _ => Err(DEFAULT_ERROR.to_string()),
    }
}

/// Sets the GPIO level, on or off.
/// If PWM or servo pulses are active on the GPIO they are switched off.
pub fn write(gpio: u32, level: Level) -> GpioResult {
    match unsafe { gpioWrite(gpio, level as u32) } {
        OK => Ok(()),
        BAD_GPIO => Err("Bad gpio".to_string()),
        BAD_LEVEL => Err("Bad level".to_string()),
        _ => Err(DEFAULT_ERROR.to_string()),
    }
}

/// Delays for at least the number of microseconds specified by microseconds.
pub fn delay(microseconds: u32) -> u32 {
    unsafe { gpioDelay(microseconds) }
}

/// Registers a function to be called (a callback) when the specified GPIO changes state
// http://abyz.me.uk/rpi/pigpio/cif.html#gpioSetAlertFunc
pub fn set_alert_func(gpio: u32, alert_func: extern fn(u32, u32, u32)) -> GpioResult {
    match unsafe { gpioSetAlertFunc(gpio, alert_func)} {
        OK => Ok(()),
        BAD_USER_GPIO => Err("Bad user gpio".to_string()),
        _ => Err(DEFAULT_ERROR.to_string()),
    }
}
