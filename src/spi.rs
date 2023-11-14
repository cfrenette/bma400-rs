use embedded_hal::digital::OutputPin;

use crate::{
    hal::spi::{SpiBus, SpiDevice},
    interface::{ReadFromRegister, WriteToRegister},
    registers::{ChipId, ConfigReg, InterfaceConfig, ReadReg},
    BMA400Error, Config, BMA400,
};
#[cfg(feature = "async")]
use crate::{
    hal_async::spi::SpiBus as AsyncSpiBus,
    hal_async::spi::SpiDevice as AsyncSpiDevice,
    interface::{AsyncReadFromRegister, AsyncWriteToRegister},
    AsyncBMA400,
};

/// SPI Interface wrapper
// Wrapper class to instantiate BMA400 with an SPI interface
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct SPIInterface<SPI, CSBPin> {
    spi: SPI,
    csb: CSBPin,
}

impl<SPI, CSBPin> SPIInterface<SPI, CSBPin> {
    /// Consumes the Interface returning underlying SPI peripheral and the pin
    pub fn destroy(self) -> (SPI, CSBPin) {
        (self.spi, self.csb)
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> WriteToRegister for SPIInterface<SPI, CSBPin>
where
    SPI: SpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.csb
            .set_low()
            .map_err(BMA400Error::ChipSelectPinError)?;
        self.spi
            .write(&[register.addr(), register.to_byte()])
            .map_err(BMA400Error::IOError)?;
        self.csb
            .set_high()
            .map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> ReadFromRegister for SPIInterface<SPI, CSBPin>
where
    SPI: SpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.csb
            .set_low()
            .map_err(BMA400Error::ChipSelectPinError)?;
        self.spi
            .transfer(&mut [0, 0], &[register.addr() | 1 << 7, 0])
            .map_err(BMA400Error::IOError)?;
        self.spi
            .transfer_in_place(buffer)
            .map_err(BMA400Error::IOError)?;
        self.csb
            .set_high()
            .map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, CSBPin, InterfaceError, PinError> AsyncWriteToRegister for SPIInterface<SPI, CSBPin>
where
    SPI: AsyncSpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    async fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.csb
            .set_low()
            .map_err(BMA400Error::ChipSelectPinError)?;
        self.spi
            .write(&[register.addr(), register.to_byte()])
            .await
            .map_err(BMA400Error::IOError)?;
        self.csb
            .set_high()
            .map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, CSBPin, InterfaceError, PinError> AsyncReadFromRegister for SPIInterface<SPI, CSBPin>
where
    SPI: AsyncSpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    async fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.csb
            .set_low()
            .map_err(BMA400Error::ChipSelectPinError)?;
        self.spi
            .transfer(&mut [0, 0], &[register.addr() | 1 << 7, 0])
            .await
            .map_err(BMA400Error::IOError)?;
        self.spi
            .transfer_in_place(buffer)
            .await
            .map_err(BMA400Error::IOError)?;
        self.csb
            .set_high()
            .map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> BMA400<SPIInterface<SPI, CSBPin>>
where
    SPI: SpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    /// Create a new instance of the BMA400 using 4-wire SPI
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::{
    /// # spi::{Mock, Transaction},
    /// # pin::{Mock as MockPin, Transaction as PinTransaction, State},
    /// # };
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
    /// #   Transaction::transfer_in_place(vec![0x00], vec![0x00]),
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
    /// #   Transaction::transfer_in_place(vec![0x00], vec![0x90]),
    /// # ];
    /// # let expected_pin = vec![
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// # ];
    /// # let mut spi = Mock::new(&expected_io);
    /// # let mut csb_pin = MockPin::new(&expected_pin);
    /// // spi implements embedded-hal spi::Transfer and spi::Write
    /// // csb_pin implements embedded-hal digital::v2::OutputPin
    /// let mut accelerometer = BMA400::new_spi(&mut spi, &mut csb_pin);
    /// assert!(accelerometer.is_ok());
    /// # spi.done();
    /// # csb_pin.done();
    /// ```
    pub fn new_spi(
        spi: SPI,
        csb: CSBPin,
    ) -> Result<BMA400<SPIInterface<SPI, CSBPin>>, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1])?;
        // Validate Chip ID
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }

    /// Create a new instance of the BMA400 using 3-wire SPI
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::{
    /// # spi::{Mock, Transaction},
    /// # pin::{Mock as MockPin, Transaction as PinTransaction, State},
    /// # };
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
    /// #   Transaction::transfer_in_place(vec![0x00], vec![0x00]),
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
    /// #   Transaction::transfer_in_place(vec![0x00], vec![0x90]),
    /// #   Transaction::write_vec(vec![0x7C, 0x01]),
    /// # ];
    /// # let expected_pin = vec![
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// # ];
    /// # let mut spi = Mock::new(&expected_io);
    /// # let mut csb_pin = MockPin::new(&expected_pin);
    /// // spi implements embedded-hal spi::Transfer and spi::Write
    /// // csb_pin implements embedded-hal digital::v2::OutputPin
    /// let mut accelerometer = BMA400::new_spi_3wire(&mut spi, &mut csb_pin);
    /// assert!(accelerometer.is_ok());
    /// # spi.done();
    /// # csb_pin.done();
    /// ```
    pub fn new_spi_3wire(
        spi: SPI,
        csb: CSBPin,
    ) -> Result<BMA400<SPIInterface<SPI, CSBPin>>, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1])?;
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        let if_config = InterfaceConfig::default().with_spi_3wire_mode(true);
        interface.write_register(if_config)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
}

#[cfg(any(docsrs, feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, CSBPin, InterfaceError, PinError> AsyncBMA400<SPIInterface<SPI, CSBPin>>
where
    SPI: AsyncSpiBus<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    /// async equivalent to [`BMA400::new_spi`]
    pub async fn new_spi(
        spi: SPI,
        csb: CSBPin,
    ) -> Result<Self, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1]).await?;
        // Validate Chip ID
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id).await?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(Self { interface, config })
        }
    }
    /// async equivalent to [`BMA400::new_spi_3wire`]
    pub async fn new_spi_3wire(
        spi: SPI,
        csb: CSBPin,
    ) -> Result<Self, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1]).await?;
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id).await?;
        let if_config = InterfaceConfig::default().with_spi_3wire_mode(true);
        interface.write_register(if_config).await?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(Self { interface, config })
        }
    }
}

#[derive(Debug)]
struct SPIDevice<SPI> {
    device: SPI,
}

impl<SPI, DeviceError> WriteToRegister for SPIDevice<SPI>
where
    SPI: SpiDevice<u8, Error = DeviceError>,
{
    type Error = BMA400Error<DeviceError, ()>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.device
            .write(&[register.addr(), register.to_byte()])
            .map_err(BMA400Error::IOError)
    }
}

impl<SPI, DeviceError> ReadFromRegister for SPIDevice<SPI>
where
    SPI: SpiDevice<u8, Error = DeviceError>,
{
    type Error = BMA400Error<DeviceError, ()>;
    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.device
            .transfer(&mut [0, 0], &[register.addr() | 1 << 7, 0])
            .map_err(BMA400Error::IOError)?;
        self.device
            .transfer_in_place(buffer)
            .map_err(BMA400Error::IOError)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, DeviceError> AsyncWriteToRegister for SPIDevice<SPI>
where
    SPI: AsyncSpiDevice<u8, Error = DeviceError>,
{
    type Error = BMA400Error<DeviceError, ()>;

    async fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.device
            .write(&[register.addr(), register.to_byte()])
            .await
            .map_err(BMA400Error::IOError)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, DeviceError> AsyncReadFromRegister for SPIDevice<SPI>
where
    SPI: AsyncSpiDevice<u8, Error = DeviceError>,
{
    type Error = BMA400Error<DeviceError, ()>;

    async fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.device
            .transfer(&mut [0, 0], &[register.addr() | 1 << 7, 0])
            .await
            .map_err(BMA400Error::IOError)?;
        self.device
            .transfer_in_place(buffer)
            .await
            .map_err(BMA400Error::IOError)
    }
}

impl<SPI, InterfaceError> BMA400<SPIDevice<SPI>>
where
    SPI: SpiDevice<u8, Error = InterfaceError>,
{
    /// Create a new instance of the BMA400 using a SPI device (an SPI peripheral with a CS pin,
    /// where the SPI bus is shared between multiple devices).
    pub fn new_spi_device(
        device: SPI,
    ) -> Result<BMA400<SPIDevice<SPI>>, BMA400Error<InterfaceError, ()>> {
        let mut interface = SPIDevice { device };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0])?;
        // Validate Chip ID
        let mut chip_id = [0];
        interface.read_register(ChipId, &mut chip_id)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
}

#[cfg(any(docsrs, feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<SPI, InterfaceError> AsyncBMA400<SPIDevice<SPI>>
where
    SPI: AsyncSpiDevice<u8, Error = InterfaceError>,
{
    /// async equivalent to [`BMA400::new_spi_device`]
    pub async fn new_spi_device(device: SPI) -> Result<Self, BMA400Error<InterfaceError, ()>> {
        let mut interface = SPIDevice { device };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0]).await?;
        // Validate Chip ID
        let mut chip_id = [0];
        interface.read_register(ChipId, &mut chip_id).await?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(Self { interface, config })
        }
    }
}
