use crate::ft60x::*;

use std::mem::MaybeUninit;
use std::time::Duration;

use log::{info, trace, warn};

use flow_core::{
    error::{Error, Result},
    size,
};

use bitfield::bitfield;
use dataview::Pod;

pub const FPGA_CONFIG_CORE: u16 = 0x0003;
pub const FPGA_CONFIG_PCIE: u16 = 0x0001;
pub const FPGA_CONFIG_SPACE_READONLY: u16 = 0x0000;
pub const FPGA_CONFIG_SPACE_READWRITE: u16 = 0x8000;

// TODO: remove unused
#[allow(unused)]
pub struct PhyConfig {
    magic: u8,           // 8 bit
    tp_cfg: u8,          // 4 bit
    tp: u8,              // 4 bit
    pub wr: PhyConfigWr, // 16 bits
    pub rd: PhyConfigRd, // 32 bits
}

bitfield! {
    pub struct PhyConfigWr(u16);
    impl Debug;
    pl_directed_link_auton, _: 0;
    pl_directed_link_change, _: 2, 1;
    pl_directed_link_speed, _: 3;
    pl_directed_link_width, _: 5, 4;
    pl_upstream_prefer_deemph, _: 6;
    pl_transmit_hot_rst, _: 7;
    pl_downstream_deemph_source, _: 8;
    //_, _: 16, 9;
}

bitfield! {
    pub struct PhyConfigRd(u32);
    impl Debug;
    pl_ltssm_state, _: 5, 0;
    pl_rx_pm_state, _: 7, 6;
    pl_tx_pm_state, _: 10, 8;
    pl_initial_link_width, _: 13, 11;
    pl_lane_reversal_mode, _: 15, 14;
    pl_sel_lnk_width, _: 17, 16;
    pl_phy_lnk_up, _: 18;
    pl_link_gen2_cap, _: 19;
    pl_link_partner_gen2_supported, _: 20;
    pl_link_upcfg_cap, _: 21;
    pl_sel_lnk_rate, _: 22;
    pl_directed_change_done, _: 23;
    pl_received_hot_rst, _: 24;
    //_, _: 31, 25;
}

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
        self.read_version_clear_pipe()?;

        self.read_version_v4()?;

        Ok(())
    }

    fn read_version_clear_pipe(&mut self) -> Result<()> {
        let dummy = [
            // cmd msg: FPGA bitstream version (major.minor)    v4
            0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x13, 0x77,
            // cmd msg: FPGA bitstream version (major)          v3
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03, 0x77,
        ];

        self.ft60.write_pipe(&dummy)?;

        let mut buf = vec![0u8; size::mb(16)];
        let bytes = self.ft60.read_pipe(&mut buf[..0x1000])?;
        if bytes >= 0x1000 {
            self.ft60.read_pipe(&mut buf)?;
        }

        Ok(())
    }

    fn read_version_v4(&mut self) -> Result<()> {
        let version_major =
            self.read_config::<u8>(0x0008, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("version_major = {}", version_major);
        let version_minor =
            self.read_config::<u8>(0x0009, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("version_minor = {}", version_minor);
        let fpga_id =
            self.read_config::<u8>(0x000a, FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY)?;
        println!("fpga_id = {}", fpga_id);

        // this will cause the hardware to reset briefly
        /*
        let inactivity_timer = 0x000186a0u32; // set inactivity timer to 1ms (0x0186a0 * 100MHz) [only later activated on UDP bitstreams]
        self.write_config(
            0x0008,
            inactivity_timer,
            FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READWRITE,
        )?;
        */

        let mut device_id = self
            .read_config::<u16>(0x0008, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY)
            .unwrap_or_default();
        if device_id == 0 {
            let magic_pcie = self
                .read_config::<u16>(0x0000, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE)
                .unwrap_or_default();
            println!("magic_pcie = {:?}", magic_pcie);
            if magic_pcie == 0x6745 {
                println!("failed to get device_id - trying to recover via hot reset");
                self.hot_reset_v4().ok();
                device_id = self
                    .read_config::<u16>(0x0008, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY)
                    .unwrap_or_default();
            }
        }
        println!("device_id = {:?}", device_id);

        let (wr, rd) = self.get_phy_v4()?;
        println!("wr: {:?}", wr);
        println!("rd: {:?}", rd);

        /*
        ctx->wDeviceId = _byteswap_ushort(wbsDeviceId);
        ctx->phySupported = DeviceFPGA_GetPHYv4(ctx);
        */

        Ok(())
    }

    fn hot_reset_v4(&mut self) -> Result<()> {
        trace!("hot resetting the fpga");
        let (wr, _) = self.get_phy_v4()?;
        self.write_config(0x0016, wr.0, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE)?;
        std::thread::sleep(Duration::from_millis(250)); // TODO: poll pl_ltssm_state + timeout with failure
        self.write_config(0x0016, wr.0, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE)?;
        Ok(())
    }

    fn get_phy_v4(&mut self) -> Result<(PhyConfigWr, PhyConfigRd)> {
        let wr_raw =
            self.read_config::<u16>(0x0016, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE)?;
        let rd_raw =
            self.read_config::<u32>(0x000a, FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY)?;
        Ok((PhyConfigWr { 0: wr_raw }, PhyConfigRd { 0: rd_raw }))
    }

    #[allow(clippy::uninit_assumed_init)]
    fn read_config<T: Pod>(&mut self, addr: u16, flags: u16) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.read_config_into_raw(addr, obj.as_bytes_mut(), flags)?;
        Ok(obj)
    }

    fn read_config_build_request(addr: u16, bytes: u16, flags: u16) -> Vec<u8> {
        let mut res = Vec::new();
        for a in (addr..addr + bytes).step_by(2) {
            let mut req = [0u8; 8];
            req[4] = ((a | (flags & 0x8000)) >> 8) as u8;
            req[5] = (a & 0xff) as u8;
            req[6] = (0x10 | (flags & 0x03)) as u8;
            req[7] = 0x77;
            res.extend_from_slice(&req);
        }
        res
    }

    fn read_config_parse_response(
        addr: u16,
        respbuf: &[u8],
        outbuf: &mut [u8],
        flags: u16,
    ) -> Result<()> {
        let view = respbuf.as_data_view();
        let mut skip = 0;
        for i in (0..respbuf.len()).step_by(32) {
            if i + skip >= respbuf.len() {
                break;
            }

            while view.copy::<u32>(i + skip) == 0x55556666 {
                //trace!("ftdi workaround detected, skipping 4 bytes");
                skip += 4;
                if i + skip + 32 > respbuf.len() {
                    return Err(Error::Connector("out of range config read"));
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
                    //trace!("status source flag does not match source");
                    continue;
                }

                let data = view.copy::<u32>(i + skip + 4 + j * 4);
                let mut a = (data as u16).to_be(); // only enforce a byteswap if we are on le
                a -= (flags & 0x8000) + addr;
                if a >= outbuf.len() as u16 {
                    trace!("address data out of range, skipping");
                    continue;
                }

                if a == outbuf.len() as u16 - 1 {
                    outbuf[a as usize] = ((data >> 16) & 0xff) as u8;
                } else {
                    let b = (((data >> 16) & 0xffff) as u16).to_le_bytes();
                    outbuf[a as usize] = b[0];
                    outbuf[a as usize + 1] = b[1];
                }
            }
        }
        Ok(())
    }

    fn read_config_into_raw(&mut self, addr: u16, buf: &mut [u8], flags: u16) -> Result<()> {
        if buf.is_empty() || buf.len() > size::kb(4) || addr > size::kb(4) as u16 {
            return Err(Error::Connector("invalid config address requested"));
        }

        let req = Self::read_config_build_request(addr, buf.len() as u16, flags);

        self.ft60.write_pipe(&req)?;

        let mut readbuf = [0u8; size::kb(128)];
        let bytes = self.ft60.read_pipe(&mut readbuf)?;

        Self::read_config_parse_response(addr, &readbuf[..bytes], buf, flags)
    }

    fn write_config<T: Pod>(&mut self, addr: u16, obj: T, flags: u16) -> Result<()> {
        self.write_config_raw(addr, obj.as_bytes(), flags)
    }

    fn write_config_raw(&mut self, addr: u16, buf: &[u8], flags: u16) -> Result<()> {
        if buf.is_empty() || buf.len() > 0x200 || addr > size::kb(4) as u16 {
            return Err(Error::Connector("invalid config address to write"));
        }

        let mut outbuf = [0u8; 0x800];
        let mut ptr = 0;
        for i in (0..buf.len()).step_by(2) {
            let a = (addr + i as u16) | (flags & 0x8000);
            outbuf[ptr] = buf[i as usize]; // byte_value_addr
            outbuf[ptr + 1] = if buf.len() == i + 1 {
                0
            } else {
                buf[i as usize + 1]
            }; // byte_value_addr + 1
            outbuf[ptr + 2] = 0xFF; // byte_mask_addr
            outbuf[ptr + 3] = if buf.len() == i + 1 { 0 } else { 0xFF }; // byte_mask_addr + 1
            outbuf[ptr + 4] = (a >> 8) as u8; // addr_high = bit[6:0], write_regbank = bit[7]
            outbuf[ptr + 5] = (a & 0xFF) as u8; // addr_low
            outbuf[ptr + 6] = (0x20 | (flags & 0x03)) as u8; // target = bit[0:1], read = bit[4], write = bit[5]
            outbuf[ptr + 7] = 0x77; // MAGIC 0x77
            ptr += 8;
        }

        self.ft60.write_pipe(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_struct_sizes() {
        assert_eq!(size_of::<PhyConfigWr>(), 2);
        assert_eq!(size_of::<PhyConfigRd>(), 4);
    }

    #[test]
    fn test_config_read_build_request() {
        assert_eq!(
            Device::read_config_build_request(
                0x0008,
                1,
                FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY
            ),
            [0x0, 0x0, 0x0, 0x0, 0x0, 0x8, 0x13, 0x77]
        );
        assert_eq!(
            Device::read_config_build_request(
                0x0009,
                1,
                FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY
            ),
            [0x0, 0x0, 0x0, 0x0, 0x0, 0x9, 0x13, 0x77]
        );

        assert_eq!(
            Device::read_config_build_request(
                0x0008,
                2,
                FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY
            ),
            [0x0, 0x0, 0x0, 0x0, 0x0, 0x8, 0x11, 0x77]
        );
        assert_eq!(
            Device::read_config_build_request(
                0x0000,
                2,
                FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE
            ),
            [0x0, 0x0, 0x0, 0x0, 0x80, 0x0, 0x11, 0x77]
        );

        assert_eq!(
            Device::read_config_build_request(
                0x0016,
                2,
                FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE
            ),
            [0x0, 0x0, 0x0, 0x0, 0x80, 0x16, 0x11, 0x77]
        );
        assert_eq!(
            Device::read_config_build_request(
                0x000a,
                4,
                FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY
            ),
            [0x0, 0x0, 0x0, 0x0, 0x0, 0xA, 0x11, 0x77, 0x0, 0x0, 0x0, 0x0, 0x0, 0xC, 0x11, 0x77]
        );
    }

    #[test]
    fn test_config_parse_version_major() {
        let mut version_major = 0u8;
        Device::read_config_parse_response(
            0x0008,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 243, 255, 255, 239, 0, 8, 4, 2, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            version_major.as_bytes_mut(),
            FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY,
        )
        .unwrap();
        assert_eq!(version_major, 4);
    }

    #[test]
    fn test_config_parse_version_minor() {
        let mut version_minor = 0u8;
        Device::read_config_parse_response(
            0x0009,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 243, 255, 255, 239, 0, 9, 2, 1, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            version_minor.as_bytes_mut(),
            FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY,
        )
        .unwrap();
        assert_eq!(version_minor, 2);
    }

    #[test]
    fn test_config_parse_fpga_id() {
        let mut fpga_id = 0u8;
        Device::read_config_parse_response(
            0x000a,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 243, 255, 255, 239, 0, 10, 1, 0, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            fpga_id.as_bytes_mut(),
            FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY,
        )
        .unwrap();
        assert_eq!(fpga_id, 1);
    }

    #[test]
    fn test_config_parse_device_id() {
        let mut fpga_id = 0u8;
        Device::read_config_parse_response(
            0x000a,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 243, 255, 255, 239, 0, 10, 1, 0, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            fpga_id.as_bytes_mut(),
            FPGA_CONFIG_CORE | FPGA_CONFIG_SPACE_READONLY,
        )
        .unwrap();
        assert_eq!(fpga_id, 1);
    }

    #[test]
    fn test_config_parse_phy_wr() {
        let mut wr = 0u16;
        Device::read_config_parse_response(
            0x0016,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 241, 255, 255, 239, 128, 22, 72, 0, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            wr.as_bytes_mut(),
            FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READWRITE,
        )
        .unwrap();
        assert_eq!(wr, 0x48);
    }

    #[test]
    fn test_config_parse_phy_rd() {
        let mut rd = 0u32;
        Device::read_config_parse_response(
            0x000a,
            &[
                102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102, 85, 85, 102, 102,
                85, 85, 17, 255, 255, 239, 0, 10, 25, 8, 0, 12, 28, 0, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            rd.as_bytes_mut(),
            FPGA_CONFIG_PCIE | FPGA_CONFIG_SPACE_READONLY,
        )
        .unwrap();
        assert_eq!(rd, 0x1C0819);
    }
}
