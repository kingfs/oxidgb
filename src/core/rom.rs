/**
 * rom.rs
 *
 * Loads and parses .gb cartridges, and provides a interface for mappers.
**/

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::string::String;

/// The different kinds of cartridges that can be handled. Each has a
///  specific way of managing memory/providing additional capabilities.
#[derive(PartialEq, Debug)]
#[allow(dead_code)] // For debug messages
pub enum CartridgeType {
    RomOnly           = 0x00,
    RomMbc1           = 0x01,
    RomMbc1Ram        = 0x02,
    RomMbc1RamBatt    = 0x03,
    RomMbc2           = 0x05,
    RomMbc2Batt       = 0x06,
    RomRam            = 0x08,
    RomRamBatt        = 0x09,
    RomMMMD1          = 0x0B,
    RomMMMD1Sram      = 0x0C,
    RomMMMD1SramBatt  = 0x0D,
    RomMbc3TimerBatt  = 0x0F,
    RomMbc3TimerRamBatt = 0x10,
    RomMbc3           = 0x11,
    RomMbc3Ram        = 0x12,
    RomMbc3RamBatt    = 0x13,
    RomMbc5           = 0x19,
    RomMbc5Ram        = 0x1A,
    RomMbc5RamBatt    = 0x1B,
    RomMbc5Rumble     = 0x1C,
    RomMbc5RumbleSram = 0x1D,
    RomMbc5RumbleSramBatt = 0x1E,
    PocketCamera      = 0x1F,
    BandaiTAMA5       = 0xFD,
    HudsonHuC3        = 0xFE,
    HudsonHuC1        = 0xFF
}

/// Holds a game's ROM, and exposes interfaces to read information from
///  it intelligently.
pub struct GameROM {
    backing_data : Vec<u8>,
    current_bank : u8,

    cart_ram : Vec<u8>,
    ram_size : usize,

    pub name : String,
    pub cart_type : CartridgeType
}

impl GameROM {
    pub fn read(&self, ptr : u16) -> u8 {
        return match self.cart_type {
            CartridgeType::RomOnly => {
                self.backing_data[ptr as usize]
            }
            CartridgeType::RomMbc1 |
                CartridgeType::RomMbc1Ram |
                CartridgeType::RomMbc1RamBatt |
                CartridgeType::RomMbc3RamBatt  => {
                if ptr < 0x4000 {
                    self.backing_data[ptr as usize]
                } else {
                    let target = ptr as usize + (self.current_bank as usize - 1)
                                            * 0x4000;
                    if target >= self.backing_data.len() {
                        println!("Out of range read for MBC1!");
                        0xFF
                    } else {
                        self.backing_data[target]
                    }
                }
            }
            _ => {
                panic!("Unimplemented cart type: {:?}", self.cart_type);
            }
        };
    }

    pub fn read_ram(&self, ptr : u16) -> u8 {
        if self.ram_size == 0 {
            println!("WARN: Reading from RAM on a ROM-only cartridge!");
            return 0xFF;
        }

        return self.cart_ram[ptr as usize];
    }

    pub fn write(&mut self, ptr : u16, val : u8) {
        match self.cart_type {
            CartridgeType::RomOnly => {
                println!("WARN: Writing to ROM: {:04x} = {:02x}", ptr, val);
            }
            CartridgeType::RomMbc1 |
            CartridgeType::RomMbc1Ram |
            CartridgeType::RomMbc1RamBatt |
            CartridgeType::RomMbc3RamBatt => {
                match ptr {
                    0x0000 ... 0x1FFF => { // ROM bank activation/deactivation
                        println!("STUB: ROM bank activation: {}", val > 0);
                    }
                    0x2000 ... 0x3FFF => { // Bank switching
                        self.current_bank = val & 0b11111;
                        if self.current_bank < 1 {
                            self.current_bank = 1;
                        }
                    }
                    0x6000 ... 0x7FFF => { // Memory models
                        println!("WARN: MBC1 memory models are not supported!");
                    }
                    _ => {
                        println!("Attempted to write to ROM+MBC1 cartridge @ {:04x} = {:02x}",
                                 ptr, val);
                    }

                }
            }
            _ => {
                panic!("Unimplemented cart type: {:?}", self.cart_type);
            }
        }
    }

    pub fn write_ram(&mut self, ptr : u16, val : u8) {
        if self.ram_size == 0 {
            println!("WARN: Writing to RAM on a ROM-only cartridge!");
            return;
        }

        self.cart_ram[ptr as usize] = val;
    }

    /// Builds a new ROM from the specified file. Expects
    ///  a correctly formatted file.
    ///
    /// * `path` - The path to load from. Must be readable.
    pub fn build(path : &Path) -> GameROM {
        let file_size = match fs::metadata(path) {
            Err(why) => panic!("couldn't read metadata of {}: {}", path.display(),
                                why.description()),
            Ok(meta) => meta.len()
        };

        let mut data = Vec::with_capacity(file_size as usize);

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(),
                               why.description()),
            Ok(file) => file,
        };

        let read = file.read_to_end(&mut data).unwrap();

        println!("Read: {}, expected: {}", read, file_size);

        let name = String::from_utf8(data[0x134 .. 0x142].to_vec()).unwrap();
        let cart_type = unsafe { ::std::mem::transmute(data[0x0147]) };
        let ram_size = get_ram_size(data[0x149]);

        let ram = vec![0xFF; ram_size];

        println!("Allocated {} bytes of cart RAM", ram.len());

        return GameROM {
            backing_data : data,
            name : name,
            cart_type : cart_type,
            current_bank : 1,

            cart_ram : ram,
            ram_size : ram_size
        };
    }
}

/// Returns a RAM size for a particular RAM id.
fn get_ram_size(id : u8) -> usize {
    return match id {
        0 => 0,          // ROM only
        1 => 2   * 1024, // 2  Kbyte
        2 => 8   * 1024, // 8  Kbyte
        3 => 32  * 1024, // 32 Kbyte
        4 => 128 * 1024,  // 128 Kbyte,
        _ => panic!("Unknown RAM size: {}", id)
    }
}