#![no_std]

extern crate alloc;

use linux_kernel_module as lkm;

use lkm::{println, cstr, KernelResult, file_operations};
use lkm::file_operations::FileOperations;
use lkm::user_ptr::UserSlicePtrWriter;
use lkm::KernelModule;

struct HelloWorldModule {
    #[allow(dead_code)]
    chrdev_registration: lkm::chrdev::Registration
}

impl KernelModule for HelloWorldModule {
    fn init() -> lkm::KernelResult<Self> {
        let chrdev_registration =
            linux_kernel_module::chrdev::builder(cstr!("chrdev-tests"), 0..1)?
                .register_device::<HelloFile>()
                .build()?;
        Ok(HelloWorldModule {
            chrdev_registration,
        })
    }
}

impl Drop for HelloWorldModule {
    fn drop(&mut self) {
        println!("Goodbye kernel module!");
    }
}

struct HelloFile;

impl HelloFile {
    fn read(&self, _file: &file_operations::File, buf: &mut UserSlicePtrWriter, offset: u64) -> KernelResult<()> {
        for c in b"123456789\n"
            .iter()
            .skip(offset as _)
            .take(buf.len())
        {
            buf.write(&[*c])?;
        }
        Ok(())
    }
}

impl FileOperations for HelloFile {
    fn open() -> KernelResult<Self> {
        Ok(HelloFile)
    }

    const READ: file_operations::ReadFn<Self> = Some(HelloFile::read);
}

lkm::kernel_module!(
    HelloWorldModule,
    license: b"GPL"
);
