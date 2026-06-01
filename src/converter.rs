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
    pub input: PathBuf,
    pub output: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

pub fn convert(job: &ConvertJob) -> ConvertResult {
    match do_convert(job) {
        Ok(output) => ConvertResult {
            input: job.input.clone(),
            output,
            success: true,
            error: None,
        },
        Err(e) => ConvertResult {
            input: job.input.clone(),
            output: PathBuf::new(),
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

    let img = image::open(&job.input).context("failed to open image")?;
    img.save_with_format(&output_path, job.format.as_image_format())
        .context("failed to save image")?;

    Ok(output_path)
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
    )
}
