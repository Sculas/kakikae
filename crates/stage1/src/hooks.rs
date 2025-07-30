pub unsafe fn install_hook_at(target: *mut u32, hook_fn_addr: usize) {
    target.write_unaligned(0x47304e00); // ldr r6, [pc, #0]; bx r6
    target.offset(1).write_unaligned(hook_fn_addr as _);
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_pattern {
    ($name:ident = $pat:literal @ $align:literal) => {
        const $name: patterns::Pattern<$align, { macros::pattern_len!($pat) }> = patterns::Pattern::new($pat);
    };
}

#[macro_export]
macro_rules! pattern_match {
    ($match_fn:ident; ($base:expr, $size:expr) $({
        $pat:literal @ $align:literal $(= $mod:expr)?,
        $vis:vis $func:ident($($arg:ident: $argtype:ty),*) $(-> $rtype:ty)? $(,)?
    }),* $(,)?) => {
        pastey::paste! {
            $(
                #[doc(hidden)]
                static mut [<__addr_ $func>]: *const u8 = 0x0 as _;
                $vis unsafe fn $func($($arg: $argtype),*) $(-> $rtype)? {
                    let func: extern "C" fn($($argtype),*) $(-> $rtype)? = core::mem::transmute([<__addr_ $func>]);
                    func($($arg),*)
                }
            )*

            pub unsafe fn $match_fn() {
                $crate::eprintln!("Running pattern match for: {}", stringify!($match_fn));
                $crate::eprintln!("Base: 0x{:08X}, Size: 0x{:08X}", $base, $size);
                let match_area = core::ptr::slice_from_raw_parts($base as _, $base + $size);
                $crate::eprintln!("Match area is valid...");
                $({
                    $crate::eprintln!("Pattern: {} @ {}", $pat, $align);
                    // Create the pattern from the string.
                    $crate::__create_pattern!([<__pattern_ $func>] = $pat @ $align);
                    // Execute the pattern match and store the address of the first match.
                    for match_addr in [<__pattern_ $func>].matches(&*match_area) {
                        $crate::eprintln!("Found match for {} at 0x{:08X}", stringify!($func), match_addr);
                        [<__addr_ $func>] = $crate::pattern_match!(@mod match_addr; $($mod)?) as _;
                        break;
                    }
                    $crate::eprintln!("Finished pattern match for {}", stringify!($func));
                })*
            }
        }
    };

    (@mod $addr:ident; $modifier:expr) => {
        $modifier($addr as _)
    };

    (@mod $addr:ident; ) => {
        $addr
    };
}

#[macro_export]
macro_rules! install_hooks {
    ($install_fn:ident; ($base:expr, $size:expr) $({
        $pat:literal @ $align:literal,
        $hook_fn:ident(orig: _ $(, $($arg:ident: $argtype:ty),*)?) $(-> $rtype:ty)? $(,)?
    }),* $(,)?) => {
        pastey::paste! {
            pub unsafe fn $install_fn() {
                $crate::eprintln!("Running pattern match for: {}", stringify!($match_fn));
                $crate::eprintln!("Base: 0x{:08X}, Size: 0x{:08X}", $base, $size);
                let match_area = core::ptr::slice_from_raw_parts($base as _, $base + $size);
                $({
                    // Create the pattern from the string.
                    $crate::__create_pattern!([<__pattern_ $hook_fn>] = $pat @ $align);
                    // Install the hook at the pattern matches.
                    for match_addr in [<__pattern_ $hook_fn>].matches(&*match_area) {
                        $crate::eprintln!("Found match for {} at 0x{:08X}", stringify!($hook_fn), match_addr);
                        unsafe { [<__install_ $hook_fn>](match_addr as _) };
                    }
                    $crate::eprintln!("Finished pattern match for {}", stringify!($hook_fn));
                })*
            }
        }

        $(pastey::paste! {
            // Ensure the hook function has the expected signature.
            const _: unsafe fn(orig: [<orig_ $hook_fn>] $(, $($arg: $argtype),*)?) $(-> $rtype:ty)? = $hook_fn;

            // A no_std reimplementation of bhook::hook_fn!() macro.
            #[doc(hidden)]
            static mut [<__hook_ctx_ $hook_fn>]: core::option::Option<[<__hook_ctx_ty_ $hook_fn>]> = None;
            #[doc(hidden)]
            struct [<__hook_ctx_ty_ $hook_fn>] {
                original: *mut u8,
                hook: *const u8,
                backup: [u8; bhook::BACKUP_LEN]
            }

            // This is the only type exposed to the user, since they will need this in their hook.
            type [<orig_ $hook_fn>] = unsafe fn($($($arg: $argtype),*)?) $(-> $rtype)?;

            // The intermediate hook function that will be called by the original function.
            #[doc(hidden)]
            #[inline(never)]
            unsafe extern "C" fn [<__hook_ $hook_fn>]($($($arg: $argtype),*)?) $(-> $rtype:ty)? {
                $crate::eprintln!("calling hooked function: {}", stringify!($hook_fn));
                $hook_fn([<__original_ $hook_fn>] $(, $($arg),*)?)
            }

            // The intermediate function that will call the original function.
            #[doc(hidden)]
            #[inline(never)]
            unsafe fn [<__original_ $hook_fn>]($($($arg: $argtype),*)?) $(-> $rtype:ty)? {
                #[allow(static_mut_refs)] // this is fine...
                let Some(sus) = [<__hook_ctx_ $hook_fn>].as_ref() else {
                    $crate::eprintln!("FATAL: missing hook context for {}", stringify!($hook_fn));
                    core::hint::unreachable_unchecked();
                };

                $crate::eprintln!("calling original function: {}", stringify!($hook_fn));
                $crate::hooks::__disable_hook(sus.original, sus.backup);
                let original: extern "C" fn($($($arg: $argtype),*)?) $(-> $rtype)? = core::mem::transmute(sus.original);
                let result = original($($arg),*);
                let _ = $crate::hooks::__enable_hook(sus.original, sus.hook);
                result
            }

            // The function that will install the hook at the given address.
            #[doc(hidden)]
            #[inline(never)]
            unsafe fn [<__install_ $hook_fn>](orig_addr: *mut u8) {
                $crate::eprintln!("installing hook: {}", stringify!($hook_fn));
                let backup = $crate::hooks::__enable_hook(orig_addr, [<__hook_ $hook_fn>] as _);
                [<__hook_ctx_ $hook_fn>] = Some([<__hook_ctx_ty_ $hook_fn>] {
                    original: orig_addr,
                    hook: [<__hook_ $hook_fn>] as _,
                    backup,
                });
            }
        })*
    };
}

#[doc(hidden)]
pub unsafe fn __enable_hook(orig_fn: *mut u8, hook_fn: *const u8) -> [u8; bhook::BACKUP_LEN] {
    let orig_fn = orig_fn.offset(-1);
    let backup = orig_fn.cast::<[u8; bhook::BACKUP_LEN]>().read_unaligned();
    bhook::raw_hook(orig_fn, hook_fn as usize);
    clear_cache_and_flush(orig_fn, orig_fn.add(bhook::BACKUP_LEN));
    backup
}

#[doc(hidden)]
pub unsafe fn __disable_hook(orig_fn: *mut u8, orig_code: [u8; bhook::BACKUP_LEN]) {
    let backup = orig_fn.offset(-1).cast::<[u8; bhook::BACKUP_LEN]>();
    backup.write_unaligned(orig_code);

    let orig_fn = orig_fn.cast_const();
    clear_cache_and_flush(orig_fn, orig_fn.add(bhook::BACKUP_LEN));
}

unsafe fn clear_cache_and_flush<T>(start: *const T, end: *const T) {
    let start_addr = start as usize;
    let end_addr = end as usize;

    crate::eprintln!(
        "clear_cache_and_flush: 0x{:08X} -> 0x{:08X}",
        start_addr,
        end_addr
    );

    // Clean and invalidate data cache by MVA (Modified Virtual Address)
    let mut addr = start_addr & !31; // Align to cache line (32 bytes for Cortex-A9)
    while addr < end_addr {
        // DCCIMVAC - Clean and Invalidate data cache line by MVA to PoC
        core::arch::asm!(
            "mcr p15, 0, {addr}, c7, c14, 1",
            addr = in(reg) addr,
            options(nomem, nostack)
        );
        addr += 32; // Cortex-A9 cache line size
    }

    // Data Synchronization Barrier - ensure cache operations complete
    core::arch::asm!("dsb", options(nomem, nostack));

    // Invalidate instruction cache
    core::arch::asm!(
        "mcr p15, 0, {}, c7, c5, 0", // Invalidate entire I-cache
        in(reg) 0,
        options(nomem, nostack)
    );

    // Instruction Synchronization Barrier - flush pipeline
    core::arch::asm!("isb", options(nomem, nostack));
}

pub fn follow_bl_insn(match_addr: u32) -> u32 {
    let instruction = unsafe { core::ptr::read_unaligned(match_addr as *const u32) };
    let first_halfword = (instruction >> 16) & 0xFFFF;
    let second_halfword = instruction & 0xFFFF;

    let s = (first_halfword >> 10) & 1;
    let imm10 = first_halfword & 0x3FF;
    let j1 = (second_halfword >> 13) & 1;
    let j2 = (second_halfword >> 11) & 1;
    let imm11 = second_halfword & 0x7FF;

    let i1 = if j1 ^ s != 0 { 0 } else { 1 };
    let i2 = if j2 ^ s != 0 { 0 } else { 1 };

    let mut imm32 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
    if s != 0 {
        imm32 |= 0xFF000000;
    }

    (match_addr + 4).wrapping_add(imm32)
}
