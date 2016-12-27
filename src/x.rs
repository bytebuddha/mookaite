
use x11_dl::xlib;
use libc::{ c_uchar, c_int,  c_ulong };
use slog::Logger;
use std::ffi::{ CString };
use std::ptr::{ null, null_mut };
use std::mem::{zeroed, uninitialized };
use std::slice::from_raw_parts;
use std::path::PathBuf;
use std::process::Command;

pub struct XWrapper {
    logger: Logger,
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
    root_window: xlib::Window,
}

impl XWrapper {
    pub fn new(logger: Logger) -> XWrapper {
        let xlib = xlib::Xlib::open().expect("Unable to open Xlib!");
        let display = unsafe { (xlib.XOpenDisplay)(null()) };
        if display == null_mut() {
            panic!("Unable to open display");
        }
        //Get the root window.
        let rt = unsafe { (xlib.XRootWindow)(display, (xlib.XDefaultScreen)(display)) };
        let mut a = XWrapper {
            logger: logger,
            display: display,
            root_window: rt,
            xlib: xlib,
        };
        a.init();
        a
    }

    fn get_atom_by_name(&self, atom: &str) -> c_ulong {
        let d_atom_s = CString::new(atom).unwrap();
        unsafe {
            (self.xlib.XInternAtom)(self.display, d_atom_s.as_ptr(), xlib::False)
        }
    }

    fn init(&mut self) {
        unsafe {
            (self.xlib.XSelectInput)(self.display, self.root_window, xlib::PropertyChangeMask);
        }
    }

    pub fn next_event(&mut self) -> xlib::XPropertyEvent {
        trace!(self.logger, "Waiting for Xlib event!");
        loop {
            let mut event: xlib::XEvent = unsafe { zeroed() };
            unsafe { (self.xlib.XNextEvent)(self.display, &mut event) };
            match event.get_type() {
                xlib::PropertyNotify => {
                    let e: xlib::XPropertyEvent = From::from(event);
                    let got: u64 = e.atom;
                    let wanted = self.get_atom_by_name("_NET_CURRENT_DESKTOP");
                    if got == wanted {
                        trace!(self.logger, "Returning event");
                        return e;
                    }
                },
                _ => { unreachable!() }
            }
        }
    }

    pub fn get_number_of_desktops(&self) -> u8 {
        trace!(self.logger, "Counting number of desktops");
        let mut actual_type_return: c_ulong = 0;
        let mut actual_format_return: c_int = 0;
        let mut nitems_return: c_ulong = 0;
        let mut bytes_after_return: c_ulong = 0;
        let mut prop_return: *mut c_uchar = unsafe{ uninitialized() };
        unsafe {
        (self.xlib.XGetWindowProperty) (self.display, self.root_window as c_ulong,
                self.get_atom_by_name("_NET_NUMBER_OF_DESKTOPS"),
                                   0, 0xFFFFFFFF, 0, 0,
                                   &mut actual_type_return,
                                   &mut actual_format_return,
                                   &mut nitems_return,
                                   &mut bytes_after_return,
                                   &mut prop_return);
        }
        let rtn: Vec<u64> = unsafe { from_raw_parts(prop_return as *const c_ulong, nitems_return as usize)}.iter()
                          .map(|&c| c as u64)
                          .collect();
        if rtn.len() != 1 {
            crit!(self.logger, "Their was a problem reading number of desktops.");
           panic!()
        } else {
            //assert!(rtn.len() == 1);
            rtn[0] as u8
        }

    }

    pub fn change_background(&mut self, img_file: &PathBuf) {
        // Simple using feh for changing backgrounds for now.
        trace!(self.logger,"Changing background to {:?}",img_file);
        Command::new("/usr/bin/feh")
                     .arg("--bg-scale")
                     .arg(img_file)
                     .spawn()
                     .expect("Failed to run feh");
    }

    pub fn get_current_desktop(&self) -> usize {
        let mut actual_type_return: c_ulong = 0;
        let mut actual_format_return: c_int = 0;
        let mut nitems_return: c_ulong = 0;
        let mut bytes_after_return: c_ulong = 0;
        let mut prop_return: *mut c_uchar = unsafe{ uninitialized() };
        unsafe {
        (self.xlib.XGetWindowProperty) (self.display, self.root_window as c_ulong,
                self.get_atom_by_name("_NET_CURRENT_DESKTOP"),
                                   0, 0xFFFFFFFF, 0, 0,
                                   &mut actual_type_return,
                                   &mut actual_format_return,
                                   &mut nitems_return,
                                   &mut bytes_after_return,
                                   &mut prop_return);
        }
        let rtn: Vec<u64> = unsafe { from_raw_parts(prop_return as *const c_ulong, nitems_return as usize)}.iter()
                          .map(|&c| c as u64)
                          .collect();
        if rtn.len() != 1 {
           crit!(self.logger, "There was a problem reading the number of desktops");
           panic!();
        } else {
            //assert!(rtn.len() == 1);
            rtn[0] as usize
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_desktop() {
        let mut x = XWrapper::new();
        println!("{:#?}",x.get_current_desktop());
    }
    #[test]
    fn number_of_desktops() {
        let mut x = XWrapper::new();
        println!("{:#?}",x.get_number_of_desktops());
    }

    #[test]
    fn next_event() {
        let mut x = XWrapper::new();
        x.next_event();
    }
}
