#[derive(Debug)]
pub enum Action {
    Invoking,
    Creating,

    Compiling,
    Linking,

    Warning,
    Failed,

    Skipping,
}

pub fn log(action: Action, message: &str) {
    // let s = std::env::current_dir().unwrap();
    // let s = s.to_str().unwrap();
    // let s = &s[s.len()-10..];
    let s = unsafe{&*&raw const STATE};
    eprintln!(
        "\x1B[0;31m{s:<10}\x1B[0m \x1B[1;35m{:>12}\x1B[0m {}",
        format!("{action:?}"),
        message
    );
}

#[macro_export] macro_rules! Log {
    ($action:ident, $($y:expr),+) => {
        $crate::log::log($crate::log::Action::$action, &format!($($y),+))
    };
}

static mut STATE: String = String::new();

#[macro_export] macro_rules! LogState {
    ($x:expr) => {
        $crate::log::setstate($x)
    };
}

pub fn setstate(s: &str) {
    unsafe {
        STATE = s.to_string();
    }
}
