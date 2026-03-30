use anyhow::Result;
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;

/// 内嵌静态 HTML 页面
const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Serialize)]
struct SplitResponse {
    success: bool,
    message: String,
    files: Vec<FileInfo>,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    /// Base64 编码的 PDF 数据
    data: String,
    page_count: u32,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct InfoResponse {
    success: bool,
    total_pages: u32,
    file_name: String,
}

/// 启动 Axum Web 服务器
pub async fn start_server(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/info", post(info_handler))
        .route("/api/split", post(split_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 返回首页 HTML
async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// 获取 PDF 文件信息（总页数）
async fn info_handler(mut multipart: Multipart) -> Response {
    let mut pdf_data: Option<(String, Vec<u8>)> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            let file_name = field
                .file_name()
                .unwrap_or("unknown.pdf")
                .to_string();
            match field.bytes().await {
                Ok(data) => {
                    pdf_data = Some((file_name, data.to_vec()));
                }
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            success: false,
                            message: format!("读取文件失败: {}", e),
                        }),
                    )
                        .into_response();
                }
            }
        }
    }

    let (file_name, data) = match pdf_data {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: "未收到 PDF 文件".to_string(),
                }),
            )
                .into_response();
        }
    };

    match crate::get_page_count(&data) {
        Ok(total_pages) => Json(InfoResponse {
            success: true,
            total_pages,
            file_name,
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: format!("无法解析 PDF: {}", e),
            }),
        )
            .into_response(),
    }
}

/// 处理 PDF 分割请求
async fn split_handler(mut multipart: Multipart) -> Response {
    let mut pdf_data: Option<(String, Vec<u8>)> = None;
    let mut mode = String::new();
    let mut pages_str = String::new();
    let mut split_every: usize = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let file_name = field
                    .file_name()
                    .unwrap_or("unknown.pdf")
                    .to_string();
                match field.bytes().await {
                    Ok(data) => {
                        pdf_data = Some((file_name, data.to_vec()));
                    }
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                success: false,
                                message: format!("读取文件失败: {}", e),
                            }),
                        )
                            .into_response();
                    }
                }
            }
            "mode" => {
                if let Ok(bytes) = field.bytes().await {
                    mode = String::from_utf8_lossy(&bytes).to_string();
                }
            }
            "pages" => {
                if let Ok(bytes) = field.bytes().await {
                    pages_str = String::from_utf8_lossy(&bytes).to_string();
                }
            }
            "split_every" => {
                if let Ok(bytes) = field.bytes().await {
                    let s = String::from_utf8_lossy(&bytes);
                    split_every = s.parse().unwrap_or(0);
                }
            }
            _ => {}
        }
    }

    let (file_name, data) = match pdf_data {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: "未收到 PDF 文件".to_string(),
                }),
            )
                .into_response();
        }
    };

    let stem = file_name
        .strip_suffix(".pdf")
        .or_else(|| file_name.strip_suffix(".PDF"))
        .unwrap_or(&file_name);

    match mode.as_str() {
        "pages" => {
            // 按页码范围提取
            let total = match crate::get_page_count(&data) {
                Ok(t) => t,
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            success: false,
                            message: format!("无法解析 PDF: {}", e),
                        }),
                    )
                        .into_response();
                }
            };

            let pages = match crate::parse_page_ranges(&pages_str, total) {
                Ok(p) => p,
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            success: false,
                            message: format!("页码解析错误: {}", e),
                        }),
                    )
                        .into_response();
                }
            };

            match crate::extract_pages(&data, &pages) {
                Ok(result) => {
                    let page_count = crate::get_page_count(&result).unwrap_or(0);
                    Json(SplitResponse {
                        success: true,
                        message: format!("成功提取 {} 页", pages.len()),
                        files: vec![FileInfo {
                            name: format!("{}_pages_{}.pdf", stem, pages_str.replace(',', "_")),
                            data: base64_encode(&result),
                            page_count,
                        }],
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        message: format!("分割失败: {}", e),
                    }),
                )
                    .into_response(),
            }
        }
        "split" => {
            // 按页数分割
            if split_every == 0 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        success: false,
                        message: "每份页数必须大于 0".to_string(),
                    }),
                )
                    .into_response();
            }

            match crate::split_by_count(&data, split_every) {
                Ok(chunks) => {
                    let files: Vec<FileInfo> = chunks
                        .iter()
                        .enumerate()
                        .map(|(i, chunk)| {
                            let page_count = crate::get_page_count(chunk).unwrap_or(0);
                            FileInfo {
                                name: format!("{}_part_{}.pdf", stem, i + 1),
                                data: base64_encode(chunk),
                                page_count,
                            }
                        })
                        .collect();

                    let count = files.len();
                    Json(SplitResponse {
                        success: true,
                        message: format!("成功分割为 {} 份", count),
                        files,
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        message: format!("分割失败: {}", e),
                    }),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: format!("未知分割模式: '{}'，请使用 'pages' 或 'split'", mode),
            }),
        )
            .into_response(),
    }
}

/// 简单的 Base64 编码（使用标准字符集）
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
