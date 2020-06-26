// ----------------------------------------------------------------------------
// Facade implementation of FTDI functions using functionality provided by
// a basic implmentation of FTDI library.
// NB! functionality below is by no way complete - only minimal functionality
// required by PCILeech use is implemented ...
// ----------------------------------------------------------------------------

// https://github.com/ufrisk/LeechCore/blob/0c1aa94be8ad32ece13fb2532a4cfccb9254b694/leechcore/oscompatibility.c#L186

mod chip;
pub use chip::{ChannelConfig, FifoMode, OptionalFeatureSupport};

use rusb::{
    request_type, DeviceHandle, DeviceList, Direction, GlobalContext, Recipient, RequestType,
};

use std::mem::size_of;
use std::time::Duration;

use log::{info, trace, warn};

use flow_core::error::{Error, Result};

use dataview::Pod;

pub const FTDI_VENDOR_ID: u16 = 0x0403;
pub const FTDI_FT60X_PRODUCT_ID: u16 = 0x601f;

// TODO: enum?
pub const FTDI_COMMUNICATION_INTERFACE: u8 = 0x00;
pub const FTDI_DATA_INTERFACE: u8 = 0x01;

// TODO: enum?
pub const FTDI_ENDPOINT_SESSION_OUT: u8 = 0x01;
pub const FTDI_ENDPOINT_OUT: u8 = 0x02;
pub const FTDI_ENDPOINT_IN: u8 = 0x82;

pub struct FT60x {
    handle: DeviceHandle<GlobalContext>,
}

impl FT60x {
    pub fn new() -> Result<Self> {
        let (dev, desc) = DeviceList::new()
            .map_err(|_| Error::Connector("unable to get usb device list"))?
            .iter()
            .filter_map(|dev| match dev.device_descriptor() {
                Ok(desc) => Some((dev, desc)),
                Err(_) => None,
            })
            .find(|(_dev, desc)| {
                desc.vendor_id() == FTDI_VENDOR_ID && desc.product_id() == FTDI_FT60X_PRODUCT_ID
            })
            .ok_or_else(|| Error::Connector("unable to find ftdi device"))?;

        info!(
            "found FTDI device: {}:{} (bus {}, device {})",
            desc.vendor_id(),
            desc.product_id(),
            dev.bus_number(),
            dev.address()
        );

        // open handle and reset chip
        let mut handle = dev
            .open()
            .map_err(|_| Error::Connector("unable to open ftdi usb device"))?;
        handle
            .reset()
            .map_err(|_| Error::Connector("unable to reset ftdi device"))?;

        /*
        let manufacturer = handle
            .read_string_descriptor_ascii(desc.manufacturer_string_index().unwrap_or_default())
            .map_err(|_| Error::Connector("unable to read ftdi manufacturer name"))?;
        let product = handle
            .read_string_descriptor_ascii(desc.product_string_index().unwrap_or_default())
            .map_err(|_| Error::Connector("unable to read ftdi product name"))?;
        let serial = handle
            .read_string_descriptor_ascii(desc.serial_number_string_index().unwrap_or_default())
            .map_err(|_| Error::Connector("unable to read ftdi serial number"))?;
        info!(
            "device: manufacturer={} product={} serial={}",
            manufacturer, product, serial
        );
        */

        // check driver state
        if handle
            .kernel_driver_active(FTDI_COMMUNICATION_INTERFACE)
            .map_err(|_| Error::Connector("ftdi driver check failed"))?
        {
            return Err(Error::Connector(
                "ftdi driver is already active on FTDI_COMMUNICATION_INTERFACE",
            ));
        }
        info!("ftdi driver is not active on FTDI_COMMUNICATION_INTERFACE");

        if handle
            .kernel_driver_active(FTDI_DATA_INTERFACE)
            .map_err(|_| Error::Connector("ftdi driver check failed"))?
        {
            return Err(Error::Connector(
                "ftdi driver is already active on FTDI_DATA_INTERFACE",
            ));
        }
        info!("ftdi driver is not active on FTDI_DATA_INTERFACE");

        // claim interfaces
        handle
            .claim_interface(FTDI_COMMUNICATION_INTERFACE)
            .map_err(|_| Error::Connector("unable to claim FTDI_COMMUNICATION_INTERFACE"))?;
        handle
            .claim_interface(FTDI_DATA_INTERFACE)
            .map_err(|_| Error::Connector("unable to claim FTDI_DATA_INTERFACE"))?;

        Ok(Self { handle })
    }

    pub fn abort_pipe(&mut self, pipe_id: u8) -> Result<()> {
        // dummy function, only used for ftdi compat
        trace!("abort_pipe: {}", pipe_id);
        Ok(())
    }

    pub fn set_suspend_timeout(&mut self, timeout: Duration) -> Result<()> {
        // dummy function, only used for ftdi compat
        trace!("set_suspend_timeout: {:?}", timeout);
        Ok(())
    }

    /// Retrieves the FT60x chip configuration
    pub fn config(&mut self) -> Result<chip::Config> {
        let mut buf = vec![0u8; size_of::<chip::Config>()];
        self.handle
            .read_control(
                request_type(Direction::In, RequestType::Vendor, Recipient::Device),
                0xCF,
                1,
                0,
                &mut buf,
                Duration::from_millis(1000),
            )
            .map_err(|_| Error::Connector("unable to get ft60x config"))?;

        // dataview buf to struct
        let view = buf.as_data_view();
        Ok(view.copy::<chip::Config>(0))
    }

    /// Writes the FT60x chip configuration
    pub fn set_config(&mut self, conf: &chip::Config) -> Result<()> {
        let bytes = self
            .handle
            .write_control(
                request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
                0xCF,
                0,
                0,
                conf.as_bytes(),
                Duration::from_millis(1000),
            )
            .map_err(|_| Error::Connector("unable to set ft60x config"))?;
        if bytes == size_of::<chip::Config>() {
            Ok(())
        } else {
            Err(Error::Connector("unable to set ft60x config"))
        }
    }

    // TODO: impl for pod? + _raw
    pub fn write_pipe(&mut self, data: &[u8]) -> Result<()> {
        self.write_bulk_raw(FTDI_ENDPOINT_OUT, data)
    }

    // TODO: implement this in a blocking manner
    // TODO: impl for pod? + _raw
    pub fn read_pipe(&mut self, data: &mut [u8]) -> Result<usize> {
        self.send_read_request(data.len() as u32)?;
        self.handle.read_bulk(FTDI_ENDPOINT_IN, data, Duration::from_millis(1000)).map_err(|_| Error::Connector("unable to read from ft60x"))
    }

    /// Sends a ControlRequest to issue a read with a given size
    fn send_read_request(&mut self, len: u32) -> Result<()> {
        let req = chip::ControlRequest::new(1, FTDI_ENDPOINT_IN, 1, len);
        self.write_bulk_raw(FTDI_ENDPOINT_SESSION_OUT, req.as_bytes())
    }

    // Does a bulk write and validates the sent size
    fn write_bulk_raw(&self, endpoint: u8, buf: &[u8]) -> Result<()> {
        // TODO: customizable write_bulk timeout
        let bytes = self.handle.write_bulk(endpoint, buf, Duration::from_millis(1000)).map_err(|_| Error::Connector("unable to write to ft60x"))?;
        if bytes == buf.len() {
            Ok(())
        } else {
            Err(Error::Connector("unable to write the entire buffer to the ft60x"))
        }
    }
}
