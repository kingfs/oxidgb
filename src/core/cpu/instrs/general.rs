/**
 * general.rs
 *
 * General arithmetic.
**/

use core::cpu::CPU;

use core::cpu::instrs::utils::*;

/**
 * **0xC6** - *ADD a,#* - Add # to a.
 */
pub fn add_a_n(cpu : &mut CPU) -> u8 {
    let prev_value = cpu.regs.a;
    let value = get_n(cpu);
    let new_value = prev_value.wrapping_add(value);

    cpu.regs.a = new_value;

    cpu.regs.set_flag_z(new_value == 0);
    cpu.regs.set_flag_n(false);
    cpu.regs.set_flag_h(((value & 0x0F) + (prev_value & 0x0F)) > 0xF);
    cpu.regs.set_flag_c(((prev_value as u16) + (value as u16)) > 0xFF);

    return 8 /* Cycles */;
}

/**
 * **0xC6** - *SUB a,#* - Subtract # to a.
 */
pub fn sub_a_n(cpu : &mut CPU) -> u8 {
    let prev_value = cpu.regs.a;
    let value = get_n(cpu);
    let new_value = prev_value.wrapping_sub(value);

    cpu.regs.a = new_value;

    cpu.regs.set_flag_z(new_value == 0);
    cpu.regs.set_flag_n(true);
    cpu.regs.set_flag_h(((value as i16 & 0x0F) - (prev_value as i16 & 0x0F)) < 0);
    cpu.regs.set_flag_c(((prev_value as i16) - (value as i16)) < 0);

    return 8 /* Cycles */;
}

/// **0x19 ~ 0x39** - *ADD HL,X* - Add XX to HL.
pub fn add_hl_x(val : u16, cpu : &mut CPU) -> u8 {
    // TODO: Check accuracy of flags
    let prev_value = cpu.regs.get_hl();
    cpu.regs.set_hl(prev_value.wrapping_add(val));

    cpu.regs.set_flag_n(false);

    cpu.regs.set_flag_h(((val & 0x0FFF) + (prev_value & 0x0FFF)) > 0xFFF);
    cpu.regs.set_flag_c(((prev_value as u32) + (val as u32)) > 0xFFFF);

    return 8 /* Cycles */;
}

/// **0xCE** - *ADC a,#* - Add # + Carry flag to a.
pub fn adc_a_n(cpu : &mut CPU) -> u8 {
    let value = get_n(cpu);
    let old_value = cpu.regs.a;
    let new_value = old_value
        .wrapping_add(value)
        .wrapping_add(if cpu.regs.get_flag_c() {1} else {0});

    let unwrapped_value = (old_value as u16)
        + (value as u16)
        + if cpu.regs.get_flag_c() {1} else {0};

    cpu.regs.a = new_value;

    cpu.regs.f = 0;
    cpu.regs.set_flag_z(new_value == 0);

    cpu.regs.set_flag_h((old_value ^ new_value ^ value & 0x10) == 0x10);
    cpu.regs.set_flag_c(((old_value as u16)
            ^ unwrapped_value
            ^ (value as u16)
            & 0x100) == 0x100);

    return 8 /* Cycles */;
}