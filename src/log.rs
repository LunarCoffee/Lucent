use std::fmt::Display;
use std::process;

use crate::util;

pub fn fatal(msg: impl Display) {
    eprintln!("[ CRT ] [ {} ] {}", get_time_now_formatted(), msg);
    process::exit(1);
}

pub fn warn(msg: impl Display) {
    eprintln!("[ WRN ] [ {} ] {}", get_time_now_formatted(), msg);
}

pub fn info(msg: impl Display) {
    println!("[ INF ] [ {} ] {}", get_time_now_formatted(), msg);
}

fn get_time_now_formatted() -> impl Display {
    util::get_time_local().format("%d/%m/%Y %r")
}
