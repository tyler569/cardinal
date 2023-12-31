use crate::x86::pio;

const PRIMARY_COMMAND: u16 = 0x20;
const PRIMARY_DATA: u16 = 0x21;
const SECONDARY_COMMAND: u16 = 0xA0;
const SECONDARY_DATA: u16 = 0xA1;

pub unsafe fn remap_and_disable() {
    pio::write_u8(PRIMARY_COMMAND, 0x11); // reset and program
    pio::write_u8(PRIMARY_DATA, 0x20); // starting at interrupt 0x20
    pio::write_u8(PRIMARY_DATA, 0x04); // secondary at line 2
    pio::write_u8(PRIMARY_DATA, 0x01); // 8086 mode
    pio::write_u8(PRIMARY_DATA, 0xFF); // mask all interrupts

    pio::write_u8(SECONDARY_COMMAND, 0x11); // reset and program
    pio::write_u8(SECONDARY_DATA, 0x28); // starting at interrupt 0x20
    pio::write_u8(SECONDARY_DATA, 0x02); // (not 100% sure)
    pio::write_u8(SECONDARY_DATA, 0x01); // 8086 mode
    pio::write_u8(SECONDARY_DATA, 0xFF); // mask all interrupts
}
