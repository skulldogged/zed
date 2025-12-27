//! Video frame types for cross-platform video rendering.
//!
//! This module provides a platform-agnostic video frame type that can be used
//! to render video content efficiently across different operating systems.

use std::sync::Arc;

/// A video frame that can be painted to the screen.
///
/// This type abstracts over platform-specific video buffer types,
/// allowing efficient video rendering on all supported platforms.
#[derive(Clone)]
pub struct VideoFrame {
    pub(crate) data: VideoFrameData,
    /// The width of the video frame in pixels.
    pub width: u32,
    /// The height of the video frame in pixels.
    pub height: u32,
}

/// The inner data of a video frame.
#[derive(Clone)]
pub(crate) enum VideoFrameData {
    /// A CPU buffer in BGRA format.
    /// This is the fallback format that works on all platforms.
    Bgra(Arc<Vec<u8>>),

    /// A macOS CoreVideo pixel buffer (zero-copy path).
    #[cfg(target_os = "macos")]
    CoreVideo(core_video::pixel_buffer::CVPixelBuffer),

    /// A Windows D3D11 texture (zero-copy path).
    #[cfg(target_os = "windows")]
    D3D11 {
        texture: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        subresource_index: u32,
    },
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field(
                "data",
                &match &self.data {
                    VideoFrameData::Bgra(_) => "Bgra",
                    #[cfg(target_os = "macos")]
                    VideoFrameData::CoreVideo(_) => "CoreVideo",
                    #[cfg(target_os = "windows")]
                    VideoFrameData::D3D11 { .. } => "D3D11",
                },
            )
            .finish()
    }
}

impl VideoFrame {
    /// Create a video frame from raw BGRA pixel data.
    ///
    /// The buffer should contain `width * height * 4` bytes in BGRA format.
    pub fn from_bgra(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        debug_assert_eq!(
            buffer.len(),
            (width * height * 4) as usize,
            "BGRA buffer size mismatch"
        );
        Self {
            data: VideoFrameData::Bgra(Arc::new(buffer)),
            width,
            height,
        }
    }

    /// Create a video frame from an existing Arc'd BGRA buffer.
    ///
    /// This avoids an extra copy when the buffer is already reference-counted.
    pub fn from_bgra_arc(buffer: Arc<Vec<u8>>, width: u32, height: u32) -> Self {
        debug_assert_eq!(
            buffer.len(),
            (width * height * 4) as usize,
            "BGRA buffer size mismatch"
        );
        Self {
            data: VideoFrameData::Bgra(buffer),
            width,
            height,
        }
    }

    /// Create a video frame from a macOS CoreVideo pixel buffer.
    ///
    /// This provides a zero-copy path on macOS.
    #[cfg(target_os = "macos")]
    pub fn from_cv_pixel_buffer(buffer: core_video::pixel_buffer::CVPixelBuffer) -> Self {
        let width = buffer.get_width() as u32;
        let height = buffer.get_height() as u32;
        Self {
            data: VideoFrameData::CoreVideo(buffer),
            width,
            height,
        }
    }

    /// Create a video frame from a Windows D3D11 texture.
    ///
    /// This provides a zero-copy path on Windows when using hardware-accelerated
    /// video decoding with Media Foundation.
    ///
    /// # Arguments
    /// * `texture` - The D3D11 texture containing the decoded video frame
    /// * `subresource_index` - The subresource index within the texture array (usually 0)
    /// * `width` - The width of the video frame
    /// * `height` - The height of the video frame
    #[cfg(target_os = "windows")]
    pub fn from_d3d11_texture(
        texture: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        subresource_index: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            data: VideoFrameData::D3D11 {
                texture,
                subresource_index,
            },
            width,
            height,
        }
    }

    /// Get the size of this video frame in pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get access to the raw pixel data, if this is a CPU-backed frame.
    ///
    /// Returns `None` for hardware-backed frames (e.g., CoreVideo on macOS, D3D11 on Windows).
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.data {
            VideoFrameData::Bgra(buffer) => Some(buffer.as_slice()),
            #[cfg(target_os = "macos")]
            VideoFrameData::CoreVideo(_) => None,
            #[cfg(target_os = "windows")]
            VideoFrameData::D3D11 { .. } => None,
        }
    }
}
