use quote::{format_ident, quote};
use std::{collections::BTreeSet, env, fs, path::PathBuf, process::Command};

fn run(command: &mut Command) -> String {
    let output = command.output().expect("execute native build command");
    if !output.status.success() {
        panic!(
            "{command:?}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).expect("tool output is UTF-8")
}
fn input(name: &str) -> PathBuf {
    println!("cargo:rerun-if-env-changed={name}");
    PathBuf::from(
        env::var_os(name).unwrap_or_else(|| panic!("{name} must name the native build input")),
    )
}
fn main() {
    assert_eq!(
        env::var("CARGO_CFG_TARGET_OS").unwrap(),
        "scarlet",
        "this final link is Native only"
    );
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        "aarch64",
        "native export ABI currently verified on AArch64"
    );
    let include = input("DEP_SCARLET_C_INCLUDE").canonicalize().unwrap();
    let native = input("DEP_SCARLET_C_NATIVE").canonicalize().unwrap();
    let headers_source = input("VULKAN_HEADERS_SOURCE_DIR").canonicalize().unwrap();
    let llvm_libc = input("SCARLET_LLVM_LIBC_SOURCE_DIR")
        .canonicalize()
        .unwrap();
    let cc = input("SCARLET_NATIVE_CC");
    let nm = input("SCARLET_NATIVE_NM");
    let ar = input("SCARLET_NATIVE_AR");
    let cxx = input("SCARLET_NATIVE_CXX");
    let ranlib = input("SCARLET_NATIVE_RANLIB");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();
    let build = out.join("cmake");
    let headers = out.join("headers");
    let cmake = env::var("CMAKE").unwrap_or_else(|_| "cmake".into());
    run(Command::new(&cmake)
        .arg("-S")
        .arg(&headers_source)
        .arg("-B")
        .arg(out.join("headers-build"))
        .args(["-G", "Ninja", "-DVULKAN_HEADERS_ENABLE_TESTS=OFF"])
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", headers.display())));
    run(Command::new(&cmake)
        .arg("--install")
        .arg(out.join("headers-build")));
    let resource = run(Command::new(&cc).arg("-print-resource-dir"))
        .trim()
        .to_string();
    let flags = format!(
        "-nostdinc -isystem {resource}/include -isystem {} -D__scarlet__=1 -ffunction-sections -fdata-sections -fPIC -mno-outline-atomics",
        include.display()
    );
    let configure = |extra: &str| {
        run(Command::new(&cmake)
            .arg("-S")
            .arg(&source)
            .arg("-B")
            .arg(&build)
            .args(["-G", "Ninja"])
            .arg(format!(
                "-DCMAKE_TOOLCHAIN_FILE={}/cmake/aarch64-scarlet.cmake",
                native.display()
            ))
            .arg(format!("-DCMAKE_C_COMPILER={}", cc.display()))
            .arg(format!("-DCMAKE_ASM_COMPILER={}", cc.display()))
            .arg(format!("-DCMAKE_AR={}", ar.display()))
            .arg(format!("-DCMAKE_RANLIB={}", ranlib.display()))
            .arg(format!(
                "-DVulkanHeaders_DIR={}/share/cmake/VulkanHeaders",
                headers.display()
            ))
            .arg(format!("-DCMAKE_C_FLAGS={flags} {extra}"))
            .arg(format!("-DCMAKE_ASM_FLAGS={flags}"))
            .args([
                "-DCMAKE_BUILD_TYPE=Release",
                "-DSCARLET_STATIC_LOADER=ON",
                "-DBUILD_TESTS=OFF",
                "-DUSE_GAS=ON",
                "-DSYSCONFDIR=/etc",
                "-DFALLBACK_DATA_DIRS=/share:/usr/share",
                "-DHAVE_ALLOCA_H=1",
            ]));
        run(Command::new(&cmake)
            .arg("--build")
            .arg(&build)
            .args(["--parallel", "4"]));
    };
    // Collect implemented public C entry points, including enabled WSI only.
    configure("");
    let archive = build.join("loader/libvulkan.a");
    let defined = run(Command::new(nm)
        .args(["--defined-only", "--extern-only"])
        .arg(&archive));
    let symbols: BTreeSet<_> = defined
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .filter(|name| name.starts_with("vk"))
        .map(str::to_owned)
        .collect();
    let mut bindings = bindgen::Builder::default()
        .header(headers.join("include/vulkan/vulkan.h").to_str().unwrap())
        .clang_arg("--target=aarch64-unknown-none-elf")
        .clang_arg("-nostdinc")
        .clang_arg(format!("-I{}/include", headers.display()))
        .clang_arg(format!("-isystem{resource}/include"))
        .clang_arg(format!("-isystem{}", include.display()))
        .layout_tests(false)
        .generate_comments(false);
    for symbol in &symbols {
        bindings = bindings.allowlist_function(format!("^{symbol}$"));
    }
    let bindings = bindings
        .generate()
        .expect("generate C ABI from matching Vulkan headers")
        .to_string();
    let mut parsed = syn::parse_file(&bindings).unwrap();
    let mut wrappers = Vec::new();
    let mut rename = String::new();
    for item in &mut parsed.items {
        if let syn::Item::ForeignMod(module) = item {
            module
                .attrs
                .push(syn::parse_quote!(#[link(name = "vulkan", kind = "static")]));
            for item in &mut module.items {
                if let syn::ForeignItem::Fn(function) = item {
                    let original = function.sig.ident.clone();
                    let implementation = format_ident!("scarlet_loader_{}", original);
                    let mut signature = function.sig.clone();
                    signature.unsafety = Some(syn::parse_quote!(unsafe));
                    signature.abi = Some(syn::parse_quote!(extern "C"));
                    let arguments: Vec<_> = signature
                        .inputs
                        .iter()
                        .map(|argument| match argument {
                            syn::FnArg::Typed(argument) => argument.pat.clone(),
                            _ => panic!("unexpected method in C ABI"),
                        })
                        .collect();
                    wrappers.push(quote! { #[unsafe(no_mangle)] pub #signature { unsafe { #implementation(#(#arguments),*) } } });
                    rename.push_str(&format!("#define {original} {implementation}\n"));
                    function.sig.ident = implementation;
                }
            }
        }
    }
    // The C implementation remains upstream code. Typed forwards let rustc
    // export its C ABI while applying the native CRT and ELF OSABI itself.
    let rename_header = out.join("loader_rename.h");
    fs::write(&rename_header, rename).unwrap();
    configure(&format!("-include {}", rename_header.display()));
    fs::write(
        out.join("loader_exports.rs"),
        quote! { #parsed #(#wrappers)* }.to_string(),
    )
    .unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        build.join("loader").display()
    );
    // cJSON needs correctly rounded strtod; use the native libc companion's
    // real LLVM conversion source, without pulling in an executable startup.
    for file in ["float-conversion.cpp", "scanf.cpp"] {
        let float_source = native.join(file);
        let float_object = out.join(format!("{file}.o"));
        run(Command::new(&cxx)
            .args([
                "--target=aarch64-unknown-none-elf",
                "-O3",
                "-std=c++23",
                "-fPIC",
                "-nostdinc",
                "-nostdinc++",
                "-fno-exceptions",
                "-fno-rtti",
                "-mno-outline-atomics",
                "-ffunction-sections",
                "-fdata-sections",
                "-D__scarlet__=1",
            ])
            .arg(format!("-isystem{resource}/include"))
            .arg(format!("-isystem{}", include.display()))
            .arg(format!("-I{}", llvm_libc.display()))
            .arg("-c")
            .arg(&float_source)
            .arg("-o")
            .arg(&float_object));
        run(Command::new(&ar)
            .arg("crs")
            .arg(out.join("libscarlet_loader_float.a"))
            .arg(float_object));
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=scarlet_loader_float");
        println!("cargo:rerun-if-changed={}", float_source.display());
    }
    for argument in [
        "--hash-style=both",
        "-z",
        "now",
        "--entry=0",
        "--exclude-libs=ALL",
        "-soname",
        "libvulkan.so.1",
        "--unresolved-symbols=ignore-all",
    ] {
        println!("cargo:rustc-link-arg={argument}");
    }
    println!("cargo:rerun-if-changed=../loader");
    println!("cargo:rerun-if-changed=../CMakeLists.txt");
    println!("cargo:rerun-if-changed={}", include.display());
}
