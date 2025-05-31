/// Catching and handling ARM CPU exceptions durign the test-case execution

use libafl_qemu::qemu::Qemu;
use libafl_qemu::*;
use libafl::prelude::*;
use libafl_bolts::Named;

use log;
use serde::{Deserialize, Serialize};
use core::fmt::{Debug};
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub enum ExceptionType {
    RESET   = 0,
    UNDEF   = 1,
    SVC     = 2,
    PREAB   = 3,
    DATAB   = 4,
    HYP     = 5,
    IRQ     = 6,
    FIQ     = 7,
    UNKNOWN = 8,
}

impl From<u32> for ExceptionType {
    fn from(orig: u32) -> Self {
        match orig {
            0 => return ExceptionType::RESET,
            1 => return ExceptionType::UNDEF,
            2 => return ExceptionType::SVC,
            3 => return ExceptionType::PREAB,
            4 => return ExceptionType::DATAB,
            5 => return ExceptionType::HYP,
            6 => return ExceptionType::IRQ,
            7 => return ExceptionType::FIQ,
            _ => return ExceptionType::UNKNOWN,
        };
    }
}

pub struct ExceptionHandler {
    exception_vector_base   : GuestAddr,
    #[allow(dead_code)]
    exception_addr_reset    : GuestAddr,
    exception_addr_undef    : GuestAddr,
    exception_addr_svc      : GuestAddr,
    exception_addr_preab    : GuestAddr,
    exception_addr_datab    : GuestAddr,
    exception_addr_hyp      : GuestAddr,
    exception_addr_irq      : GuestAddr,
    exception_addr_fiq      : GuestAddr,
}

static mut EXCEPTION_VECTOR_BASE: GuestAddr = 0;

impl ExceptionHandler {
    pub fn new(exception_vector_base: GuestAddr) -> Self {
        Self {
            exception_vector_base   : exception_vector_base,
            exception_addr_reset    : exception_vector_base + 4*(ExceptionType::RESET as u32),
            exception_addr_undef    : exception_vector_base + 4*(ExceptionType::UNDEF as u32),
            exception_addr_svc      : exception_vector_base + 4*(ExceptionType::SVC as u32),
            exception_addr_preab    : exception_vector_base + 4*(ExceptionType::PREAB as u32),
            exception_addr_datab    : exception_vector_base + 4*(ExceptionType::DATAB as u32),
            exception_addr_hyp      : exception_vector_base + 4*(ExceptionType::HYP as u32),
            exception_addr_irq      : exception_vector_base + 4*(ExceptionType::IRQ as u32),
            exception_addr_fiq      : exception_vector_base + 4*(ExceptionType::FIQ as u32),
        }
    }

    pub fn start(&self, qemu: &Qemu) {
        unsafe { EXCEPTION_VECTOR_BASE = self.exception_vector_base};
        // qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_reset, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_undef, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_svc, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_preab, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_datab, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_hyp, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_irq, exception_hook, false);
        qemu.hooks().add_instruction_hooks(qemu as *const _ as u64, self.exception_addr_fiq, exception_hook, false);
    }

    pub fn stop(&self, qemu: &Qemu) {
        // let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_reset, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_undef, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_svc, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_preab, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_datab, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_hyp, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_irq, true);
        let _ = qemu.hooks().remove_instruction_hooks_at(self.exception_addr_fiq, true);
    }
}

extern "C" fn exception_hook(data: u64, pc: GuestAddr) {
    log::debug!("Exception hook: pc={:#x}", pc);

    match ((pc - unsafe { EXCEPTION_VECTOR_BASE }) / 4).into() {
        ExceptionType::RESET    => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::RESET as u8 },
        ExceptionType::UNDEF    => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::UNDEF as u8 },
        ExceptionType::SVC      => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::SVC as u8 },
        ExceptionType::PREAB    => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::PREAB as u8 },
        //ExceptionType::DATAB    => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::DATAB as u8 },
        ExceptionType::DATAB    => log::info!("Data abort triggered"),
        ExceptionType::HYP      => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::HYP as u8 },
        ExceptionType::IRQ      => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::IRQ as u8 },
        ExceptionType::FIQ      => unsafe{ HOOK_TRIGGERED |= 1 << ExceptionType::FIQ as u8 },
        _                       => log::error!("Unknown exception triggered"),
    }
    match ((pc - unsafe { EXCEPTION_VECTOR_BASE }) / 4).into() {
        ExceptionType::RESET    => log::debug!("Exception: RESET"),
        ExceptionType::UNDEF    => log::debug!("Exception: UNDEF"),
        ExceptionType::SVC      => log::debug!("Exception: SVC"),
        ExceptionType::PREAB    => log::debug!("Exception: PREAB"),
        ExceptionType::DATAB    => log::debug!("Exception: DATAB"),
        ExceptionType::HYP      => log::debug!("Exception: HYP"),
        ExceptionType::IRQ      => log::debug!("Exception: IRQ"),
        ExceptionType::FIQ      => log::debug!("Exception: FIQ"),
        _                       => log::error!("Unknown exception triggered"),
    }

    let qemu = unsafe { (data as *const Qemu).as_ref().unwrap() };
    qemu.current_cpu().unwrap().trigger_breakpoint();
}

static mut HOOK_TRIGGERED: usize = 0;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExceptionFeedback {}

impl<S> StateInitializer<S> for ExceptionFeedback {}


impl<EM, I, OT, S> Feedback<EM, I, OT, S> for ExceptionFeedback
where
    S: HasClientPerfMonitor,
    EM: EventFirer<I, S>,
    OT: ObserversTuple<I, S>,
{
    #[allow(clippy::wrong_self_convention)]
    fn is_interesting(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &I,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    {
        unsafe{
            if HOOK_TRIGGERED != 0 {
                log::info!("ExceptionFeedback=True");
                HOOK_TRIGGERED = 0;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

impl Named for ExceptionFeedback {
    #[inline]
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("ExceptionFeedback");
        &NAME
    }
}

impl ExceptionFeedback {
    /// Creates a new [`ExceptionFeedback`]
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ExceptionFeedback {
    fn default() -> Self {
        Self::new()
    }
}

