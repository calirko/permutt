use anyhow::{Context, Result};
use image::ImageFormat;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOutputFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
}

impl ImageOutputFormat {
    pub fn all() -> &'static [ImageOutputFormat] {
        &[
            ImageOutputFormat::Png,
            ImageOutputFormat::Jpeg,
            ImageOutputFormat::Webp,
            ImageOutputFormat::Gif,
        ]
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ImageOutputFormat::Png => "png",
            ImageOutputFormat::Jpeg => "jpg",
            ImageOutputFormat::Webp => "webp",
            ImageOutputFormat::Gif => "gif",
        }
    }

    pub fn as_image_format(&self) -> ImageFormat {
        match self {
            ImageOutputFormat::Png => ImageFormat::Png,
            ImageOutputFormat::Jpeg => ImageFormat::Jpeg,
            ImageOutputFormat::Webp => ImageFormat::WebP,
            ImageOutputFormat::Gif => ImageFormat::Gif,
        }
    }
}

impl std::fmt::Display for ImageOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

pub struct ConvertJob {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub format: ImageOutputFormat,
}

pub struct ConvertResult {
    pub success: bool,
    pub error: Option<String>,
}

pub fn convert(job: &ConvertJob) -> ConvertResult {
    match do_convert(job) {
        Ok(_) => ConvertResult {
            success: true,
            error: None,
        },
        Err(e) => ConvertResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}

fn do_convert(job: &ConvertJob) -> Result<PathBuf> {
    let stem = job
        .input
        .file_stem()
        .context("no file stem")?
        .to_string_lossy();

    let output_path = job
        .output_dir
        .join(format!("{}.{}", stem, job.format.extension()));

    std::fs::create_dir_all(&job.output_dir).context("failed to create output directory")?;

    let img = load_image(&job.input)?;
    img.save_with_format(&output_path, job.format.as_image_format())
        .context("failed to save image")?;

    Ok(output_path)
}

fn load_image(path: &Path) -> Result<image::DynamicImage> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "heic" | "heif" => load_heic(path),
        _ => image::open(path).context("failed to open image"),
    }
}

fn load_heic(path: &Path) -> Result<image::DynamicImage> {
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let path_str = path.to_str().context("non-UTF8 path")?;
    let lib = LibHeif::new();
    let ctx = HeifContext::read_from_file(path_str)?;
    let handle = ctx.primary_image_handle()?;
    let heif_img = lib.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)?;

    let planes = heif_img.planes();
    let plane = planes.interleaved.context("no interleaved plane in HEIC")?;

    let width = heif_img.width();
    let height = heif_img.height();
    let stride = plane.stride;

    // libheif may pad rows; copy row-by-row to strip padding
    let row_bytes = (width * 3) as usize;
    let mut packed = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * stride;
        packed.extend_from_slice(&plane.data[start..start + row_bytes]);
    }

    let buf = image::RgbImage::from_raw(width, height, packed)
        .context("failed to construct image from HEIC planes")?;

    Ok(image::DynamicImage::ImageRgb8(buf))
}

pub fn is_supported_input(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" | "tif" | "ico"
        | "heic" | "heif"
    )
}
