# hello_uefi

Build
=====

```
cargo build --target x86_64-unknown-uefi
```

Run
===

```
qemu-system-x86_64 -enable-kvm -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd -drive format=raw,file=fat:rw:esp
```
