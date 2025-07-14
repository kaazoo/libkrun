// SPDX-License-Identifier: Apache-2.0

use nitro_enclaves::{
    launch::{ImageType, Launcher, MemoryInfo, PollTimeout, StartFlags},
    Device,
};
use nix::{
    poll::{poll, PollFd, PollFlags},
    sys::{
        socket::{connect, socket, AddressFamily, SockFlag, SockType, VsockAddr as NixVsockAddr},
        time::{TimeVal, TimeValLike},
    },
    unistd::read,
};
use std::{
    fs::File,
    io::{Read, Write},
    os::fd::{AsRawFd, RawFd},
};
use vsock::{VsockAddr, VsockListener};

const ENCLAVE_VM_SIZE_MIB: usize = 128;

const ENCLAVE_READY_VSOCK_PORT: u32 = 9000;
const CID_TO_CONSOLE_PORT_OFFSET: u32 = 10000;

const VMADDR_CID_PARENT: u32 = 3;
const VMADDR_CID_HYPERVISOR: u32 = 0;

const SO_VM_SOCKETS_CONNECT_TIMEOUT: i32 = 6;

const HEART_BEAT: u8 = 0xb7;

// Create and start a nitro enclave using the library API.
#[test]
fn launch() {
    // Open /dev/nitro_enclaves.
    let device = Device::open().unwrap();

    // Create a new VM from the device.
    let mut launcher = Launcher::new(&device).unwrap();

    // Open the test EIF file.
    let mut eif = File::open("tests/test_data/hello.eif").unwrap();

    // Set enclave memory with provided EIF file and 128 MiB of memory.
    let mem = MemoryInfo::new(ImageType::Eif(&mut eif), ENCLAVE_VM_SIZE_MIB);
    launcher.set_memory(mem).unwrap();

    // Add one vCPU to the enclave.
    launcher.add_vcpu(None).unwrap();

    // Create a vsock listener to verify enclave kernel started.
    let sockaddr = VsockAddr::new(VMADDR_CID_PARENT, ENCLAVE_READY_VSOCK_PORT);
    let listener = VsockListener::bind(&sockaddr).unwrap();

    // Start the enclave (in debug mode) and get its CID.
    let cid: u32 = launcher
        .start(StartFlags::DEBUG, None)
        .unwrap()
        .try_into()
        .unwrap();

    // Given the enclave image and amount of memory (in bytes), calculate the poll timeout for the
    // vsock listener.
    let poll_timeout = PollTimeout::try_from((&eif, ENCLAVE_VM_SIZE_MIB << 20)).unwrap();

    // Verify the enclave kernel has booted (setting the vsock timeout to the value calculated in
    // poll_timeout).
    enclave_check(listener, poll_timeout.into(), cid);

    // The enclave was started in debug mode. Listen for debug output on a vsock for the enclave.
    listen(VMADDR_CID_HYPERVISOR, cid + CID_TO_CONSOLE_PORT_OFFSET);
}

pub fn enclave_check(listener: VsockListener, poll_timeout_ms: libc::c_int, cid: u32) {
    let mut poll_fds = [PollFd::new(listener.as_raw_fd(), PollFlags::POLLIN)];
    let result = poll(&mut poll_fds, poll_timeout_ms);
    if result == Ok(0) {
        panic!("no pollfds have selected events");
    } else if result != Ok(1) {
        panic!("more than one pollfd has selected events");
    }

    let mut stream = listener.accept().unwrap();

    // Wait until the other end is closed
    let mut buf = [0u8];
    let bytes = stream.0.read(&mut buf).unwrap();

    if bytes != 1 || buf[0] != HEART_BEAT {
        panic!("enclave check produced wrong output");
    }

    stream.0.write_all(&buf).unwrap();

    if stream.1.cid() != cid {
        panic!("CID mismatch");
    }
}

fn listen(cid: u32, port: u32) {
    // Create a vsock to listen for enclave debug output.
    let socket_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .unwrap();

    let sockaddr = NixVsockAddr::new(cid, port);

    // Set the vsock connection timeout.
    vsock_timeout(socket_fd);

    // Try to connect to the enclave.
    connect(socket_fd, &sockaddr).unwrap();

    // The testing EIF image prints Linux boot logs as debug output. One such message contains:
    //
    // [    0.000000] Booting Linux on physical CPU _
    //
    // Verify a substring of this message was found in the debug output from the enclave.
    let mut boot_msg_found = false;

    let mut buf = [0u8; 512];
    loop {
        // Read debug output from vsock.
        let ret = read(socket_fd, &mut buf);
        let Ok(sz) = ret else {
            break;
        };
        if sz != 0 {
            let msg = String::from_utf8(buf[..sz].to_vec()).unwrap();
            // Check if the Linux boot message is found in any of the output.
            if msg.contains("Booting Linux") {
                boot_msg_found = true;
            }
            print!("{}", msg);
        } else {
            break;
        }
    }

    // Ensure the boot message was found.
    if !boot_msg_found {
        panic!("Linux boot message not found from vsock output");
    }
}

fn vsock_timeout(socket_fd: RawFd) {
    // Set the timeout to 20 seconds.
    let timeval = TimeVal::milliseconds(20000);

    let ret = unsafe {
        libc::setsockopt(
            socket_fd,
            libc::AF_VSOCK,
            SO_VM_SOCKETS_CONNECT_TIMEOUT,
            &timeval as *const _ as *const libc::c_void,
            size_of::<TimeVal>() as u32,
        )
    };

    if ret != 0 {
        panic!("error setting vsock timeout");
    }
}
