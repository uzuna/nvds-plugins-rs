
.PHONY: bindgen
bindgen: src/imp.rs

src/imp.rs:
	bindgen /opt/nvidia/deepstream/deepstream/sources/includes/nvdsmeta.h -o src/imp.rs -- -I/usr/include/glib-2.0 -I/usr/lib/aarch64-linux-gnu/glib-2.0/include

	sed -i "1i#![allow(non_upper_case_globals)]\n#![allow(non_camel_case_types)]\n#![allow(non_snake_case)]\n#![allow(dead_code)]\n" src/imp.rs