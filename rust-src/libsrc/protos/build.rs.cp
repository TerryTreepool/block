
use std::{path::{PathBuf, Path}, sync::Mutex, io::Write, };

use protobuf_codegen::{Customize, CustomizeCallback};

extern crate protobuf_codegen;
extern crate prost_build;

pub const OUT_DIR: &str = "src/protos";
pub const INNER_MAROC: &str = 
r#"
macro_rules! inner_impl_default_protobuf_raw_codec {
    ($proto_name:ty) => {
        impl near_base::Serialize for $proto_name {
            fn raw_capacity(&self) -> usize {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::raw_capacity(self)
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::serialize(self, buf)
            }
        }

        impl near_base::Deserialize for $proto_name {
            fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::deserialize(buf)
            }
        }
    }
}
"#;

fn gen_descriptor(file_array: &[impl AsRef<Path>]) {

    let path = PathBuf::new().join(OUT_DIR).join("bin");
    std::fs::create_dir_all(&path)
        .expect("failed create_dir_all()");

    prost_build::Config::new()
        .file_descriptor_set_path(path.join("near_descriptor_set.bin"))
        .compile_protos(file_array, &["protos"])
        .expect("failed gen descriptor");

}

struct GenProtoc {
    file: PathBuf
}

fn gen_protoc(file_array: &[impl AsRef<Path>]) {

    pub struct CustomizeCallbackImpl<T: Write>(Mutex<T>);

    impl<T: Write + 'static> CustomizeCallback for CustomizeCallbackImpl<T> {
        fn file(&self, file: &protobuf::reflect::FileDescriptor) -> Customize {
            let file_name = file.name();

            let package = 
                    file_name
                        .strip_suffix(".proto")
                        .unwrap_or(file_name);

            let fs = &mut *self.0.lock().unwrap();
            let _ = fs.write(format!("pub mod {};\n", package).as_bytes());
            let _ = fs.flush();

            Customize::default()
        }

        fn message(&self, message: &protobuf::reflect::MessageDescriptor) -> Customize {

            let package = 
                 message.file_descriptor()
                        .name()
                        .strip_suffix(".proto")
                        .unwrap_or(message.file_descriptor().name());

            let message = {
                let name = message.name();
                name[0..1].to_uppercase() + &name[1..]
            };

            let fs = &mut *self.0.lock().unwrap();
            let _ = fs.write(format!("inner_impl_default_protobuf_raw_codec!({package}::{});\n", message).as_bytes());
            let _ = fs.flush();

            Customize::default()
        }
    }

    let mut fs = 
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(PathBuf::new().join(OUT_DIR).join("mod.rs")).expect("write protos mod error");

    let _ = fs.write_all("// @by near generated\n".as_bytes());
    let _ = fs.write_all(INNER_MAROC.as_bytes());

    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(OUT_DIR)
        .inputs(file_array)
        .include("protos")
        .customize(Customize::default().generate_accessors(true).gen_mod_rs(false))
        .customize_callback(CustomizeCallbackImpl(Mutex::new(fs)))
        .run()
        .expect("Codegen failed.");
}

fn get_files(dir: std::fs::ReadDir) -> Vec<PathBuf> {
    let mut file_array = vec![];

    for file in dir {
        if let Ok(f) = file {
            let file_path = f.path();
            if file_path.is_file() && file_path.extension().unwrap_or_default().eq_ignore_ascii_case("proto") {
                file_array.push(file_path);
            } else if file_path.is_dir() {
                file_array.extend(get_files(std::fs::read_dir(file_path).expect("failed get {file_path} file")));
            }
        }
    }

    file_array
}

fn main() {
    let dir = std::fs::read_dir("protos").expect("Failed get proto file");

    let file_array = get_files(dir);

    file_array.iter()
              .for_each(| f | {
                println!("cargo:rerun-if-changed={}", f.display());
              });

    std::fs::create_dir_all(OUT_DIR).expect("create dir failed.");

    gen_protoc(&file_array);
    gen_descriptor(&file_array);

}
