#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCH: Arch = Arch::X86_64;

#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCH: Arch = Arch::AArch64;

#[allow(dead_code)] // Only one arch will be used, so it's an expected behaviour
#[derive(Debug)]
pub enum Arch {
    X86_64,
    AArch64,
}