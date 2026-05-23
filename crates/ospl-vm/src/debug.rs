//! Quick-and-dirty VM debugging module

#[cfg(debug_assertions)]
mod dbg_ctx {
    use std::cell::RefCell;

    thread_local! {
        static STACK: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    #[allow(unused)]  // intentionally unused
    pub struct DbgMark(String);

    impl DbgMark {
        #[inline(always)]
        #[allow(unused)]
        #[track_caller]
        pub fn new(msg: String) -> Self {
            let caller_location = std::panic::Location::caller();
            let (col, ln, fp) = (caller_location.column(), caller_location.line(), caller_location.file());
            eprintln!("TRACE >>> \t[{fp}:{ln}:{col}]\t {msg}");
            STACK.with(|s| s.borrow_mut().push(msg.clone()));
            Self(msg)
        }
    }

    impl Drop for DbgMark {
        #[inline(always)]
        fn drop(&mut self) {
            let Some(_) = STACK.with(|s| s.borrow_mut().pop())
            else { return; };

            // eprintln!("TRACE <<< {s}");
        }
    }

    pub fn dump_take() -> Vec<String> {
        STACK.with(|s| s.take())
    }
}

#[cfg(not(debug_assertions))]
mod dbg_ctx {
    #[allow(unused)]
    pub struct DbgMark(());

    impl DbgMark {
        #[inline(always)]
        #[allow(unused)]
        pub fn new(_: String) -> Self {
            Self(())
        }
    }

    pub fn dump_take() -> Vec<&'static str> {
        Vec::new()
    }
}

#[allow(unused)]
pub use dbg_ctx::{DbgMark, dump_take};

#[allow(unused)]
pub fn setup_debug_panic_handler() {
    std::panic::set_hook(Box::new(|info| {
        #[cfg(debug_assertions)]
        {
            eprintln!("VM context: {:#?}", dump_take());
        }

        eprintln!("panic: {info}");
        std::process::exit(1);
    }));
}