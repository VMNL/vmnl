// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Linux-only injection owned by the driver; no VMNL or GLFW calls.

use std::{
    ffi::CStr,
    fs::{self, File, OpenOptions},
    io,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt as _},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;
const FIRST_BUTTON: u16 = 0x130;
const HAT_X: u16 = 0x10;
const HAT_Y: u16 = 0x11;
const CREATE: libc::Ioctl = libc::_IO(0x55, 1);
const DESTROY: libc::Ioctl = libc::_IO(0x55, 2);

pub(crate) struct VirtualJoystick {
    file: File,
    created: bool,
}

impl VirtualJoystick {
    pub(crate) fn open() -> Result<Self, String> {
        let file = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open("/dev/uinput")
            .map_err(|e| format!("cannot open /dev/uinput: {e}; native qualification requires explicit host access; no permissions were changed"))?;
        let fixture = Self {
            file,
            created: false,
        };
        let mut version = 0_u32;
        fixture.data(libc::_IOR::<u32>(u32::from(b'U'), 45), &mut version)?;
        if version < 5 {
            return Err(format!(
                "uinput ABI version {version} does not support the required setup interface"
            ));
        }
        Ok(fixture)
    }

    pub(crate) fn create(&mut self, name: &str) -> Result<PathBuf, String> {
        if self.created || name.len() >= libc::UINPUT_MAX_NAME_SIZE || !name.is_ascii() {
            return Err("invalid virtual joystick creation request".into());
        }
        self.set_bit(100, EV_KEY)?;
        self.set_bit(100, EV_ABS)?;
        for button in FIRST_BUTTON..FIRST_BUTTON + 15 {
            self.set_bit(101, button)?;
        }
        for axis in (0..6).chain([HAT_X, HAT_Y]) {
            self.set_bit(103, axis)?;
            // SAFETY: zero is valid for all scalar fields in the Linux UAPI structure.
            let mut setup: libc::uinput_abs_setup = unsafe { std::mem::zeroed() };
            setup.code = axis;
            let range = if axis >= HAT_X { 1 } else { 1000 };
            setup.absinfo.minimum = -range;
            setup.absinfo.maximum = range;
            self.data(
                libc::_IOW::<libc::uinput_abs_setup>(u32::from(b'U'), 4),
                &mut setup,
            )?;
        }
        // SAFETY: zero is valid for every field and padding in this Linux UAPI structure.
        let mut setup: libc::uinput_setup = unsafe { std::mem::zeroed() };
        setup.id = libc::input_id {
            bustype: 3,
            vendor: 0xf00d,
            product: 0x0094,
            version: 0x0091,
        };
        for (target, byte) in setup.name.iter_mut().zip(name.bytes()) {
            *target = libc::c_char::try_from(byte).map_err(|e| e.to_string())?;
        }
        self.data(
            libc::_IOW::<libc::uinput_setup>(u32::from(b'U'), 3),
            &mut setup,
        )?;
        self.control(CREATE)?;
        self.created = true;
        let mut sysname = [0_u8; 128];
        self.data(libc::_IOR::<[u8; 128]>(u32::from(b'U'), 44), &mut sysname)?;
        let sysname = CStr::from_bytes_until_nul(&sysname)
            .map_err(|e| e.to_string())?
            .to_str()
            .map_err(|e| e.to_string())?;
        if !sysname.starts_with("input")
            || sysname.len() <= 5
            || !sysname[5..].bytes().all(|b| b.is_ascii_digit())
        {
            return Err(format!("invalid uinput sysname: {sysname:?}"));
        }
        let directory = PathBuf::from("/sys/devices/virtual/input").join(sysname);
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut reason = "event node not created".to_owned();
        loop {
            for entry in fs::read_dir(&directory).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("event") {
                    let path = PathBuf::from("/dev/input").join(name);
                    match File::open(&path) {
                        Ok(_) => return Ok(path),
                        Err(error) => {
                            reason = format!("{} is not readable: {error}", path.display());
                        }
                    }
                }
            }
            if Instant::now() >= deadline {
                return Err(reason);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub(crate) fn button(&self, index: u16, pressed: bool) -> Result<(), String> {
        if index >= 15 {
            return Err("fixture button index is out of range".into());
        }
        self.emit(EV_KEY, FIRST_BUTTON + index, i32::from(pressed))?;
        self.sync()
    }

    pub(crate) fn axes(&self, values: [i32; 6]) -> Result<(), String> {
        if !values.iter().all(|value| (-1000..=1000).contains(value)) {
            return Err("fixture axis value is out of range".into());
        }
        for (axis, value) in (0..6).zip(values) {
            self.emit(EV_ABS, axis, value)?;
        }
        self.sync()
    }

    pub(crate) fn hat(&self, x: i32, y: i32) -> Result<(), String> {
        if !(-1..=1).contains(&x) || !(-1..=1).contains(&y) {
            return Err("fixture hat coordinate is out of range".into());
        }
        self.emit(EV_ABS, HAT_X, x)?;
        self.emit(EV_ABS, HAT_Y, y)?;
        self.sync()
    }

    pub(crate) fn neutral(&self) -> Result<(), String> {
        for button in 0..15 {
            self.emit(EV_KEY, FIRST_BUTTON + button, 0)?;
        }
        for axis in (0..6).chain([HAT_X, HAT_Y]) {
            self.emit(EV_ABS, axis, 0)?;
        }
        self.sync()
    }

    pub(crate) fn destroy(&mut self) -> Result<(), String> {
        if !self.created {
            return Err("fixture is not connected".into());
        }
        self.control(DESTROY)?;
        self.created = false;
        Ok(())
    }

    fn set_bit(&self, number: u32, value: u16) -> Result<(), String> {
        // SAFETY: UI_SET_*BIT takes an integer capability, not a pointer. The live fd belongs
        // to this fixture and only Linux input codes defined above reach this call.
        let result = unsafe {
            libc::ioctl(
                self.file.as_raw_fd(),
                libc::_IOW::<libc::c_int>(u32::from(b'U'), number),
                libc::c_int::from(value),
            )
        };
        checked(result)
    }

    fn data<T>(&self, request: libc::Ioctl, data: &mut T) -> Result<(), String> {
        // SAFETY: private callers pair each ioctl with its exact libc Linux UAPI type. The
        // initialized exclusive buffer remains live throughout this synchronous kernel call.
        checked(unsafe { libc::ioctl(self.file.as_raw_fd(), request, std::ptr::from_mut(data)) })
    }

    fn control(&self, request: libc::Ioctl) -> Result<(), String> {
        // SAFETY: CREATE/DESTROY use no argument and act only on this owned uinput fd.
        checked(unsafe { libc::ioctl(self.file.as_raw_fd(), request) })
    }

    fn emit(&self, kind: u16, code: u16, value: i32) -> Result<(), String> {
        if !self.created {
            return Err("injection requires a connected fixture".into());
        }
        // SAFETY: zero initializes Linux input_event including architecture-specific timestamps.
        let mut event: libc::input_event = unsafe { std::mem::zeroed() };
        event.type_ = kind;
        event.code = code;
        event.value = value;
        let size = std::mem::size_of_val(&event);
        // SAFETY: input_event is a C-layout Linux UAPI value with initialized padding. The
        // kernel reads exactly its size synchronously; neither a Rust reference nor fd escapes.
        let count = unsafe {
            libc::write(
                self.file.as_raw_fd(),
                std::ptr::from_ref(&event).cast(),
                size,
            )
        };
        if usize::try_from(count).ok() == Some(size) {
            Ok(())
        } else {
            Err(format!(
                "uinput event write failed/short: count={count}, error={}",
                io::Error::last_os_error()
            ))
        }
    }

    fn sync(&self) -> Result<(), String> {
        self.emit(0, 0, 0)
    }
}

fn checked(result: libc::c_int) -> Result<(), String> {
    if result < 0 {
        Err(io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

impl Drop for VirtualJoystick {
    fn drop(&mut self) {
        if self.created {
            // Error paths still release every injected control before removing the owned device.
            let _ = self.neutral();
            let _ = self.destroy();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn uinput_setup_layout_matches_linux_uapi() {
        assert_eq!(std::mem::size_of::<libc::input_id>(), 8);
        assert_eq!(std::mem::size_of::<libc::uinput_setup>(), 92);
        assert_eq!(std::mem::size_of::<libc::uinput_abs_setup>(), 28);
    }
}
