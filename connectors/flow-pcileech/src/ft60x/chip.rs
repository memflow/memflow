use dataview::Pod;

/// Chip configuration - FIFO Mode
#[allow(unused)]
pub enum FifoMode {
    Mode245 = 0,
    Mode600 = 1,
    Max = 2,
}

/// Chip configuration - Channel Configuration
#[allow(unused)]
pub enum ChannelConfig {
    Config4 = 0,
    Config2 = 1,
    Config1 = 2,
    Config1OutPipe = 3,
    Config1InPipe = 4,
    Max = 5,
}

/// Chip configuration - Optional Feature Support
#[allow(unused)]
pub enum OptionalFeatureSupport {
    DisableAll = 0,
    EnableBatteryCharging = 1,
    DisableCancelSessionUnderrun = 2,
    EnableNotificationMessageInch1 = 4,
    EnableNotificationMessageInch2 = 8,
    EnableNotificationMessageInch3 = 0x10,
    EnableNotificationMessageInch4 = 0x20,
    EnableNotificationMessageInchAll = 0x3C,
    DisableUnderrunInch1 = 0x1 << 6,
    DisableUnderrunInch2 = 0x1 << 7,
    DisableUnderrunInch3 = 0x1 << 8,
    DisableUnderrunInch4 = 0x1 << 9,
    DisableUnderrunInchAll = 0xF << 6,
}

/// Chip configuration - Config structure
#[repr(C)]
#[derive(Clone, Pod)]
pub struct Config {
    // Device Descriptor
    pub vendor_id: i16,
    pub product_id: i16,

    // String Descriptors
    pub string_descriptors: [i8; 128],

    // Configuration Descriptor
    reserved1: i8,
    pub power_attributes: i8,
    pub power_consumption: i16,

    // Data Transfer Configuration
    reserved2: i8,
    pub fifo_clock: i8,
    pub fifo_mode: i8,
    pub channel_config: i8,

    // Optional Feature Support
    pub optional_feature_support: i16,
    pub battery_charging_gpio_config: i8,
    pub flash_eeprom_detection: i8, // Read-only

    // MSIO and GPIO Configuration
    pub msio_control: u32,
    pub gpio_control: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Pod)]
pub struct ControlRequest {
    pub idx: u32,
    pub pipe: u8,
    pub cmd: u8,
    unknown1: u8,
    unknown2: u8,
    pub len: u32,
    unknown3: u32,
    unknown4: u32,
}

impl ControlRequest {
    pub fn new(idx: u32, pipe: u8, cmd: u8, len: u32) -> Self {
        Self{
idx,
pipe,
cmd,
unknown1: 0,
unknown2: 0,
len,
unknown3: 0,
unknown4: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_struct_sizes() {
        assert_eq!(size_of::<Config>(), 0x98);
        assert_eq!(size_of::<ControlRequest>(), 0x14);
    }
}
