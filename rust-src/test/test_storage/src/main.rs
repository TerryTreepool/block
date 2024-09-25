
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, path::{PathBuf, Path}};

use near_base::now;

struct ProtocData {
    file_path: PathBuf,
    file_name: String,
}

impl ProtocData {
    fn println(&self) {
        println!("cargo:rerun-if-changed={}", self.file_path.display())
    }    
}

#[derive(Default)]
struct ProtocInfo {
    protoc_domain: Option<String>,
    protoc_data_list: Vec<ProtocData>,
    protoc_children: Option<Vec<ProtocInfo>>,
}

impl ProtocInfo {
    fn println(&self) {
        // println!("{:?}", self.protoc_data_list);
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

// struct ProtocInfoArray(Vec<ProtocInfo>);

// impl ProtocInfoArray {

//     fn println(&self) {
//         for item in self.0.iter() {
//             match item {
//                 ProtocInfo::ProtoDir(_, list) => list.println(),
//                 ProtocInfo::ProtoFile(file) => file.println()
//             }
//         }
//     }

// }

fn enum_path(path: &Path) -> ProtocInfo {
    let mut r = ProtocInfo::default();
    let dir = std::fs::read_dir(path).expect("Failed get proto file");

    for file in dir {
        if let Ok(f) = file {
            let file_path = f.path();
            if file_path.is_file() && file_path.extension().unwrap_or_default().eq_ignore_ascii_case("rs") {

                r.protoc_data_list.push(ProtocData {
                        file_name: file_path.file_stem().unwrap().to_string_lossy().to_string(),
                        file_path: file_path,
                    });
                // r.push(ProtocInfo::ProtoFile(ProtocData {
                //     file_name: file_path.file_stem().unwrap().to_string_lossy().to_string(),
                //     file_path: file_path,
                // }));
                // file_array.push(file_path);
            } else if file_path.is_dir() {
                let mut children = enum_path(file_path.as_path());
                if children.protoc_data_list.len() > 0 || children.protoc_children.is_some() {
                    let protoc_domain = file_path.iter().last().unwrap().to_string_lossy().to_string();
                    children.protoc_domain = Some(protoc_domain);
                    println!("{:?}", children.protoc_domain);

                    if let Some(protoc_children) = r.protoc_children.as_mut() {
                        protoc_children.push(children);
                    } else {
                        r.protoc_children = Some(vec![children]);
                    }
                    // r.protoc_children.Some(children);
                }
                // file_array.extend(get_files(std::fs::read_dir(file_path).expect("failed get {file_path} file")));
            }
        }
    }

    r
}

fn main() {
    let r = enum_path(PathBuf::new().join("D:\\p\\CYFS_main-m").as_path());

    // r.println(); 
}
