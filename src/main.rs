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
use uefi::table::boot::{
    AllocateType, MemoryDescriptor, MemoryMap, MemoryType, OpenProtocolAttributes,
    OpenProtocolParams,
};
use uefi::CStr16;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

#[derive(Debug, Copy, Clone)]
struct UefiMemoryDescriptor(pub MemoryDescriptor);

struct FrameAlloc<I, D> {
    memory_map: I,
}

impl<I, D> FrameAlloc<I, D>
where
    I: ExactSizeIterator<Item = D> + Clone,
{
    fn new(memory_map: I) -> Self {
        let start_frame = PhysFrame::containing_address(PhysAddr::new(0x1000));
        Self::new_starting_at(start_frame, memory_map)
    }

    fn new_starting_at(frame: PhysFrame, memory_map: I) -> Self {
        Self { memory_map }
    }
}

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    info!("Initialization successful, accessing ESP...");

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

    let opened_handle = match unsafe {
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

    let mut buf = [0; 500];
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

    info!("Kernel image loaded successfully, exiting boot services...");

    let (system_table, mut memory_map) = system_table.exit_boot_services();
    memory_map.sort();

    let mut frame_allocator =
        FrameAlloc::new(memory_map.entries().copied().map(UefiMemoryDescriptor));

    let phys_offset = VirtAddr::new(0);

    let bootloader_page_table = {
        let old_table = {
            let frame = Cr3::read().0;
            let ptr: *const PageTable = (phys_offset + frame.start_address().as_u64()).as_ptr();
            unsafe { &*ptr }
        };
        let new_frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for new level 4 table");
        let new_table: &mut PageTable = {
            let ptr: *mut PageTable =
                (phys_offset + new_frame.start_address().as_u64()).as_mut_ptr();
            unsafe {
                ptr.write(PageTable::new());
                &mut *ptr
            }
        };

        new_table[0] = old_table[0].clone();

        unsafe {
            Cr3::write(new_frame, Cr3Flags::empty());
            OffsetPageTable::new(&mut *new_table, phys_offset)
        }
    };

    let (kernel_page_table, kernel_level_4_frame) = {
        let frame: PhysFrame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for new kernel level 4 table");
        info!("New page table at {:#?}", &frame);

        let addr = phys_offset + frame.start_address().as_u64();
        let ptr = addr.as_mut_ptr();
        unsafe { *ptr = PageTable::new() };
        let level_4_table = unsafe { &mut *ptr };
        (
            unsafe { OffsetPageTable::new(level_4_table, phys_offset) },
            frame,
        )
    };

    unsafe {
        asm!(
            r#"
            xor rbp, rbp
            mov cr3, {}
            mov rsp, {}
            push 0
            jmp {}
            "#,
            in(reg) kernel_level_4_frame.start_address().as_u64(),
            in(reg) stack_top.as_u64(),
            in(reg) file_ptr.as_u64(),
        );
    }

    unreachable!()
}
