//! watch file descriptors, wake up on changes and hide BSD kernel queueing complexities
use std::ptr;

pub const EVFILT_VNODE: i16 = -4;
pub const EV_ADD: u16 = 0x1;
pub const EV_ENABLE: u16 = 0x4;
pub const EV_CLEAR: u16 = 0x20;

pub const NOTE_WRITE: u32 = 0x00000002;

#[derive(Debug)]
#[repr(C)]
pub struct Timespec {
    /// Seconds
    tv_sec: isize,
    /// Nanoseconds
    v_nsec: usize,
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Kevent {
    pub ident: u64,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: i64,
    pub udata: u64,
}

impl Kevent {
    /// contrsuctor for an event filter, which filters for write events on vnodes
    pub fn wait_for_write_on(fd: i32) -> Kevent {
        Kevent {
            ident: fd as u64,
            filter: EVFILT_VNODE,
            flags: EV_ADD | EV_ENABLE | EV_CLEAR,
            fflags: NOTE_WRITE,
            data: 0,
            udata: 0,
        }
    }
}
/// encapsulates a kernel queue file descriptor.
/// surrogate object to attach methods onto it
#[derive(Debug, Clone)]
pub struct Kqueue {
    fd: i32,
}

impl Kqueue {
    /// creates a new kernel queue, using lib c 
    pub fn create() -> Kqueue {
        let queue_fd = unsafe { kqueue() };
        if queue_fd < 0 {
            panic!("{}", std::io::Error::last_os_error());
        }
        Kqueue { fd: queue_fd }
    }
    /// creates an event filter specifying we are interessted in writes on a file
    /// and call kevent to wait for one of those events, i.e., this call blocks
    /// until some process writes to the file below the given file descriptor
    pub fn wait_for_write_on(&self, fd: i32) {
        let event = Kevent::wait_for_write_on(fd);
        let mut changelist = [event];
        println!("watching:\t\t{}", fd);
        let res = unsafe {
            kevent(
                self.fd,
                changelist.as_ptr(),
                1,
                changelist.as_mut_ptr(),
                1,
                ptr::null(),
            )
        };
        if res < 0 {
            panic!("{}", std::io::Error::last_os_error());
        }
    }
}

#[link(name = "c")]
extern "C" {
    pub fn kqueue() -> i32;
    pub fn kevent(
        kq: i32,
        changelist: *const Kevent,
        nchanges: i32,
        eventlist: *mut Kevent,
        nevents: i32,
        timeout: *const Timespec,
    ) -> i32;
}
