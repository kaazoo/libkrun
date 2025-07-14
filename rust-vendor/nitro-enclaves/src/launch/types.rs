// SPDX-License-Identifier: Apache-2.0

use super::error::*;

use bitflags::bitflags;
use std::fs::File;

/// The image type of the enclave.
#[derive(Debug)]
pub enum ImageType<'a> {
    /// Enclave Image Format.
    Eif(&'a mut File),
}

/// Data related to setting enclave memory.
#[derive(Debug)]
pub struct MemoryInfo<'a> {
    /// Enclave image type.
    pub image_type: ImageType<'a>,

    /// Amount of memory (in MiB) to allocate to the enclave.
    pub size_mib: usize,
}

impl<'a> MemoryInfo<'a> {
    pub fn new(image_type: ImageType<'a>, size_mib: usize) -> Self {
        Self {
            image_type,
            size_mib,
        }
    }
}

bitflags! {
    /// Configuration flags for starting an enclave.
    #[repr(transparent)]
    #[derive(Copy, Clone, Default)]
    pub struct StartFlags: u64 {
        /// Start enclave in debug mode.
        const DEBUG = 1;
    }
}

/// Calculate an enclave's poll timeout from its image size and the amount of memory allocated to
/// it.
pub struct PollTimeout(pub i32);

impl TryFrom<(&File, usize)> for PollTimeout {
    type Error = LaunchError;

    fn try_from(args: (&File, usize)) -> Result<Self, Self::Error> {
        let mul = 60 * 1000; // One minute in milliseconds.
        let size = {
            let metadata = args
                .0
                .metadata()
                .map_err(MemInitError::ImageMetadata)
                .map_err(LaunchError::MemInit)?;

            metadata.len()
        };

        let file: i32 = ((1 + (size - 1) / (6 << 30)) as i32).saturating_mul(mul);
        let alloc: i32 = ((1 + (args.1 - 1) / (100 << 30)) as i32).saturating_mul(mul);

        Ok(Self(file + alloc))
    }
}

impl From<PollTimeout> for i32 {
    fn from(arg: PollTimeout) -> i32 {
        arg.0
    }
}
