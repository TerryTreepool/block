
extern crate cc;

fn main() {
    cc::Build::new()
        .include("./bluez-libs/include")
        .file("./bluez-libs/bluetooth.c")
        .file("./bluez-libs/hci.c")
        .file("./bluez-libs/sdp.c")
        .file("./bluez-libs/parser.c")
        .shared_flag(true)
        .warnings(false)
        .compile("libbluex-libs.so");

    println!("cargo:rerun-if-changed=src/hci.c");
    println!("cargo:rerun-if-changed=src/sdp.c");
    println!("cargo:rerun-if-changed=src/parser.c");
    println!("cargo:rerun-if-changed=src/bluetooth.c");
    println!("cargo:rerun-if-changed=src/include/hci_lib.h");

    let mut include_dir = vec![];
    std::env::var("TOOLCHAIN_INC")
        .expect("NOT FOUND {TOOLCHAIN_INC}")
        .split(';')
        .for_each(| item | {
            include_dir.push(format!("-I{item}"));
        });

        let b = 
        bindgen::builder()
            .header("./bluez-libs/include/bluetooth.h")
            .header("./bluez-libs/include/bnep.h")
            .header("./bluez-libs/include/cmtp.h")
            .header("./bluez-libs/include/hci.h")
            .header("./bluez-libs/include/hci_lib.h")
            .header("./bluez-libs/include/hidp.h")
            .header("./bluez-libs/include/l2cap.h")
            .header("./bluez-libs/include/rfcomm.h")
            .header("./bluez-libs/include/sco.h")
            .header("./bluez-libs/include/sdp.h")
            .header("./bluez-libs/include/sdp_lib.h")
            .clang_args(include_dir)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("failed");

    b.write_to_file("./src/bindgen.rs").expect("failed");

}
