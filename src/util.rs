use anyhow::anyhow;
use exif::Tag;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::ops::Deref;
use std::sync::LazyLock;
use std::thread;
// Using image crate: https://github.com/image-rs/image
use std::{fs, path::PathBuf};
use webp::{Encoder, WebPMemory}; // Using webp crate: https://github.com/jaredforth/webp

use std::fs::File;
use std::io::{BufReader, Cursor, Seek, SeekFrom, Write};
use std::path::Path;
use tracing::warn;

use crate::auth::handlers::upload::{FileAttachment, UploadMultipartItemType};

const NON_WEBP_EXTS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["jpg", "jpeg", "png"]);
const ENCODING_QUALITY: f32 = 65f32;

#[tracing::instrument(name = "getting all images in directory path")]
pub fn all_images_in_directory(path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = vec![];
    // let image_exts = vec!["jpg", "jpeg", "png", "JPG", "webp"];
    let path = PathBuf::from(path);
    let entries = fs::read_dir(path).map_err(|err| {
        warn!("error with entries: {:?}", err);
        err
    })?;
    for file in entries {
        let path = file
            .map_err(|err| {
                warn!("error with entry: {:?}", err);
                err
            })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str().unwrap() == "webp" {
                    images.push(path)
                }
            }
        }
    }
    warn!("returning images: {:?}", images);
    Ok(images)
}

#[tracing::instrument(name = "bytes to webp", skip(bytes))]
pub fn bytes_to_webp(bytes: &[u8], ext: &str) -> anyhow::Result<WebPMemory> {
    let already_webp = ext.to_lowercase().as_str() == "webp";

    let mut cursor = Cursor::new(bytes);
    let mut orientation_fn = None;
    if !already_webp {
        warn!("file is not already webp");
        let exif_reader = exif::Reader::new();
        let exif_ = exif_reader.read_from_container(&mut cursor)?;
        if let Some(tag) = exif_.get_field(Tag::Orientation, exif::In::PRIMARY) {
            if let Some(codes) = tag
                .value
                .iter_uint()
                .and_then(|ints| Some(ints.into_iter().collect()))
            {
                orientation_fn = Some(orientation_codes_to_fn(codes))
            }
        }
        cursor.rewind().unwrap();
    }

    let reader = ImageReader::new(cursor).with_guessed_format()?;
    warn!("created image reader");
    let mut image = reader
        .decode()
        .map_err(|err| warn!("problem decoding image: {err:?}"))
        .expect("failed to decode image");
    warn!("read image");
    if let Some(mut f) = orientation_fn {
        warn!("correcting orientation");
        image = f(image);
    }

    warn!("creating encoder");
    let encoder: Encoder = Encoder::from_image(&image)
        .map_err(|err| warn!("problem creating encoder: {err:?}"))
        .expect("failed to encode image");

    warn!("encoding image");
    let encoded_webp: WebPMemory =
        encoder
            .encode_simple(false, ENCODING_QUALITY)
            .map_err(|err| {
                let m = format!("error encoding image: {err:?}");
                warn!(m);
                anyhow!(m)
            })?;
    warn!("returning webp encoded memory");
    Ok(encoded_webp)
}

fn remove_all_non_webp_imgs_in_directory(path: &str) -> anyhow::Result<usize> {
    let path = PathBuf::from(path);
    let entries = fs::read_dir(path).map_err(|err| {
        warn!("error with entries: {:?}", err);
        err
    })?;
    let mut count = 0;
    for file in entries {
        let path = file
            .map_err(|err| {
                warn!("error with entry: {:?}", err);
                err
            })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if LazyLock::force(&NON_WEBP_EXTS)
                    .contains(&ext.to_str().unwrap().to_lowercase().as_str())
                {
                    fs::remove_file(path)?;
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn convert_all_files_in_directory(path: &str) -> anyhow::Result<Vec<String>> {
    let all_non_webp = all_non_webp_in_directory(path)?;
    let mut return_paths = vec![];

    if all_non_webp.is_empty() {
        warn!("all files in directory are webp");
        return Ok(return_paths);
    }

    let mut handles = vec![];
    for path in all_non_webp {
        let h = thread::spawn(move || image_to_webp(path).expect("failed to convert img to webp"));
        handles.push(h);
    }

    for h in handles {
        return_paths.push(h.join().map_err(|e| anyhow!("handle join error: {e:?}"))?);
    }

    Ok(return_paths)
}

fn all_non_webp_in_directory(path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = vec![];
    let path = PathBuf::from(path);
    let entries = fs::read_dir(path).map_err(|err| {
        warn!("error with entries: {:?}", err);
        err
    })?;
    for file in entries {
        let path = file
            .map_err(|err| {
                warn!("error with entry: {:?}", err);
                err
            })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if LazyLock::force(&NON_WEBP_EXTS)
                    .contains(&ext.to_str().unwrap().to_lowercase().as_str())
                {
                    images.push(path)
                }
            }
        }
    }
    warn!("returning images: {:?}", images);
    Ok(images)
}

fn orientation_codes_to_fn(codes: Vec<u32>) -> impl FnMut(DynamicImage) -> DynamicImage {
    move |mut image: DynamicImage| {
        for code in &codes {
            match code {
                1 => {
                    warn!("Normal orientation");
                    image = image;
                }
                2 => {
                    warn!("Horizontal flip");
                    image = image.fliph();
                }
                3 => {
                    warn!("Upside down");
                    image = image.rotate180();
                }
                4 => {
                    warn!("Vertical flip");
                    image = image.flipv();
                }
                5 => {
                    warn!("90 degrees clockwise + horizontal flip");
                    image = image.fliph().rotate90();
                }
                6 => {
                    warn!("90 degrees clockwise");
                    image = image.rotate90();
                }
                7 => {
                    warn!("90 degrees counterclockwise + horizontal flip");
                    image = image.fliph().rotate270();
                }
                8 => {
                    warn!("90 degrees counterclockwise");
                    image = image.rotate270();
                }
                o => {
                    warn!("{o:?} has no rotation");
                    image = image;
                }
            };
        }
        image
    }
}

fn image_to_webp(file_path: PathBuf) -> anyhow::Result<String> {
    let parent_directory: &Path = file_path.parent().unwrap();
    let webp_folder_path = format!("{}", parent_directory.to_str().unwrap());
    std::fs::create_dir_all(webp_folder_path.to_string())?;

    let filename_original_image = file_path.file_stem().unwrap().to_str().unwrap();

    let webp_image_path = format!(
        "{}/{}.webp",
        webp_folder_path.to_string(),
        filename_original_image
    );

    let file = File::open(file_path.clone())?;
    let mut bufreader = BufReader::new(&file);
    let mut orientation_fn = None;
    let exif_reader = exif::Reader::new();
    let exif_ = exif_reader.read_from_container(&mut bufreader)?;
    if let Some(tag) = exif_.get_field(Tag::Orientation, exif::In::PRIMARY) {
        if let Some(codes) = tag
            .value
            .iter_uint()
            .and_then(|ints| Some(ints.into_iter().collect()))
        {
            orientation_fn = Some(orientation_codes_to_fn(codes))
        }
    }

    let mut image: DynamicImage = match image::ImageReader::open(&file_path) {
        Ok(img) => img.with_guessed_format().unwrap().decode().unwrap(),
        Err(e) => {
            warn!("Error: {}", e);
            return Err(anyhow!("error opening file as an image: {e:?}"));
        }
    };

    if let Some(mut f) = orientation_fn {
        warn!("rotating image: {filename_original_image:?}");
        image = f(image);
    }

    let encoder: Encoder = Encoder::from_image(&image).unwrap();
    let encoded_webp: WebPMemory = encoder.encode(ENCODING_QUALITY);

    let mut webp_image_file = File::create(webp_image_path.to_string()).unwrap();

    webp_image_file
        .write_all(encoded_webp.deref())
        .map_err(|e| anyhow!("error writing encoded image: {e:?}"))?;

    Ok(webp_image_path)
}

mod tests {
    use std::path::PathBuf;

    use crate::util::{convert_all_files_in_directory, image_to_webp};

    use super::remove_all_non_webp_imgs_in_directory;

    // #[test]
    fn image_to_webp_works() {
        let path = "public/assets/images/board_members/business.jpg";
        let expected = "public/assets/images/board_members/business.webp".to_string();

        assert_eq!(image_to_webp(PathBuf::from(path)).unwrap(), expected)
    }

    // #[test]
    fn do_it() {
        let dir_path = "public/assets/images/landing_page";
        convert_all_files_in_directory(dir_path).unwrap();
        remove_all_non_webp_imgs_in_directory(&dir_path).unwrap();

        // let dir_path = "public/assets/images/merchandise/misc";
        // convert_all_files_in_directory(dir_path).unwrap();
        // remove_all_non_webp_imgs_in_directory(&dir_path).unwrap();
        //
        // let dir_path = "public/assets/images/merchandise/tops";
        // convert_all_files_in_directory(dir_path).unwrap();
        // remove_all_non_webp_imgs_in_directory(&dir_path).unwrap();
        //
        // let dir_path = "public/assets/images/patrol_log/fishing_trip";
        // convert_all_files_in_directory(dir_path).unwrap();
        // remove_all_non_webp_imgs_in_directory(&dir_path).unwrap();
    }

    // #[test]
    fn all_imgs_in_in_dir_to_webp_and_deletion_works() {
        let dir_path = "public/assets/images/board_members";
        let expected_names = vec![
            "beverly.webp",
            "business.webp",
            "business2.webp",
            "dan.webp",
            "jamie.webp",
            "old.webp",
        ];

        let expected_paths: Vec<String> = expected_names
            .into_iter()
            .map(|p| format!("{dir_path}/{p}"))
            .collect();

        for p in convert_all_files_in_directory(dir_path).unwrap() {
            assert!(expected_paths.contains(&p));
        }
        assert_eq!(
            remove_all_non_webp_imgs_in_directory(&dir_path).unwrap(),
            expected_paths.len()
        );
    }
}
