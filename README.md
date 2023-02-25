# bma400-rs
A platform-agnostic Rust driver for the BMA400 accelrometer implemented using [`embedded-hal`](https://github.com/japaric/embedded-hal) traits

🚧 Under Development 🚧

## Status

- [x] Support Basic Sensor Features
- [ ] Support Programmable (Custom) Interrupts
- [ ] Tests
- [ ] SPI Support
- [ ] Documentation and More Examples

### About the Sensor (from the manufacturer)
#### Basic Description
12 bit, digital, triaxial acceleration sensor with smart on-chip motion and position-triggered interrupt features.

#### Key features
- Small Package Size 
  - LGA package (12 pins), footprint 2mm x 2mm, height 0.95 mm
- Ultra-low power
  - Low current consumption of data acquisition without compromising on performance (< 14.5 µA with highest performance)
- Programmable functionality
  - Acceleration ranges ±2g/±4g/±8g/±16g
  - Low-pass filter bandwidths = (0.24/0.48)*ODR up to a max. output data read out of 800Hz
- On-chip FIFO
  - Integrated FIFO on sensor with 1 KB
- On-chip interrupt features
  - Auto-low power/Auto wakeup
  - Activity/In-activity
  - Step Counter (overall device current consumption 4µA)
  - Activity Recognition (Walking, Running, Standing still)
  - Orientation detection
  - Tap/double tap
- Digital interface
  - SPI (4-wire, 3-wire)
  - I²C, 
  - 2 interrupt pins
- VDDIO voltage range: 1.2V to 3.6V
- RoHS compliant, halogen-free

#### Typical applications
- Step Counting with ultra-low current consumption for extensive battery lifetime
- Advanced system power management for mobile applications and (smart) watches
- Fitness applications / Activity Tracking
- Tap / double tap sensing
- Drop detection for warranty logging
- Window/door measurements for climate control and alarm systems
- IoT applications powered by coin cell driven batteries, requiring <1µA and auto-wakeup
functionality

## License
Licensed under your choice of either:
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT) 

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.