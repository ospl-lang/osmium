#[derive(Debug)]
pub enum Action {
    Resolving,
    Parsing,
    Compiling,
    Invoking,
    Finished,
}

pub fn log(action: Action, message: &str) {
    eprintln!("\x1B[1;35m{:>12}\x1B[0m {}", format!("{action:?}"), message);
}

#[macro_export] macro_rules! Log {
    ($action:ident, $($y:expr),+) => {
        $crate::log::log($crate::log::Action::$action, &format!($($y),+));
    };
}
