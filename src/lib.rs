#![no_std]

#[macro_use]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    borrow::{Cow, ToOwned},
    vec::Vec,
};
use core::fmt::{Formatter, Write};

use lazy_static::lazy_static;

use linux_kernel_module::{
    self as lkm,
    println,
    cstr,
    KernelResult,
    file_operations::{self, FileOperations},
    user_ptr::{UserSlicePtrWriter, UserSlicePtrReader},
    KernelModule,
};

use try_lock::TryLock;

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    surname: String,
    email: String,
    phone: String,
    age: u32,
}

impl core::fmt::Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "User#{}: {} {}, {} y.o. ({})",
            self.id, self.surname, self.name, self.age, self.phone
        ))
    }
}

// This is safe because Registration doesn't actually expose any methods.
// https://github.com/fishinabarrel/linux-kernel-module-rust/blob/b9a4c2db5c255051981deef269dd6892631771c7/src/chrdev.rs#L86
struct RegistrationWrapper(lkm::chrdev::Registration);

unsafe impl Send for RegistrationWrapper {}

struct Data {
    request: String,
    response: Cow<'static, str>,
    users: Vec<User>,
    next_id: u64,
    // Will be unregistered when dropped.
    #[allow(dead_code)]
    chrdev_registration: RegistrationWrapper,
}

impl Data {
    fn handle_request(&mut self) {
        let mut splitted = self.request.split('\n');
        let command = splitted.next();
        self.response = match command {
            Some("ADD") => {
                let args = splitted.collect::<Vec<&str>>();
                match args[..] {
                    [surname, name, email, phone, age] => {
                        match age.parse() {
                            Err(err) => Cow::Owned(format!("Invalid age: {}", err)),
                            Ok(age) => {
                                let user = User {
                                    id: self.next_id,
                                    name: name.to_owned(),
                                    surname: surname.to_string(),
                                    email: email.to_string(),
                                    phone: phone.to_string(),
                                    age
                                };
                                self.next_id += 1;
                                let response = user.to_string();
                                self.users.push(user);
                                Cow::Owned(response)
                            }
                        }
                    },
                    ref args => Cow::Owned(format!("Expected 4 arguments, but given `{:#?}`", args))
                }
            },
            Some(unknown) => Cow::Owned(format!("Unknown command: `{}`", unknown)),
            None => Cow::Borrowed("Command is missing"),
        }
    }
}

#[derive(Clone)]
struct PhoneBookModule;

lazy_static! {
    static ref MODULE: TryLock<Option<Data>> = TryLock::new(None);
}

impl KernelModule for PhoneBookModule {
    fn init() -> lkm::KernelResult<Self> {
        let chrdev_registration =
            linux_kernel_module::chrdev::builder(cstr!("chrdev-tests"), 0..1)?
                .register_device::<HelloFile>()
                .build()?;
        let data = Data {
            request: String::new(),
            response: Cow::Borrowed(""),
            users: Vec::new(),
            next_id: 0,
            chrdev_registration: RegistrationWrapper(chrdev_registration),
        };
        // This may cause a panic, but only if KernelModule is will be initialized when used.
        let mut locked = MODULE.try_lock().unwrap();
        *locked = Some(data);
        Ok(PhoneBookModule)
    }
}

impl Drop for PhoneBookModule {
    fn drop(&mut self) {
        let mut locked = MODULE.try_lock().unwrap();
        *locked = None;
    }
}

struct HelloFile;

impl HelloFile {
    fn read(&self, _file: &file_operations::File, buf: &mut UserSlicePtrWriter, offset: u64) -> KernelResult<()> {
        let data = MODULE.try_lock().ok_or(lkm::Error::EAGAIN)?;
        let data = data.as_ref().ok_or(lkm::Error::EFAULT)?;
        let response = data.response.as_bytes();
        let offset = offset as usize;
        if offset >= response.len() {
            return Ok(())
        }
        buf.write(&response[offset..])
    }

    fn read_until_zero(buf: &mut UserSlicePtrReader) -> KernelResult<(Vec<u8>, bool)> {
        let mut data = Vec::with_capacity(buf.len());
        let mut buffer = [0; 1];
        let mut was_zero = false;
        for i in 0..buf.len() {
            buf.read(&mut buffer)?;
            let x = buffer[0];
            if x != 0 {
                data.push(x);
            } else {
                was_zero = true;
                break;
            }
        }
        Ok((data, was_zero))
    }

    fn write(&self, buf: &mut UserSlicePtrReader, offset: u64) -> KernelResult<()> {
        let (data, is_complete) = Self::read_until_zero(buf)?;
        let decoded = match String::from_utf8(data) {
            Ok(x) => x,
            Err(_invalid_unicode) => {
                return Err(lkm::Error::EINVAL);
            }
        };

        let mut data = MODULE.try_lock().ok_or(lkm::Error::EAGAIN)?;
        let data = data.as_mut().ok_or(lkm::Error::EFAULT)?;
        data.request.push_str(&decoded);

        if is_complete {
            data.handle_request();
            data.request.clear();
        }

        Ok(())
    }
}

impl FileOperations for HelloFile {
    fn open() -> KernelResult<Self> {
        Ok(HelloFile)
    }

    const READ: file_operations::ReadFn<Self> = Some(HelloFile::read);
    const WRITE: file_operations::WriteFn<Self> = Some(HelloFile::write);
}

lkm::kernel_module!(
    PhoneBookModule,
    license: b"GPL"
);
