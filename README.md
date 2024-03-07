# Microbit Freefall Detection

This code is for a Microbit that can detect when it is falling using an accelerometer. It uses the LSM303agr accelerometer and the microbit's display and speaker to indicate when it is falling.

## Dependencies

- `critical_section_lock_mut`
- `microbit`
- `panic_rtt_target`
- `rtt-target`
- `state`

## Code Overview

The code consists of several modules:

- `leds`: Defines the `Leds` enum, which represents the two possible states of the LEDs on the Microbit. (CenterLED/ExclamationMark)
- `speaker`: Defines the `Speaker` enum, which represents the two possible states of the speaker on the Microbit. (On/Off)
- `state`: Defines the `MB2` enum, which represents the two possible states of the Microbit: (Stable/Falling)
- `BoardAccel`: A struct that stores the acceleration data for the Microbit and provides methods for calculating the average acceleration over a number of samples and for determining if the Microbit is falling.
