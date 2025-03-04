mod generator;
mod arithmetic_instructions;
mod architecture;
mod stack_handler;
mod subroutines;
mod os;

pub use generator::Generator;

pub use architecture::TARGET_ARCH;
pub use os::TARGET_OS;