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
