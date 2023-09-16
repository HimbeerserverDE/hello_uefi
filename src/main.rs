#![no_main]
#![no_std]

use log::info;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();

    info!("Initialization successful.");

    let kernel = {
        let mut fs = bs.get_image_file_system(bs.image_handle()).unwrap();
        let img = fs.read(cstr16!("vmlinuz-linux-hardened")).unwrap();

        info!("Kernel image loaded successfully.");
        img
    };

    let cmdline = cstr16!("loglevel=3 quiet cryptdevice=UUID=48a1c620-1a69-4daf-9e16-110670e0b16e:root_crypt root=UUID=d1752d46-78cf-44a2-8bc7-e67086f59bea ro rootflags=subvol=root");

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
