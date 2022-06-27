fn main() {
    println!("cargo:rustc-link-search=native=/opt/nvidia/deepstream/deepstream/lib/");
    println!("cargo:rustc-link-lib=dylib=nvdsgst_meta");
}
