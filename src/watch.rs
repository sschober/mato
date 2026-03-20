//! watch files for changes, abstracting over platform-specific kernel APIs

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod imp {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    use std::ptr;

    pub const EVFILT_VNODE: i16 = -4;
    pub const EV_ADD: u16 = 0x1;
    pub const EV_ENABLE: u16 = 0x4;
    pub const EV_CLEAR: u16 = 0x20;

    pub const NOTE_DELETE: u32 = 0x0000_0001;
    pub const NOTE_WRITE: u32 = 0x0000_0002;

    #[derive(Debug)]
    #[repr(C)]
    pub struct Timespec {
        tv_sec: isize,
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
        /// constructor for an event filter, which filters for write events on vnodes
        #[must_use]
        pub const fn wait_for_write_on(fd: i32) -> Self {
            #[allow(clippy::cast_sign_loss)]
            Self {
                ident: fd as u64,
                filter: EVFILT_VNODE,
                flags: EV_ADD | EV_ENABLE | EV_CLEAR,
                // some editors do not write to a file directly,
                // but create a new file and rename that to the
                // old file, which then results in a DELETE on
                // that
                fflags: NOTE_DELETE | NOTE_WRITE,
                data: 0,
                udata: 0,
            }
        }
    }

    /// encapsulates a kernel queue file descriptor.
    #[derive(Debug, Clone)]
    pub struct Kqueue {
        fd: i32,
    }

    impl Kqueue {
        #[must_use]
        pub fn create() -> Self {
            let queue_fd = unsafe { kqueue() };
            assert!(queue_fd >= 0, "{}", std::io::Error::last_os_error());
            Self { fd: queue_fd }
        }

        pub fn wait_for_write_on_file_name(&self, file_name: &str) -> std::io::Result<()> {
            let f = File::open(file_name)?;
            let fd = f.as_raw_fd();
            self.wait_for_write_on_file_descriptor(fd);
            Ok(())
        }

        pub fn wait_for_write_on_file_descriptor(&self, fd: i32) {
            let event = Kevent::wait_for_write_on(fd);
            let mut changelist = [event];
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
            assert!(res >= 0, "{}", std::io::Error::last_os_error());
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

    #[cfg(test)]
    mod tests {
        use super::{Kevent, Kqueue};

        #[test]
        fn kevent_construction() {
            let kevent = Kevent::wait_for_write_on(0);
            assert!(kevent.ident == 0);
        }

        #[test]
        fn kqueue_construction() {
            let kqueue = Kqueue::create();
            assert!(kqueue.fd >= 0);
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::ffi::CString;

    // inotify event masks
    const IN_CLOSE_WRITE: u32 = 0x0000_0008;
    const IN_DELETE_SELF: u32 = 0x0000_0400;
    const IN_MOVE_SELF: u32 = 0x0000_0800;
    // sent by the kernel when a watch descriptor is removed; must be filtered out
    // to avoid spurious wake-ups on the next wait call
    const IN_IGNORED: u32 = 0x0000_8000;

    extern "C" {
        fn inotify_init() -> i32;
        fn inotify_add_watch(fd: i32, pathname: *const i8, mask: u32) -> i32;
        fn inotify_rm_watch(fd: i32, wd: i32) -> i32;
        fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
        fn close(fd: i32) -> i32;
    }

    pub struct Kqueue {
        fd: i32,
    }

    impl Kqueue {
        #[must_use]
        pub fn create() -> Self {
            let fd = unsafe { inotify_init() };
            assert!(fd >= 0, "{}", std::io::Error::last_os_error());
            Self { fd }
        }

        pub fn wait_for_write_on_file_name(&self, file_name: &str) -> std::io::Result<()> {
            let path = CString::new(file_name).expect("file name contains null byte");
            // buffer must fit at least one inotify_event (16 bytes header + name)
            let mut buf = [0u8; 256];
            loop {
                // Add (or re-add after an atomic replace) a watch on the current
                // inode.  If the file is temporarily absent mid-rename, spin until
                // it reappears.
                let wd = loop {
                    let wd = unsafe {
                        inotify_add_watch(
                            self.fd,
                            path.as_ptr(),
                            IN_CLOSE_WRITE | IN_DELETE_SELF | IN_MOVE_SELF,
                        )
                    };
                    if wd >= 0 {
                        break wd;
                    }
                    // File doesn't exist yet (editor is mid-rename); retry shortly.
                    std::thread::sleep(std::time::Duration::from_millis(10));
                };

                let res = unsafe { read(self.fd, buf.as_mut_ptr(), buf.len()) };
                assert!(res >= 0, "{}", std::io::Error::last_os_error());
                // inotify_event layout: wd(i32) mask(u32) cookie(u32) len(u32) name[]
                // mask sits at byte offset 4
                let mask = u32::from_ne_bytes([buf[4], buf[5], buf[6], buf[7]]);

                if mask & IN_IGNORED != 0 {
                    // Stale event from a previous inotify_rm_watch; discard and
                    // loop to re-add the watch.
                    continue;
                }

                if mask & (IN_DELETE_SELF | IN_MOVE_SELF) != 0 {
                    // The editor replaced the file atomically (e.g. neovim renames
                    // a temp file over the original).  Remove the now-stale watch
                    // and loop: we'll re-add a watch on the new inode and wait for
                    // its IN_CLOSE_WRITE before triggering a render.
                    unsafe { inotify_rm_watch(self.fd, wd) };
                    continue;
                }

                // IN_CLOSE_WRITE on the current inode: the file is fully written.
                unsafe { inotify_rm_watch(self.fd, wd) };
                break;
            }
            Ok(())
        }
    }

    impl Drop for Kqueue {
        fn drop(&mut self) {
            unsafe { close(self.fd) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Kqueue;

        #[test]
        fn kqueue_construction() {
            let kqueue = Kqueue::create();
            assert!(kqueue.fd >= 0);
        }
    }
}

pub use imp::Kqueue;
