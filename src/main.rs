fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");
    
    println!("uefi_path: {}", uefi_path);
    println!("bios_path: {}", bios_path);
    println!("ovmf_pure_efi: {}", ovmf_prebuilt::ovmf_pure_efi().display());
    // choose whether to start the UEFI or BIOS image
    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-m").arg("2G");
        //cmd.arg("-device").arg("virtio-gpu-pci");
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    }
    println!("command: {:?}", cmd);
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}

