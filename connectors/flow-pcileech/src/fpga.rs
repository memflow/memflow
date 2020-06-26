use crate::ft60x::*;

use std::mem::MaybeUninit;
use std::time::Duration;

use log::{info, trace, warn};

use flow_core::{
    error::{Error, Result},
    size
};

use dataview::Pod;

pub const FPGA_CONFIG_CORE:u16=                0x0003;
pub const FPGA_CONFIG_PCIE :u16=               0x0001;
pub const FPGA_CONFIG_SPACE_READONLY:u16=      0x0000;
pub const FPGA_CONFIG_SPACE_READWRITE    :u16= 0x8000;


pub struct Device {
    ft60: FT60x,
}

impl Device {
    pub fn new() -> Result<Self> {
        let mut ft60 = FT60x::new()?;
        ft60.abort_pipe(0x02)?;
        ft60.abort_pipe(0x82)?;

        ft60.set_suspend_timeout(Duration::new(0, 0))?;

        // check chip configuration
        let mut conf = ft60.config()?;
        trace!(
            "ft60x config: fifo_mode={} channel_config={} optional_feature={}",
            conf.fifo_mode,
            conf.channel_config,
            conf.optional_feature_support
        );

        if conf.fifo_mode != FifoMode::Mode245 as i8
            || conf.channel_config != ChannelConfig::Config1 as i8
            || conf.optional_feature_support != OptionalFeatureSupport::DisableAll as i16
        {
            warn!("bad ft60x config, reconfiguring chip");

            conf.fifo_mode = FifoMode::Mode245 as i8;
            conf.channel_config = ChannelConfig::Config1 as i8;
            conf.optional_feature_support = OptionalFeatureSupport::DisableAll as i16;

            ft60.set_config(&conf)?;
        } else {
            info!("ft60x config is valid");
        }

        Ok(Self { ft60 })
    }

    pub fn get_version(&mut self) -> Result<()> {
        // DeviceFPGA_GetDeviceId_FpgaVersion_ClearPipe
        self.get_version_clear_pipe()?;

        self.try_get_version_v4()?;

        Ok(())
    }

    fn get_version_clear_pipe(&mut self) -> Result<()> {
        let dummy = [
            // cmd msg: FPGA bitstream version (major.minor)    v4
            0x00, 0x00, 0x00, 0x00,  0x00, 0x08, 0x13, 0x77,
            // cmd msg: FPGA bitstream version (major)          v3
            0x00, 0x00, 0x00, 0x00,  0x01, 0x00, 0x03, 0x77,
        ];

        self.ft60.write_pipe(&dummy)?;

        let mut buf = vec![0u8; size::mb(16)];
        let bytes = self.ft60.read_pipe(&mut buf[..0x1000])?;
        if bytes >= 0x1000 {
            self.ft60.read_pipe(&mut buf)?;
        }

        Ok(())
    }

    fn try_get_version_v4(&mut self) -> Result<()> {

        /*
        WORD wbsDeviceId, wMagicPCIe;
        DWORD dwInactivityTimer = 0x000186a0;       // set inactivity timer to 1ms ( 0x0186a0 * 100MHz ) [only later activated on UDP bitstreams]
        if(!DeviceFPGA_ConfigRead(ctx, 0x0008, (PBYTE)&ctx->wFpgaVersionMajor, 1, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY) || ctx->wFpgaVersionMajor < 4) { return FALSE; }
        */
        let version_major = self.read_config::<u16>(0x0008, 1, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("version_major = {}", version_major);
        let version_minor = self.read_config::<u16>(0x0009, 1, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("version_minor = {}", version_minor);
        let fpga_id = self.read_config::<u16>(0x000a, 1, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("fpga_id = {}", fpga_id);

        // TODO: write
        //DeviceFPGA_ConfigWrite(ctx, 0x0008, (PBYTE)&dwInactivityTimer, 4, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READWRITE);

        let device_id = self.read_config::<u16>(0x0008, 2, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY);
        println!("device_id = {:?}", device_id);

        // TODO: magicPcie

        // TODO: hot reset

        /*
        // PCIe
        if(!wbsDeviceId && DeviceFPGA_ConfigRead(ctx, 0x0000, (PBYTE)&wMagicPCIe, 2, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE) && (wMagicPCIe == 0x6745)) {
            // failed getting device id - assume device is connected -> try recover the bad link with hot-reset.
            DeviceFPGA_HotResetV4(ctx);
            DeviceFPGA_ConfigRead(ctx, 0x0008, (PBYTE)&wbsDeviceId, 2, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY);
        }
        ctx->wDeviceId = _byteswap_ushort(wbsDeviceId);
        ctx->phySupported = DeviceFPGA_GetPHYv4(ctx);
        return TRUE;
        */

        Ok(())
    }

    fn read_config<T: Pod>(&mut self, addr: u16, cb: u16, flags: u16) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.read_config_into_raw(addr, obj.as_bytes_mut(), cb, flags)?;
        Ok(obj)
    }

    fn read_config_into_raw(&mut self, addr: u16, buf: &mut [u8], cb: u16, flags: u16) -> Result<()> {
        if cb == 0 || cb > size::kb(4) as u16 || addr > size::kb(4) as u16 {
            return Err(Error::Connector("invalid config address requested"));
        }

        let mut req = [0u8; size::kb(128)];
        let mut ptr = 0;
        for a in (addr..addr+cb).step_by(2) {
            req[ptr + 4] = ((a | (flags & 0x8000)) >> 8) as u8;
            req[ptr + 5] = (a & 0xff) as u8;
            req[ptr + 6] = (0x10 | (flags & 0x03)) as u8;
            req[ptr + 7] = 0x77;
            ptr += 8;
        }

        self.ft60.write_pipe(&req[..ptr])?;

        let mut readbuf = [0u8; size::kb(128)];
        let bytes = self.ft60.read_pipe(&mut readbuf)?;

        let view = readbuf.as_data_view();
        let mut skip = 0;
        'outer: for i in (0..bytes).step_by(32) {
            while view.copy::<u32>(i + skip) == 0x55556666 {
                trace!("ftdi workaround detected, skipping 4 bytes");
                skip += 4;
                if i + skip + 32 > bytes {
                    // return Err(Error::Connector("out of range config read"));
                    break 'outer;
                }
            }

            let mut status = view.copy::<u32>(i + skip);
            if status & 0xf0000000 != 0xe0000000 {
                trace!("invalid status reply, skipping");
            }

            trace!("parsing data buffer");
            for j in 0..7 {
                let status_flag = (status & 0x0f) == (flags & 0x03) as u32;
                status >>= 4; // move to next status
                if !status_flag {
                    trace!("status source flag does not match source");
                    continue;
                }

                let data = view.copy::<u32>(i + skip + 4 + j);
                let mut a = u16::swap_bytes(data as u16);
                a -= (flags & 0x8000) + addr;
                if a > cb {
                    trace!("address data out of range, skipping");
                    continue;
                }

                if a == cb -1 {
                    buf[a as usize] = ((data >> 16) & 0xff) as u8;
                } else {
                    buf[a as usize] = ((data >> 16) & 0xffff) as u8;
                }
            }
        }

        Ok(())
    }

}
