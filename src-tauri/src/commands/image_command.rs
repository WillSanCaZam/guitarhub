use base64::Engine;
use tauri::State;

use crate::services::image_cache::ImageCacheService;

/// Tauri IPC command: fetch a product image via the local cache.
///
/// Returns a `data:<mime>;base64,...` string that can be used directly
/// as an `<img src>` attribute.
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: State<'_, ImageCacheService>,
) -> Result<String, String> {
    let (bytes, mime) = state
        .get(&image_url)
        .await
        .map_err(|e| format!("Image load failed: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
