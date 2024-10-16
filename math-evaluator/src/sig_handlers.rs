/// Implements a signal handler for SIGFPE that catches the Zero-Division error and sets the result of the division to 0.
/// The handler is implemented in such a way that it "skips" the faulting instruction and zeroes the result of the division.
///
/// # Arguments
///
/// * `_sig` - The signal number.
/// * `info` - The signal information.
/// * `_ucontext` - The context of the signal.
fn sigfpe_handler(_sig: i32, info: *mut libc::siginfo_t, _ucontext: *mut libc::c_void) {
    let info = unsafe { *info };
    let code = info.si_code;
    let addr = unsafe { info.si_addr() };
    eprintln!(
        "Caught Zero-Division error at address {:p}, code {}",
        addr, code
    );

    // Update the context to "skip" the faulting instruction
    // This is actually, where the bug is introduced. Usually, the `div` instruction
    // is 2 bytes long, but if its operand is the register of 64-bit size, the instruction is 3 bytes long (+REX prefix).
    let ucontext = unsafe { &mut *(_ucontext as *mut libc::ucontext_t) };
    ucontext.uc_mcontext.gregs[libc::REG_RIP as usize] += 3;

    // Set the result of the division to 0
    ucontext.uc_mcontext.gregs[libc::REG_RAX as usize] = 0;
    ucontext.uc_mcontext.gregs[libc::REG_RDX as usize] = 0;
}

/// Sets up the SIGFPE signal handler.
#[ctor::ctor]
fn setup_sigfpe_handler() {
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_flags = libc::SA_SIGINFO | libc::SA_NODEFER;
        sa.sa_sigaction = sigfpe_handler as usize;
        libc::sigaction(libc::SIGFPE, &sa, std::ptr::null_mut());
    }
}
