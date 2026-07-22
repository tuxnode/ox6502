pub struct Joypad {
    strobe: bool,
    index: u8,
    buttons: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            strobe: false,
            index: 0,
            buttons: 0,
        }
    }

    pub fn set_button(&mut self, bit: u8, pressed: bool) {
        if pressed {
            self.buttons |= bit;
        } else {
            self.buttons &= !bit;
        }
    }

    pub fn write(&mut self, val: u8) {
        self.strobe = val & 1 != 0;
        if self.strobe {
            self.index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.index > 7 {
            return 1;
        }
        let result = if self.buttons & (1 << self.index) != 0 {
            1
        } else {
            0
        };
        if !self.strobe {
            self.index += 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Joypad;

    #[test]
    fn reads_start_as_fourth_serial_button_bit() {
        let mut joypad = Joypad::new();
        joypad.set_button(1 << 3, true);

        joypad.write(1);
        joypad.write(0);

        assert_eq!(joypad.read(), 0);
        assert_eq!(joypad.read(), 0);
        assert_eq!(joypad.read(), 0);
        assert_eq!(joypad.read(), 1);
    }
}
