// SPDX-License-Identifier: Apache-2.0

use std::{fmt, io};

const NE_ERR_VCPU_ALREADY_USED: i32 = 256;
const NE_ERR_VCPU_NOT_IN_CPU_POOL: i32 = 257;
const NE_ERR_VCPU_INVALID_CPU_CORE: i32 = 258;
const NE_ERR_INVALID_MEM_REGION_SIZE: i32 = 259;
const NE_ERR_INVALID_MEM_REGION_ADDR: i32 = 260;
const NE_ERR_UNALIGNED_MEM_REGION_ADDR: i32 = 261;
const NE_ERR_MEM_REGION_ALREADY_USED: i32 = 262;
const NE_ERR_MEM_NOT_HUGE_PAGE: i32 = 263;
const NE_ERR_MEM_DIFFERENT_NUMA_NODE: i32 = 264;
const NE_ERR_MEM_MAX_REGIONS: i32 = 265;
const NE_ERR_NO_MEM_REGIONS_ADDED: i32 = 266;
const NE_ERR_NO_VCPUS_ADDED: i32 = 267;
const NE_ERR_ENCLAVE_MEM_MIN_SIZE: i32 = 268;
const NE_ERR_FULL_CORES_NOT_USED: i32 = 269;
const NE_ERR_NOT_IN_INIT_STATE: i32 = 270;
const NE_ERR_INVALID_VCPU: i32 = 271;
const NE_ERR_NO_CPUS_AVAIL_IN_POOL: i32 = 272;
const NE_ERR_INVALID_PAGE_SIZE: i32 = 273;
const NE_ERR_INVALID_FLAG_VALUE: i32 = 274;
const NE_ERR_INVALID_ENCLAVE_CID: i32 = 275;

/// Error that may occur during the launch process.
#[derive(Debug)]
pub enum LaunchError {
    /// /dev/nitro_enclaves ioctl error.
    Ioctl(IoctlError),

    /// Memory initialization error.
    MemInit(MemInitError),

    /// Error occuring when randomly-generating an enclave CID.
    CidRandomGenerate,
}

impl LaunchError {
    /// Error on ioctl, return an IoctlError from errno.
    pub fn ioctl_err_from_errno() -> Self {
        Self::Ioctl(IoctlError::from_errno())
    }
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::Ioctl(e) => format!("ioctl error: {e}"),
            Self::MemInit(e) => format!("memory initialization error: {e}"),
            Self::CidRandomGenerate => "unable to randomly-generate enclave CID".to_string(),
        };

        write!(f, "{}", msg)
    }
}

/// Error that may occur when issuing /dev/nitro_enclaves ioctls.
#[derive(Debug)]
pub enum IoctlError {
    /// copy_to_user() failure.
    CopyToUser,

    /// Memory allocation failure for internal bookkeeping variables.
    InternalMemAllocation,

    /// Current task mm is not the same as the one that created the enclave.
    DifferentTaskMm,

    /// The value of the provided flag is invalid.
    InvalidFlagValue,

    /// No nitro enclave CPU pool set or no CPUs available in the pool.
    NoCpusAvailInPool,

    /// The enclave is not in "init" (not yet started) state.
    NotInInitState,

    /// The provided vCPU is already used.
    VcpuAlreadyUsed,

    /// The provided vCPU is not available in the NE CPU pool.
    VcpuNotInCpuPool,

    /// The core id of the provided vCPU is invalid or out of range.
    VcpuInvalidCpuCore,

    /// The provided vCPU is not in the available CPUs range.
    InvalidVcpu,

    /// Invalid physical memory region(s) (e.g. unaligned addresses).
    InvalidPhysicalMemRegion,

    /// The memory size of the region is not a multiple of 2 MiB.
    InvalidMemRegionSize,

    /// Invalid user space address given.
    InvalidMemRegionAddr,

    /// Unaligned user space address given.
    UnalignedMemRegionAddr,

    /// The memory region is already used.
    MemRegionAlreadyUsed,

    /// The memory region is not backed by huge pages.
    MemNotHugePage,

    /// The memory region is not from the same NUMA node as the CPUs.
    MemDifferentNumaNode,

    /// The number of memory regions set for the enclave reached maximum.
    MemMaxRegions,

    /// The memory region is not backed by pages multiple of 2 MiB.
    InvalidPageSize,

    /// No memory regions are set.
    NoMemRegionsAdded,

    /// No vCPUs are set.
    NoVcpusAdded,

    /// Full core(s) not set for the enclave.
    FullCoresNotUsed,

    /// Enclave memory is less than minimum memory size (64 MiB).
    EnclaveMemMinSize,

    /// The provided enclave CID is invalid.
    InvalidEnclaveCid,

    /// Unknown.
    Unknown(std::io::Error),
}

impl IoctlError {
    /// Parse an error from errno.
    pub fn from_errno() -> Self {
        Self::from(std::io::Error::last_os_error())
    }
}

impl From<std::io::Error> for IoctlError {
    fn from(err: std::io::Error) -> Self {
        match err.raw_os_error() {
            Some(mut e) => {
                if e < 0 {
                    e = -e;
                }

                match e {
                    libc::EFAULT => Self::CopyToUser,
                    libc::ENOMEM => Self::InternalMemAllocation,
                    libc::EIO => Self::DifferentTaskMm,
                    libc::EINVAL => Self::InvalidPhysicalMemRegion,
                    NE_ERR_VCPU_ALREADY_USED => Self::VcpuAlreadyUsed,
                    NE_ERR_VCPU_NOT_IN_CPU_POOL => Self::VcpuNotInCpuPool,
                    NE_ERR_VCPU_INVALID_CPU_CORE => Self::VcpuInvalidCpuCore,
                    NE_ERR_INVALID_MEM_REGION_SIZE => Self::InvalidMemRegionSize,
                    NE_ERR_INVALID_MEM_REGION_ADDR => Self::InvalidMemRegionAddr,
                    NE_ERR_UNALIGNED_MEM_REGION_ADDR => Self::UnalignedMemRegionAddr,
                    NE_ERR_MEM_REGION_ALREADY_USED => Self::MemRegionAlreadyUsed,
                    NE_ERR_MEM_NOT_HUGE_PAGE => Self::MemNotHugePage,
                    NE_ERR_MEM_DIFFERENT_NUMA_NODE => Self::MemDifferentNumaNode,
                    NE_ERR_MEM_MAX_REGIONS => Self::MemMaxRegions,
                    NE_ERR_NO_MEM_REGIONS_ADDED => Self::NoMemRegionsAdded,
                    NE_ERR_NO_VCPUS_ADDED => Self::NoVcpusAdded,
                    NE_ERR_ENCLAVE_MEM_MIN_SIZE => Self::EnclaveMemMinSize,
                    NE_ERR_FULL_CORES_NOT_USED => Self::FullCoresNotUsed,
                    NE_ERR_NOT_IN_INIT_STATE => Self::NotInInitState,
                    NE_ERR_INVALID_VCPU => Self::InvalidVcpu,
                    NE_ERR_NO_CPUS_AVAIL_IN_POOL => Self::NoCpusAvailInPool,
                    NE_ERR_INVALID_PAGE_SIZE => Self::InvalidPageSize,
                    NE_ERR_INVALID_FLAG_VALUE => Self::InvalidFlagValue,
                    NE_ERR_INVALID_ENCLAVE_CID => Self::InvalidEnclaveCid,
                    _ => Self::Unknown(err),
                }
            }
            None => Self::Unknown(err),
        }
    }
}

impl fmt::Display for IoctlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::CopyToUser => "unable to copy data to/from userspace".to_string(),
            Self::InternalMemAllocation => {
                "memory allocation failure for internal bookkeeping variables".to_string()
            }
            Self::DifferentTaskMm => {
                "current task mm is not the same as the one that created the enclave".to_string()
            }
            Self::InvalidFlagValue => "the value of the provided flag is invalid".to_string(),
            Self::NoCpusAvailInPool => {
                "no nitro enclave CPU pool set or no CPUs available in the pool".to_string()
            }
            Self::NotInInitState => "not in init (not yet started) state".to_string(),
            Self::VcpuAlreadyUsed => "the provided vCPU is already used".to_string(),
            Self::VcpuNotInCpuPool => {
                "the provided vCPU is not available in the NE CPU pool".to_string()
            }
            Self::VcpuInvalidCpuCore => {
                "the core id of the provided vCPU is invalid or out of range".to_string()
            }
            Self::InvalidVcpu => "the provided vCPU is not in the available CPUs range".to_string(),
            Self::InvalidPhysicalMemRegion => {
                "invalid physical memory region(s) (e.g. unaligned addresses)".to_string()
            }
            Self::InvalidMemRegionSize => {
                "the memory size of the region is not a multiple of 2 MiB".to_string()
            }
            Self::InvalidMemRegionAddr => "invalid user space address given".to_string(),
            Self::UnalignedMemRegionAddr => "unaligned user space address given".to_string(),
            Self::MemRegionAlreadyUsed => "the memory region is already used".to_string(),
            Self::MemNotHugePage => "the memory region is not backed by huge pages".to_string(),
            Self::MemDifferentNumaNode => {
                "the memory region is not from the same NUMA node as the CPUs".to_string()
            }
            Self::MemMaxRegions => {
                "the number of memory regions set for the enclave reached maximum".to_string()
            }
            Self::InvalidPageSize => {
                "the memory region is not backed by pages multiple of 2 MiB".to_string()
            }
            Self::NoMemRegionsAdded => "no memory regions are set".to_string(),
            Self::NoVcpusAdded => "no vCPUs are set".to_string(),
            Self::FullCoresNotUsed => "full core(s) not set for the enclave".to_string(),
            Self::EnclaveMemMinSize => {
                "enclave memory is less than minimum memory size (64 MiB)".to_string()
            }
            Self::InvalidEnclaveCid => "the provided enclave CID is invalid".to_string(),
            Self::Unknown(e) => format!("unknown error: {e}"),
        };

        write!(f, "{}", msg)
    }
}

/// Error that may occur when allocating and configuring enclave memory.
#[derive(Debug)]
pub enum MemInitError {
    /// A valid combination of hugepages could not be found for the requested size.
    NoHugePageFound,

    /// Unable to retrieve image metadata.
    ImageMetadata(io::Error),

    /// Unable to rewind image to beginning of image file.
    ImageRewind(io::Error),

    /// Unable to write total image file to memory regions.
    ImageWriteIncomplete,

    /// Unable to read bytes from image file.
    ImageRead(io::Error),

    /// Overflow when checking if memory region write was greater than image offset.
    OffsetCheckOverflow,

    /// Overflow when calculating end of image region in guest memory.
    ImagePlacementOverflow,
}

impl fmt::Display for MemInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::NoHugePageFound => {
                "a valid combination of hugepages could not be found for the requested size"
                    .to_string()
            }
            Self::ImageMetadata(e) => format!("unable to retrieve image metadata: {e}"),
            Self::ImageRewind(e) => {
                format!("unable to rewind image to beginning of image file: {e}")
            }
            Self::ImageWriteIncomplete => {
                "unable to write total image file to memory regions".to_string()
            }
            Self::ImageRead(e) => format!("unable to read bytes from image file: {e}"),
            Self::OffsetCheckOverflow => {
                "overflow when checking if memory region write was greater than image offset"
                    .to_string()
            }
            Self::ImagePlacementOverflow => {
                "overflow when calculating end of image region in guest memory".to_string()
            }
        };

        write!(f, "{}", msg)
    }
}
