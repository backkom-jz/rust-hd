use std::io::Cursor;

use actix_multipart::Multipart;
use actix_web::{
    error::{ErrorBadRequest, ErrorPayloadTooLarge},
    HttpResponse,
};
use futures_util::StreamExt;
use image::ImageFormat;

const MAX_BYTES: usize = 20 * 1024 * 1024;

fn target_format(s: &str) -> Result<ImageFormat, &'static str> {
    match s.trim().to_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        _ => Err("不支持的输出格式，可选：png、jpeg、gif、bmp"),
    }
}

fn mime_for(fmt: ImageFormat) -> &'static str {
    match fmt {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Gif => "image/gif",
        ImageFormat::Bmp => "image/bmp",
        _ => "application/octet-stream",
    }
}

fn ext_for(fmt: ImageFormat) -> &'static str {
    match fmt {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        _ => "bin",
    }
}

fn safe_download_name(original: Option<&str>, fmt: ImageFormat) -> String {
    let stem = original
        .and_then(|n| {
            std::path::Path::new(n)
                .file_stem()
                .and_then(|s| s.to_str())
        })
        .filter(|s| {
            !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
        .unwrap_or("converted");
    format!("{stem}.{}", ext_for(fmt))
}

async fn read_field_limited(
    field: &mut actix_multipart::Field,
    limit: usize,
) -> actix_web::Result<Vec<u8>> {
    match field.bytes(limit).await {
        Ok(Ok(bytes)) => Ok(bytes.to_vec()),
        Ok(Err(e)) => Err(actix_web::error::ErrorInternalServerError(e)),
        // `Field::bytes` 仅在此返回 `LimitExceeded`（见 actix-multipart 文档）
        Err(_) => Err(ErrorPayloadTooLarge("文件超过 20MB 限制")),
    }
}

pub async fn convert_image(mut multipart: Multipart) -> actix_web::Result<HttpResponse> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut format_str: Option<String> = None;
    let mut original_name: Option<String> = None;

    while let Some(field) = multipart.next().await {
        let mut field = field.map_err(actix_web::error::ErrorInternalServerError)?;
        let field_name = field.name().map(str::to_owned);

        match field_name.as_deref() {
            Some("file") => {
                original_name = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename().map(String::from));
                let buf = read_field_limited(&mut field, MAX_BYTES).await?;
                if buf.is_empty() {
                    return Err(ErrorBadRequest("请上传图片文件"));
                }
                file_bytes = Some(buf);
            }
            Some("format") => {
                let buf = read_field_limited(&mut field, 256).await?;
                format_str = Some(String::from_utf8_lossy(&buf).into_owned());
            }
            _ => {
                let _ = read_field_limited(&mut field, MAX_BYTES).await?;
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| ErrorBadRequest("缺少上传文件"))?;
    let fmt_str = format_str
        .as_deref()
        .ok_or_else(|| ErrorBadRequest("请选择目标格式"))?;
    let out_fmt = target_format(fmt_str).map_err(ErrorBadRequest)?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| ErrorBadRequest(format!("无法解码图片: {e}")))?;

    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, out_fmt)
        .map_err(|e| ErrorBadRequest(format!("转换失败: {e}")))?;

    let body = out.into_inner();
    let filename = safe_download_name(original_name.as_deref(), out_fmt);

    Ok(HttpResponse::Ok()
        .content_type(mime_for(out_fmt))
        .insert_header((
            actix_web::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ))
        .body(body))
}
