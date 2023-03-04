# bma400-nrf52833

An example project demonstrating the use of the bma400 driver

[`probe-run`]: https://crates.io/crates/probe-run
[`flip-link`]: https://github.com/knurling-rs/flip-link

Based on https://github.com/rtic-rs/defmt-app-template

## Cargo Dependencies

### 1. [`flip-link`]:

```console
cargo install flip-link
```

### 2. [`probe-run`]:

``` console
cargo install probe-run
```

## Setup

### 1. Make GPIO Connections

#### NRF52 -> BMA400
- ##### VDDIO -> CSB + VCC
- ##### GND -> GND + SDO
- ##### P1.00 -> SDA
- ##### P0.26 -> SCL
- ##### P0.10 -> INT1

### 2. Add the compilation target

Add the target with `rustup`.

``` console
rustup target add thumbv7em-none-eabihf
```

### 3. Run!

Connect to the serial debug and try running the sample binary `bma400-nrf52833/src/bin/taps.rs`. 
>( `rb` is an alias for `run --bin` )

>the env variable `DEFMT_LOG=info` tells the logger to emit messages of `INFO` severity or greater

``` console
    Finished dev [optimized + debuginfo] target(s) in 1.82s
     Running `probe-run --chip nRF52833_xxAA target/thumbv7em-none-eabihf/debug/taps`
(HOST) INFO  flashing program (4 pages / 16.00 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
INFO  Now Sensing Taps...
└─ taps::__cortex_m_rt_main::{closure#0} @ src/bin/taps.rs:70
INFO  Acceleration: x: 20, y: 25, z: 493
└─ taps::__cortex_m_rt_main::{closure#1} @ src/bin/taps.rs:83
INFO  Acceleration: x: 20, y: 21, z: 493
└─ taps::__cortex_m_rt_main::{closure#1} @ src/bin/taps.rs:83
INFO  Single tap detected!
└─ taps::__cortex_m_rt_GPIOTE::{closure#0} @ src/bin/taps.rs:109
INFO  Double tap detected!
└─ taps::__cortex_m_rt_GPIOTE::{closure#0} @ src/bin/taps.rs:106
INFO  Acceleration: x: 22, y: 21, z: 492
└─ taps::__cortex_m_rt_main::{closure#1} @ src/bin/taps.rs:83
```
If you tap near the device, the INT1 pin should set state to high, triggering the GPIOTE interrupt.


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
