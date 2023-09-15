#![no_main]
#![no_std]

use log::info;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;

// const SETUP_START: u16 = 0x01f1;
//
// #[repr(C, packed(2))]
// struct Gdtr {
//     limit: u16,
//     base: u64,
// }
//
// #[no_mangle]
// static GDT: [u64; 6] = [
//     0x0000000000000000,
//     0x0000000000000000,
//     0x00af9a000000ffff,
//     0x00cf92000000ffff,
//     // 0x00af9b000000ffff,
//     // 0x00cf93000000ffff,
//     0x0080890000000000,
//     0x0000000000000000,
// ];
//
// #[no_mangle]
// static mut GDTR: Gdtr = Gdtr {
//     limit: 6 * 8 - 1,
//     base: 0,
// };
//
// #[no_mangle]
// static mut PAGE_TABLE: [u64; 6 * 512] = [0; 6 * 512];

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();

    info!("Initialization successful.");

    let kernel = {
        // let loaded_image = match unsafe {
        //     system_table.boot_services().open_protocol::<LoadedImage>(
        //         OpenProtocolParams {
        //             handle: image_handle,
        //             agent: image_handle,
        //             controller: None,
        //         },
        //         OpenProtocolAttributes::Exclusive,
        //     )
        // } {
        //     Ok(v) => v,
        //     Err(e) => {
        //         error!("Failed to open protocol LoadedImage: {}", e);
        //         return Status::SUCCESS;
        //     }
        // };

        // let device_path_r = match unsafe {
        //     system_table.boot_services().open_protocol::<DevicePath>(
        //         OpenProtocolParams {
        //             handle: loaded_image.device(),
        //             agent: image_handle,
        //             controller: None,
        //         },
        //         OpenProtocolAttributes::Exclusive,
        //     )
        // } {
        //     Ok(v) => v,
        //     Err(e) => {
        //         error!("Failed to open protocol DevicePath: {}", e);
        //         return Status::SUCCESS;
        //     }
        // };

        // let mut device_path = device_path_r.deref();

        // let fs_handle = match system_table
        //     .boot_services()
        //     .locate_device_path::<SimpleFileSystem>(&mut device_path)
        // {
        //     Ok(v) => v,
        //     Err(e) => {
        //         error!("Failed to open device path: {}", e);
        //         return Status::SUCCESS;
        //     }
        // };

        // let mut opened_handle = match unsafe {
        //     system_table
        //         .boot_services()
        //         .open_protocol::<SimpleFileSystem>(
        //             OpenProtocolParams {
        //                 handle: fs_handle,
        //                 agent: image_handle,
        //                 controller: None,
        //             },
        //             OpenProtocolAttributes::Exclusive,
        //         )
        // } {
        //     Ok(v) => v,
        //     Err(e) => {
        //         error!("Failed to open protocol SimpleFileSystem: {}", e);
        //         return Status::SUCCESS;
        //     }
        // };

        // let file_system = opened_handle.deref_mut();

        // let mut root = file_system.open_volume().unwrap();
        // let mut buf = [0u16; 256];
        // assert!("vmlinuz".len() < 256);
        // let filename = CStr16::from_str_with_buf("vmlinuz".trim_end_matches('\0'), &mut buf)
        //     .expect("Failed to convert string to utf16");

        // let file_handle = match root.open(filename, FileMode::Read, FileAttribute::empty()) {
        //     Ok(v) => v,
        //     Err(e) => {
        //         error!("Failed to open kernel file: {}", e);
        //         return Status::SUCCESS;
        //     }
        // };

        // let mut file = match file_handle.into_type().unwrap() {
        //     FileType::Regular(f) => f,
        //     FileType::Dir(_) => {
        //         error!("Kernel is a directory");
        //         return Status::SUCCESS;
        //     }
        // };

        // info!("Kernel file opened successfully.");

        // let mut buf = [0; 512];
        // let file_info: &mut FileInfo = file.get_info(&mut buf).unwrap();
        // let file_size = usize::try_from(file_info.file_size()).unwrap();

        // let file_ptr = system_table
        //     .boot_services()
        //     .allocate_pages(
        //         AllocateType::AnyPages,
        //         MemoryType::LOADER_DATA,
        //         ((file_size - 1) / 4096) + 1,
        //     )
        //     .unwrap() as *mut u8;
        // unsafe { ptr::write_bytes(file_ptr, 0, file_size) };
        // let file_slice = unsafe { slice::from_raw_parts_mut(file_ptr, file_size) };
        // file.read(file_slice).unwrap();

        // info!("Kernel image loaded successfully.");

        // file_slice

        let mut fs = bs.get_image_file_system(bs.image_handle()).unwrap();
        fs.read(cstr16!("vmlinuz")).unwrap()
    };

    // let setup_end = 0x0202 + (file_slice[0x0201] as u16);
    // info!(
    //     "Setup header start: 0x{:04x}, end: 0x{:04x}",
    //     SETUP_START, setup_end
    // );

    // let pg_ptr = unsafe { PAGE_TABLE.as_mut_ptr() };

    // info!("Creating page table at 0x{:x}...", pg_ptr as u64);

    // for i in 0..6 * 512 {
    //     unsafe { *pg_ptr.add(i) = 0 };
    // }

    // // L4
    // unsafe { *pg_ptr = (pg_ptr as u64 + 0x1000) | 0x7 };

    // // L3
    // for i in 0..4 {
    //     unsafe { *pg_ptr.add(0x1000 + i) = (pg_ptr as u64 + 0x2000 + 0x1000 * (i as u64)) | 0x7 };
    // }

    // // L2
    // for i in 0..2048 {
    //     unsafe { *pg_ptr.add(0x2000 + i) = (0x200000 * (i as u64)) | 0x183 };
    // }

    // let elf = ElfFile::new(file_slice).unwrap();
    // let entry_point = elf.header.pt2.entry_point();

    // let gdtr_base = GDT.as_ptr() as u64;
    // info!("Setting up GDT with base address 0x{:x}...", gdtr_base);

    // unsafe {
    //     GDTR.base = gdtr_base;

    //     asm!(
    //         r#"
    //         cli
    //         lgdt [{}]

    //         mov ax, 0x18
    //         mov ds, ax
    //         mov es, ax
    //         mov fs, ax
    //         mov gs, ax
    //         mov ss, ax

    //         lea rax, [white]
    //         push 0x10
    //         push rax
    //         retfq
    //         white:
    //         "#,
    //         in(reg) &GDTR,
    //         options(readonly, nostack, preserves_flags),
    //     );
    // }

    // info!("Enabling PAE...");

    // unsafe {
    //     asm!(
    //         r#"
    //         mov rax, cr4
    //         or rax, 0x20
    //         mov cr4, rax
    //         "#,
    //     );
    // }

    // info!("Enabling paging...");

    // unsafe {
    //     asm!(
    //         r#"
    //         mov cr3, {}
    //         hlt
    //         "#, in(reg) pg_ptr as u64
    //     );
    // }

    // info!("Jumping into kernel at entry point 0x{:x}...", entry_point);

    // unsafe {
    //     asm!("jmp {}", in(reg) entry_point);
    // }

    let cmdline = cstr16!("initrd=\\initrd.img");

    let img = bs
        .load_image(
            image_handle,
            LoadImageSource::FromBuffer {
                buffer: &kernel,
                file_path: None,
            },
        )
        .unwrap();
    let mut limg = bs.open_protocol_exclusive::<LoadedImage>(img).unwrap();
    unsafe { limg.set_load_options(cmdline.as_ptr() as *const _, cmdline.num_bytes() as u32) };
    bs.start_image(img).unwrap();

    Status::SUCCESS
}
