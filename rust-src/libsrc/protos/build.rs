
use std::{path::{PathBuf, Path}, sync::Mutex, io::Write, };

use protobuf_codegen::{Customize, CustomizeCallback};

extern crate protobuf_codegen;
extern crate prost_build;

pub const OUT_DIR: &str = "src/protos";

#[allow(unused)]
fn gen_descriptor(file_array: &[impl AsRef<Path>]) {

    let path = PathBuf::new().join(OUT_DIR).join("bin");
    std::fs::create_dir_all(&path)
        .expect("failed create_dir_all()");

    prost_build::Config::new()
        .file_descriptor_set_path(path.join("near_descriptor_set.bin"))
        .compile_protos(file_array, &["protos"])
        .expect("failed gen descriptor");

}

struct GenProtoc<'a> {
    protoc_include: &'a PathBuf,
    file_array: Vec<&'a ProtocData>,
    file_domain: Vec<String>,
    output_dir: &'a PathBuf,
}

impl GenProtoc<'_> {
    fn gen_protoc(&self) {

        pub struct CustomizeCallbackImpl<T: Write>(Mutex<T>);

        impl<T: Write + 'static> CustomizeCallback for CustomizeCallbackImpl<T> {
            fn file(&self, file: &protobuf::reflect::FileDescriptor) -> Customize {
                let file_name = PathBuf::new().join(file.name().to_owned());

                let package = file_name.file_stem().unwrap().to_string_lossy().to_string();

                let fs = &mut *self.0.lock().unwrap();
                let _ = fs.write(format!("pub mod {};\n", package).as_bytes());
                let _ = fs.flush();

                Customize::default()
            }

            fn message(&self, message: &protobuf::reflect::MessageDescriptor) -> Customize {

                let file_name = PathBuf::new().join(message.file_descriptor().name().to_owned());

                let package = file_name.file_stem().unwrap().to_string_lossy().to_string();

                let message = {
                    let name = message.name();
                    name[0..1].to_uppercase() + &name[1..]
                };

                let fs = &mut *self.0.lock().unwrap();
                let _ = fs.write(format!("inner_impl_default_protobuf_raw_codec!({package}::{message});\n").as_bytes());
                let _ = fs.flush();

                Customize::default()
            }
        }

        let _ = std::fs::create_dir_all(self.output_dir);

        let mut fs = {
            let out_dir_mod = self.output_dir.join("mod.rs");
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(out_dir_mod.as_path()).expect(&format!("write protos {} error", out_dir_mod.as_path().display()))
        };

        let _ = fs.write_all("// @by near generated\n".as_bytes());
        let _ = fs.write_all("use crate::inner_impl_default_protobuf_raw_codec;\n\n".as_bytes());
        for domain in self.file_domain.iter() {
            let _ = fs.write_all(format!("pub mod {domain};\n").as_bytes());
        }

        let file_array: Vec<&PathBuf> =
            self.file_array.iter()
                .map(| i | {
                    &i.file_path
                })
                .collect();

        protobuf_codegen::Codegen::new()
            .pure()
            .out_dir(self.output_dir.as_path())
            .inputs(file_array)
            .include(self.protoc_include.as_path())
            // .include("protos")
            .customize(Customize::default().generate_accessors(true).gen_mod_rs(false))
            .customize_callback(CustomizeCallbackImpl(Mutex::new(fs)))
            .run()
            .expect("Codegen failed.");
    }
}


struct ProtocData {
    file_path: PathBuf,
}

impl ProtocData {
    fn println(&self) {
        println!("cargo:rerun-if-changed={}", self.file_path.display())
    }
}

#[derive(Default)]
struct ProtocInfo {
    protoc_include: PathBuf,
    protoc_domain: Option<String>,
    protoc_data_list: Vec<ProtocData>,
    protoc_children: Option<Vec<ProtocInfo>>,
}

impl ProtocInfo {
    fn build(path: &Path) -> ProtocInfo {
        let mut r = ProtocInfo::default();
        let dir = std::fs::read_dir(path).expect("Failed get proto file");

        r.protoc_include = PathBuf::from(path);

        for file in dir {
            if let Ok(f) = file {
                let file_path = f.path();
                if file_path.is_file() && file_path.extension().unwrap_or_default().eq_ignore_ascii_case("proto") {
                    r.protoc_data_list.push(ProtocData {
                            file_path: file_path,
                        });
                } else if file_path.is_dir() {
                    let mut children = ProtocInfo::build(file_path.as_path());
                    if children.protoc_data_list.len() > 0 || children.protoc_children.is_some() {

                        let protoc_domain = file_path.iter().last().unwrap().to_string_lossy().to_string();
                        children.protoc_domain = Some(protoc_domain);

                        if let Some(protoc_children) = r.protoc_children.as_mut() {
                            protoc_children.push(children);
                        } else {
                            r.protoc_children = Some(vec![children]);
                        }
                    }
                }
            }
        }

        r
    }

    fn gen_protoc(&self, out_dir: PathBuf) {
        GenProtoc {
            protoc_include: &self.protoc_include,
            file_array: self.protoc_data_list.iter().collect(),
            file_domain: {
                if let Some(children) = self.protoc_children.as_ref() {
                    children.iter()
                        .map(| info | {
                            info.protoc_domain.as_ref().unwrap().clone()
                        })
                        .collect()
                } else {
                    vec![]
                }
            },
            output_dir: &out_dir,
        }
        .gen_protoc();

        if let Some(children) = self.protoc_children.as_ref() {
            for child in children {
                let domain =
                    if let Some(domain) = child.protoc_domain.as_ref() {
                        domain.as_str()
                    } else {
                        ""
                    };

                child.gen_protoc(out_dir.join(domain));
            }
        }
    }

    fn println(&self) {
        self.protoc_data_list.iter()
            .for_each(| data | {
                data.println();
            });

        if let Some(children) = self.protoc_children.as_ref() {
            children.iter()
                .for_each(| child | {
                    child.println();
                });
        }
    }
}

fn main() {
    let protoc_file_info = ProtocInfo::build(PathBuf::new().join("protos").as_path());

    protoc_file_info.println();

    protoc_file_info.gen_protoc(PathBuf::new().join(OUT_DIR));
}
