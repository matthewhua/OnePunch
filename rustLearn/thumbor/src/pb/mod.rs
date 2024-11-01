use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use photon_rs::transform::SamplingFilter;
use prost::Message;

mod abi;

pub use abi::*;
use crate::pb::filter::FilterType;

impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

// 让 ImageSpec 可以生成一个字符串
impl From<&ImageSpec> for String {
    fn from(image_spec: &ImageSpec) -> Self {
        let data = image_spec.encode_to_vec();
        URL_SAFE_NO_PAD.encode(data)
    }
}

// 更新 TryFrom<&str> for ImageSpec 实现
impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = URL_SAFE_NO_PAD.decode(value)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

// 辅助函数，photon_rs 相应的方法里需要字符串
impl filter::FilterType {
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            FilterType::Unspecified => None,
            FilterType::Oceanic => Some("oceanic"),
            FilterType::Islands => Some("islands"),
            FilterType::Marine => Some("marine"),
        }
    }
}

// 在我们定义的 SampleFilter 和 photon_rs 的 SamplingFilter 间转换
impl From<resize::SampleFilter> for SamplingFilter {
    fn from(v: resize::SampleFilter) -> Self {
        match v {
            resize::SampleFilter::Undefined => SamplingFilter::Nearest,
            resize::SampleFilter::Nearest => SamplingFilter::Nearest,
            resize::SampleFilter::Triangle => SamplingFilter::Triangle,
            resize::SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            resize::SampleFilter::Gaussian => SamplingFilter::Gaussian,
            resize::SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

// 提供一些辅助函数，让创建一个 spec 的过程简单一些
impl Spec {
    pub fn new_resize_seam_carve(width: u32, height: u32) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                r_type: resize::ResizeType::SeamCarve as i32,
                filter: resize::SampleFilter::Undefined as i32,
            })),
        }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                r_type: resize::ResizeType::Normal as i32,
                filter: filter as i32,
            })),
        }
    }

    pub fn new_filter(filter: filter::FilterType) -> Self {
        Self {
            data: Some(spec::Data::Filter(Filter {
                filter: filter as i32,
            })),
        }
    }

    pub fn new_watermark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::Watermark(Watermark { x, y }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use crate::pb::{filter, resize, ImageSpec, Spec};

    #[test]
    fn encoded_spec_could_be_decoded() {
        let spec1 = Spec::new_resize(600, 600, resize::SampleFilter::CatmullRom);
        let spec2 = Spec::new_filter(filter::FilterType::Marine);
        let image_spec = ImageSpec::new(vec![spec1, spec2]);
        let x: String = image_spec.borrow().into();
        assert_eq!(image_spec, x.as_str().try_into().unwrap());
    }
}


