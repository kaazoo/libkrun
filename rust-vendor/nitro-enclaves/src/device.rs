// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{File, OpenOptions},
    os::fd::{AsRawFd, RawFd},
};

/// A handle to the /dev/nitro_enclaves device.
pub struct Device(File);

impl Device {
    /// Open the device and create a handle.
    pub fn open() -> std::io::Result<Self> {
        Ok(Self(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/nitro_enclaves")?,
        ))
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
