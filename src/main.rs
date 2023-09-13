#![no_main]
#![no_std]

use core::arch::asm;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use log::{error, info};
use uefi::prelude::*;
use uefi::proto::{
    device_path::DevicePath,
    loaded_image::LoadedImage,
    media::{
        file::{File, FileAttribute, FileInfo, FileMode, FileType},
        fs::SimpleFileSystem,
    },
};
use uefi::table::boot::{AllocateType, MemoryType, OpenProtocolAttributes, OpenProtocolParams};
use uefi::CStr16;
use xmas_elf::ElfFile;

const SETUP_START: u16 = 0x01f1;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    info!("Initialization successful.");

    let loaded_image = match unsafe {
        system_table.boot_services().open_protocol::<LoadedImage>(
            OpenProtocolParams {
                handle: image_handle,
                agent: image_handle,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
    } {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open protocol LoadedImage: {}", e);
            return Status::SUCCESS;
        }
    };

    let device_path = match unsafe {
        system_table.boot_services().open_protocol::<DevicePath>(
            OpenProtocolParams {
                handle: loaded_image.device(),
                agent: image_handle,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
    } {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open protocol DevicePath: {}", e);
            return Status::SUCCESS;
        }
    };

    let mut device_path = device_path.deref();

    let fs_handle = match system_table
        .boot_services()
        .locate_device_path::<SimpleFileSystem>(&mut device_path)
    {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open device path: {}", e);
            return Status::SUCCESS;
        }
    };

    let mut opened_handle = match unsafe {
        system_table
            .boot_services()
            .open_protocol::<SimpleFileSystem>(
                OpenProtocolParams {
                    handle: fs_handle,
                    agent: image_handle,
                    controller: None,
                },
                OpenProtocolAttributes::Exclusive,
            )
    } {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open protocol SimpleFileSystem: {}", e);
            return Status::SUCCESS;
        }
    };

    let file_system = opened_handle.deref_mut();

    let mut root = file_system.open_volume().unwrap();
    let mut buf = [0u16; 256];
    assert!("vmlinuz".len() < 256);
    let filename = CStr16::from_str_with_buf("vmlinuz".trim_end_matches('\0'), &mut buf)
        .expect("Failed to convert string to utf16");

    let file_handle = match root.open(filename, FileMode::Read, FileAttribute::empty()) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open kernel file: {}", e);
            return Status::SUCCESS;
        }
    };

    let mut file = match file_handle.into_type().unwrap() {
        FileType::Regular(f) => f,
        FileType::Dir(_) => {
            error!("Kernel is a directory");
            return Status::SUCCESS;
        }
    };

    info!("Kernel file opened successfully.");

    let mut buf = [0; 512];
    let file_info: &mut FileInfo = file.get_info(&mut buf).unwrap();
    let file_size = usize::try_from(file_info.file_size()).unwrap();

    let file_ptr = system_table
        .boot_services()
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            ((file_size - 1) / 4096) + 1,
        )
        .unwrap() as *mut u8;
    unsafe { ptr::write_bytes(file_ptr, 0, file_size) };
    let file_slice = unsafe { slice::from_raw_parts_mut(file_ptr, file_size) };
    file.read(file_slice).unwrap();

    info!("Kernel image loaded successfully.");

    let setup_end = 0x0202 + (file_slice[0x0201] as u16);
    info!(
        "Setup header start: 0x{:04x}, end: 0x{:04x}",
        SETUP_START, setup_end
    );

    let elf = ElfFile::new(file_slice).unwrap();
    let entry_point = elf.header.pt2.entry_point();
    info!("Jumping into kernel at entry point 0x{:x}...", entry_point);

    unsafe {
        asm!("jmp {}", in(reg) entry_point);
    }

    Status::SUCCESS
}
