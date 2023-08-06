use core::fmt::Formatter;

#[repr(C)]
#[derive(Debug)]
pub struct InterruptFrame {
    pub ds: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub interrupt_number: u64,
    pub error_code: u64,
    pub ip: u64,
    pub cs: u64,
    pub flags: u64,
    pub user_sp: u64,
    pub ss: u64,
}

impl core::fmt::Display for InterruptFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ax {:016x} bx {:016x} cx {:016x} dx {:016x}\n",
            self.rax, self.rbx, self.rcx, self.rdx
        )?;
        write!(
            f,
            "sp {:016x} bp {:016x} si {:016x} di {:016x}\n",
            self.user_sp, self.rbp, self.rsi, self.rdi
        )?;
        write!(
            f,
            " 8 {:016x}  9 {:016x} 10 {:016x} 11 {:016x}\n",
            self.r8, self.r9, self.r10, self.r11
        )?;
        write!(
            f,
            "12 {:016x} 13 {:016x} 14 {:016x} 15 {:016x}\n",
            self.r12, self.r13, self.r14, self.r15
        )?;
        write!(
            f,
            "ip {:016x} cs {:016x} fl {:016x}",
            self.ip, self.cs, self.flags
        )?;
        Ok(())
    }
}
