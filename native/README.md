# Scarlet Native Vulkan loader

This directory links the upstream loader C archive with Scarlet’s normal Rust CRT
and native libc. The result is a Release AArch64 ELF shared object with Scarlet
OSABI 83 and SONAME `libvulkan.so.1`; the file is not retagged after linking.

The native platform currently supports offscreen Vulkan and driver-provided
image sharing. Linux/Apple WSI, executable-path diagnostics, and `dladdr` are not
available. Native `scarlet-ld` requires immediate binding and global scope;
loaded objects remain mapped until process exit. Driver manifests use the normal
Khronos discovery code, including `/share/vulkan/icd.d` and `VK_DRIVER_FILES`.

Build with a Scarlet Rust toolchain whose std includes the TLS allocator-owner
fix (petitstrawberry/rust commit 26d00a8a3a8489d73f5ae3575880e9f915dfd0f2).
Set these build inputs to canonical source/tool locations:

- `SCARLET_SOURCE_DIR`: Scarlet source, including native libc headers and companion C++ sources.
- `VULKAN_HEADERS_INSTALL_DIR`: matching Vulkan-Headers 1.4.365 installation.
- `SCARLET_LLVM_LIBC_SOURCE_DIR`: LLVM libc source used by Scarlet’s native conversion/scanning code.
- `SCARLET_NATIVE_CC`, `SCARLET_NATIVE_CXX`: raw Clang 21 executables.
- `SCARLET_NATIVE_AR`, `SCARLET_NATIVE_RANLIB`, `SCARLET_NATIVE_NM`: LLVM archive/symbol tools.
- `LIBCLANG_PATH`: libclang directory for typed export generation.

```sh
RUSTFLAGS=-Zdefault-visibility=hidden cargo build --locked --release \
  --manifest-path native/Cargo.toml --target aarch64-unknown-scarlet
```

CMake and Ninja must be on PATH. CMake compiles the upstream C code; Rust
forwards expose only implemented, correctly typed Vulkan entry points. LLVM
libc supplies real floating-point conversion and scanning, without introducing
an executable startup. Final ELF auditing must reject unexpected imports,
`DT_NEEDED`, ELF TLS, unsupported relocations, and exposed CRT/Rust internals.
The expected imports are `dlopen`, `dlsym`, `dlclose`, `dlerror`, and weak
`environ`. This development port is not a Vulkan conformance claim.
