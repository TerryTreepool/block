
use std::sync::Arc;

use near_base::{*, sequence::SequenceString};
use crate::{network::MTU, CreatorMeta};

use super::{DynamicPackage, PackageHeader, 
            package::Context, package_header::PackageHeaderExt, Data, AnyNamedRequest, };

struct PackageDataSetImpl {
    package: DynamicPackage,
    dataset: Vec<Data>,
}

#[derive(Clone)]
pub struct PackageDataSet(Arc<PackageDataSetImpl>);

impl std::fmt::Display for PackageDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{package={}, dataset-len={}}}", self.0.package, self.0.dataset.len())
    }
}

impl From<(DynamicPackage, Vec<Data>)> for PackageDataSet {
    fn from(cx: (DynamicPackage, Vec<Data>)) -> Self {
        let (package, dataset) = cx;

        Self(Arc::new(PackageDataSetImpl{
            package, dataset
        }))
    }
}

impl PackageDataSet {
    #[inline]
    #[allow(unused)]
    pub fn package_head(&self) -> &PackageHeader {
        self.0.package.as_head()
    }

    #[inline]
    #[allow(unused)]
    pub fn package(&self) -> &DynamicPackage {
        &self.0.package
    }

    pub fn dataset_count(&self) -> usize {
        self.0.dataset.len()
    }

    pub fn dataset(&self, index: usize) -> Option<&Data> {
        self.0.dataset.get(index)
    }
}

pub struct PackageBuilder {
    head: PackageHeader,
    head_ext: PackageHeaderExt,
    body: Option<AnyNamedRequest>,
}

impl PackageBuilder {

    pub fn build_head(seq: SequenceString) -> Self {
        Self {
            head: PackageHeader::default().set_sequence(seq),
            head_ext: PackageHeaderExt::default(),
            body: None,
        }
    }

    pub fn build_topic(mut self, 
                       creator: Option<CreatorMeta>, 
                       requestor: ObjectId, 
                       to: ObjectId, 
                       topic: Option<String>) -> Self {

        let (creator, local, remote) = 
            if let Some(creator) = creator {
                creator.split()
            } else {
                (None, None, None)
            };

        self.head_ext = self.head_ext
                            .set_creator(creator)
                            .set_endpoint(local, remote)
                            .set_requestor(requestor)
                            .set_to(to)
                            .set_topic(topic);
        self
    }

    pub fn build_body(mut self, body: AnyNamedRequest) -> Self {
        self.head = self.head.set_major_command(body.major_command());
        self.body = Some(body);
        self
    }

    pub async fn build<'a>(
        self,
        signer: Option<Box<dyn SignerTrait>>
    ) -> NearResult<PackageDataSet> {
        let mut cx = Context::init();
        let head_ext = {
            cx.serialize_headext(&self.head_ext)?;
            self.head_ext
        };

        let body = {
            let body = self.body.unwrap();
            cx.serialize_body(&body)?;
            body
        };
        let signature_data = cx.serialize_sign(signer.as_ref()).await?;

        let package_slice = cx.finish();
        let package_head_len = PackageHeader::raw_bytes();
        let package_head_ext_len = package_slice.head_ext.len();

        // rebuild package length, if it need split package.
        let package_body_max = MTU - (package_head_len + package_head_ext_len);

        struct SplitPackage<'a> {
            data: &'a [u8],
        }

        impl<'a> SplitPackage<'a> {
            fn split(self, mid: usize) -> Vec<&'a [u8]> {
                if self.data.len() > mid {
                    let (l, r) = self.data.split_at(mid);
                    [vec![l], SplitPackage{ data: r }.split(mid)].concat()
                } else {
                    vec![self.data]
                }
            }
        }

        let package_array = SplitPackage{data: package_slice.remain_data}.split(package_body_max);
        let package_body_count = package_array.len();
        let package_head = self.head.set_count(package_body_count as u8);

        let mut dataset = vec![];

        for index in 0..package_body_count {
            let package_body_ref = package_array[index];
            let package_body_ref_len = package_body_ref.len();

            let package_len = PackageHeader::raw_bytes() + package_head_ext_len + package_body_ref_len;
            let mut buff = vec![0u8; package_len];

            // build head
            let data = {
                let head = package_head.clone()
                                                      .set_index(index as u8)
                                                      .set_count(package_body_count as u8)
                                                      .set_length(package_len as u16);

                head.serialize(buff.as_mut_slice())
            }?;

            // build head ext
            let data = {
                unsafe {
                    std::ptr::copy(package_slice.head_ext.as_ptr(), data.as_mut_ptr(), package_head_ext_len);
                }
                &mut data[package_head_ext_len..]
            };

            // build body
            unsafe {
                std::ptr::copy(package_body_ref.as_ptr(), data.as_mut_ptr(), package_body_ref_len);
            }

            dataset.push(Data::with_data(buff));
        }

        Ok(PackageDataSet::from((DynamicPackage::from((package_head, head_ext, body, signature_data)), dataset)))
    }

}

pub struct SequenceBuild<'build> {
    pub(crate) requestor: &'build ObjectId,
    pub(crate) now: Timestamp,
    pub(crate) sync_times: u32,
}

impl SequenceBuild<'_> {
    pub(crate) fn build(self) -> NearResult<SequenceString> {
        let mut buf = vec![0u8; self.requestor.raw_capacity() + self.now.raw_capacity() + self.sync_times.raw_capacity()];

        let end = self.requestor.serialize(&mut buf)?;
        let end = self.now.serialize(end)?;
        let _end = self.sync_times.serialize(end)?;

        Ok(hash_data(&buf).as_slice().into())
    }
}

// #[test]
// fn test_package_build() {
//     use crate::{package::{AnyNamedRequest, Request}};

//     async_std::task::block_on(async move {

//         #[derive(Debug)]
//         struct X(Vec<u8>);

//         let mut a = vec![];
//         for i in 0..8192 {
//             a.push(i as u8);
//         }

//         let r = 
//         PackageBuilder::build_head(SequenceValue::default())
//             .build_topic(None, ObjectId::default(), None, Some("/test/package-builder".to_string()))
//             .build_body(AnyNamedRequest::with_request(Request::new(X(a))))
//             .build(None)
//             .await
//             .unwrap();

//         println!("{}", r);
//     });
// }

#[test]
fn test_split_package() {
    struct SplitPackage<'a> {
        data: &'a [u8],
    }

    impl<'a> SplitPackage<'a> {
        fn split(self, mid: usize) -> Vec<&'a [u8]> {
            if self.data.len() > mid {
                let (l, r) = self.data.split_at(mid);
                [vec![l], SplitPackage{ data: r }.split(mid)].concat()
            } else {
                vec![self.data]
            }
        }
    }


    let buff = "01234567890123456789".as_bytes();

    let slices = SplitPackage{data: buff}.split(7);
    for it in slices {
        println!("{:?}", it);
    }
}