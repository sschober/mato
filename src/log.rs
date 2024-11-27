static mut LEVEL: u8 = 0;

pub fn set_log_level(l: u8) -> u8 {
    unsafe {
        LEVEL = l;
        return LEVEL;
    }
}

pub fn get_log_level() -> u8 {
    unsafe { LEVEL }
}

#[macro_export]
macro_rules! mato_err {
    ($( $args:expr ), *) => {
        eprintln!( $( $args ),* );
    };
}

#[macro_export]
macro_rules! die {
    ($( $args:expr ), *) => {
        mato::mato_err!( $( $args ),* );
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! mato_inf {
    ($( $args:expr ), *) => {
       if mato::log::get_log_level() >= 1 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! mato_dbg {
    ($( $args:expr ), *) => {
       if mato::log::get_log_level() >= 2 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! m_dbg {
    ($( $args:expr ), *) => {
       if crate::log::get_log_level() >= 2 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! mato_trc {
    ($( $args:expr ), *) => {
       if mato::log::get_log_level() >= 3 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! m_trc {
    ($( $args:expr ), *) => {
       if crate::log::get_log_level() >= 3 {
           eprintln!( $( $args ),* );
       }
    };
}
