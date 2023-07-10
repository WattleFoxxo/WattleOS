cp target/debug/os iso/boot/wattleos-kernel
mv iso/boot/os iso/boot/wattleos-kernel
grub2-mkrescue -o grub.iso iso