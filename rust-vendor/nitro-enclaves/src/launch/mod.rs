// SPDX-License-Identifier: Apache-2.0

mod error;
mod linux;
mod types;

pub use error::*;
pub use types::*;

use crate::device::Device;
use linux::*;
use rand::{rngs::OsRng, TryRngCore};
use std::os::fd::{AsRawFd, RawFd};

const VMADDR_CID_PARENT: u32 = 3;

/// Facilitates the execution of the nitro enclaves launch process.
#[derive(Default)]
pub struct Launcher {
    vm_fd: RawFd,
    slot_uid: u64,
    cpu_ids: Vec<u32>,
}

impl Launcher {
    /// Begin the nitro enclaves launch process by creating a new enclave VM.
    pub fn new(dev: &Device) -> Result<Self, LaunchError> {
        let mut slot_uid: u64 = 0;
        let vm_fd = unsafe { libc::ioctl(dev.as_raw_fd(), NE_CREATE_VM as _, &mut slot_uid) };

        if vm_fd < 0 || slot_uid == 0 {
            return Err(LaunchError::ioctl_err_from_errno());
        }

        Ok(Self {
            vm_fd,
            slot_uid,
            cpu_ids: Vec::new(),
        })
    }

    /// Get the enclave's file descriptor.
    pub fn vm_fd(&self) -> RawFd {
        self.vm_fd
    }

    /// Get the enclave's slot UID.
    pub fn slot_uid(&self) -> u64 {
        self.slot_uid
    }

    /// Allocate enclave memory and populate it with the enclave image.
    pub fn set_memory(&mut self, mem: MemoryInfo) -> Result<(), LaunchError> {
        // Load the VM's enclave image type and fetch the offset in enclave memory of where to
        // start placing the enclave image.
        let mut load_info = ImageLoadInfo::from(&mem.image_type);

        // Get the image offset.
        let ret = unsafe {
            libc::ioctl(
                self.vm_fd.as_raw_fd(),
                NE_GET_IMAGE_LOAD_INFO as _,
                &mut load_info,
            )
        };

        if ret < 0 {
            return Err(LaunchError::ioctl_err_from_errno());
        }

        // Allocate the memory regions from the requested size.
        let mut regions = UserMemoryRegions::new(mem.size_mib).map_err(LaunchError::MemInit)?;

        // Populate the memory regions with the contents of the enclave image.
        regions
            .image_fill(load_info.memory_offset as usize, mem.image_type)
            .map_err(LaunchError::MemInit)?;

        // Add each memory region.
        for r in regions.inner_ref() {
            let ret = unsafe { libc::ioctl(self.vm_fd, NE_SET_USER_MEMORY_REGION, r) };
            if ret < 0 {
                panic!();
            }
        }

        Ok(())
    }

    /// Set a vCPU for an enclave. The vCPU can be auto-chosen from the NE CPU pool or it can be
    /// set by the caller.
    ///
    /// If set by the caller, the CPU needs to be available in the NE CPU pool.
    pub fn add_vcpu(&mut self, id: Option<u32>) -> Result<(), LaunchError> {
        let mut id = id.unwrap_or(0);

        let ret = unsafe { libc::ioctl(self.vm_fd, NE_ADD_VCPU as _, &mut id) };

        if ret < 0 {
            return Err(LaunchError::ioctl_err_from_errno());
        }

        self.cpu_ids.push(id);

        Ok(())
    }

    /// Start running an enclave. Supply start flags and optional enclave CID. If successful, will
    /// return the actual enclave's CID (which may be different than the supplied CID).
    pub fn start(&self, flags: StartFlags, cid: Option<u64>) -> Result<u64, LaunchError> {
        let mut cid = cid.unwrap_or(0);

        // Ensure that a valid CID is used. If the current CID is invalid, randomly-generate a
        // valid one.
        loop {
            if cid > VMADDR_CID_PARENT as u64 && cid <= u32::MAX as u64 {
                break;
            }

            cid = OsRng
                .try_next_u32()
                .map_err(|_| LaunchError::CidRandomGenerate)? as u64;
        }

        // Start the enclave VM.
        let mut start_info = StartInfo::new(flags, cid);

        let ret = unsafe { libc::ioctl(self.vm_fd, NE_START_ENCLAVE as _, &mut start_info) };

        if ret < 0 {
            return Err(LaunchError::ioctl_err_from_errno());
        }

        Ok(start_info.cid)
    }
}
