#[doc(hidden)]
#[macro_export]
macro_rules! __create_pattern {
    ($name:ident = $pat:literal @ $align:literal) => {
        const $name: $crate::__private::Pattern<$align, { $crate::__private::pattern_len!($pat) }> =
            $crate::__private::Pattern::new($pat);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __do_match {
    (($base:expr, $size:expr) $({ $pat:literal, $align:literal, $($mod:expr)?, $func:ident, $match_addr:ident, $block:block }),*) => {{
        let match_area = core::ptr::slice_from_raw_parts($base as _, $size);
        $crate::__private::paste! {$({
            $crate::__create_pattern!([<__pattern_ $func>] = $pat @ $align);
            for rel_match_addr in [<__pattern_ $func>].matches(&*match_area) {
                let abs_match_addr = $base + rel_match_addr;
                let $match_addr = $crate::__do_match!(@mod abs_match_addr; $($mod)?);
                pl_println!(concat!("Found ", stringify!($func), " at {:#010X}"), $match_addr);
                $block
            }
        })*}
    }};

    (@mod $addr:ident; $modifier:expr) => {
        $modifier($addr as _)
    };

    (@mod $addr:ident; ) => {
        $addr
    };
}

#[macro_export]
macro_rules! pattern_match {
    ($match_fn:ident; ($base:expr, $size:expr) $({
        $pat:literal @ $align:literal $(= $mod:expr)?,
        $vis:vis $func:ident($($arg:ident: $argtype:ty),*) $(-> $rtype:ty)? $(= $variadic:tt)? $(,)?
    }),* $(,)?) => {
        $crate::__private::paste! {
            $(
                #[doc(hidden)]
                static mut [<__addr_ $func>]: Option<extern "C" fn($($argtype),* $(, $variadic)?) $(-> $rtype)?> = None;
                $vis unsafe fn $func($($arg: $argtype),*) $(-> $rtype)? {
                    pl_println!(concat!("Attempting to call ", stringify!($func)));
                    match [<__addr_ $func>] {
                        Some(func) => func($($arg),*),
                        None => Default::default(),
                    }
                }
            )*

            pub unsafe fn $match_fn() {
                $crate::__do_match!(($base, $size) $({ $pat, $align, $($mod)?, $func, match_addr, {
                    [<__addr_ $func>] = Some(core::mem::transmute(match_addr | 1));
                    break;
                } }),*)
            }
        }
    };
}


#[macro_export]
macro_rules! install_hooks {
    ($install_fn:ident; ($base:expr, $size:expr) $({
        $pat:literal @ $align:literal $(= $mod:expr)?,
        $func:ident(orig: _ $(, $($arg:ident: $argtype:ty),*)?) $(-> $rtype:ty)? $(,)?
    }),* $(,)?) => {
        $crate::__private::paste! {
            pub unsafe fn $install_fn() {
                $crate::__do_match!(($base, $size) $({ $pat, $align, $($mod)?, $func, match_addr, {
                    [<__install_ $func>](match_addr);
                } }),*)
            }
        }

        $($crate::__private::paste! {
            // Ensure the hook function has the expected signature.
            const _: unsafe fn(orig: [<orig_ $func>] $(, $($arg: $argtype),*)?) $(-> $rtype:ty)? = $func;

            // A no_std reimplementation of bhook::hook_fn!() macro.
            #[doc(hidden)]
            static mut [<__hook_ctx_ $func>]: core::option::Option<[<__hook_ctx_ty_ $func>]> = None;
            #[doc(hidden)]
            struct [<__hook_ctx_ty_ $func>] {
                original: *mut u8,
                hook: *const u8,
                backup: [u8; $crate::BACKUP_LEN]
            }

            // This is the only type exposed to the user, since they will need this in their hook.
            type [<orig_ $func>] = unsafe fn($($($arg: $argtype),*)?) $(-> $rtype)?;

            // The intermediate hook function that will be called by the original function.
            #[doc(hidden)]
            #[inline(never)]
            unsafe extern "C" fn [<__hook_ $func>]($($($arg: $argtype),*)?) $(-> $rtype:ty)? {
                lk_println!(concat!("Hook ", stringify!($func), "was called!"));
                $func([<__original_ $func>] $(, $($arg),*)?)
            }

            // The intermediate function that will call the original function.
            #[doc(hidden)]
            #[inline(never)]
            unsafe fn [<__original_ $func>]($($($arg: $argtype),*)?) $(-> $rtype:ty)? {
                #[allow(static_mut_refs)] // this is fine...
                let Some(sus) = [<__hook_ctx_ $func>].as_ref() else {
                    panic!("FATAL: missing hook context for {}", stringify!($func));
                };

                lk_println!(concat!("Calling original function of ", stringify!($func)));
                $crate::__private::disable_hook(sus.original, sus.backup);
                let orig_addr = sus.original as usize | 1; // set thumb mode bit
                let original: extern "C" fn($($($arg: $argtype),*)?) $(-> $rtype)? = core::mem::transmute(orig_addr);
                let result = original($($arg),*);
                let _ = $crate::__private::enable_hook(sus.original, sus.hook);
                result
            }

            // The function that will install the hook at the given address.
            #[doc(hidden)]
            unsafe fn [<__install_ $func>](orig_addr: usize) {
                pl_println!(concat!("Installing ", stringify!($func), " at {:#010X}"), orig_addr);
                let backup = $crate::__private::enable_hook(orig_addr as _, [<__hook_ $func>] as _);
                [<__hook_ctx_ $func>] = Some([<__hook_ctx_ty_ $func>] {
                    original: orig_addr as _,
                    hook: [<__hook_ $func>] as _,
                    backup,
                });
            }
        })*
    };
}
