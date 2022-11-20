use anyhow::Result;
use core_media_sys::{
    kCMPixelFormat_24RGB, kCMPixelFormat_32BGRA, kCMPixelFormat_422YpCbCr8_yuvs,
    kCMPixelFormat_8IndexedGray_WhiteIsZero, kCMVideoCodecType_422YpCbCr8, CMSampleBufferRef,
    FourCharCode,
};
use dioxus::prelude::*;
use futures::channel::oneshot;
use image::{DynamicImage, GrayImage, ImageBuffer, Luma};
use objc::runtime::Object;
use objc::*;
use objc_foundation::NSString;
use objc_id::{Id, Owned};
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub fn scan_qrcode(cx: &ScopeState) -> impl Future<Output = Result<String>> {
    let (tx, rx) = oneshot::channel();
    let rscanner = QrcodeScanner::new(tx);
    let rscanner = Box::into_raw(Box::new(rscanner));
    let oscanner: *mut Object = unsafe { msg_send![class!(QrCodeScanner), alloc] };
    let on_image_buffer = on_image_buffer as *const ();
    let on_qrcode_scanned = on_qrcode_scanned as *const ();
    let _: () = unsafe {
        msg_send![
            oscanner,
            initWithScanner:rscanner
            onImageBuffer:on_image_buffer
            onQrcodeScanned:on_qrcode_scanned
        ]
    };
    let session: *mut Object = unsafe { msg_send![oscanner, session] };
    let view: *mut Object = unsafe { msg_send![class!(PreviewView), alloc] };
    let view: Id<Object, Owned> = unsafe { Id::from_retained_ptr(view) };
    let _: () = unsafe { msg_send![view, init] };
    let layer: *mut Object = unsafe { msg_send![view, previewLayer] };
    let _: () = unsafe { msg_send![layer, setVideoGravity: AVLayerVideoGravityResizeAspect] };
    let _: () = unsafe { msg_send![view, setSession: session] };
    let window = dioxus_desktop::use_window(cx);
    window.push_view(view.share());
    async move { Ok(rx.await?) }
}

unsafe extern "C" fn on_image_buffer(
    scanner: &mut QrcodeScanner,
    buffer: CMSampleBufferRef,
) -> u32 {
    let res = catch_unwind(AssertUnwindSafe(|| scanner.detect(buffer).unwrap()));
    match res {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(err) => {
            eprintln!("attempt to unwind out of `on_scanned` with err: {:?}", err);
            0
        }
    }
}

unsafe extern "C" fn on_qrcode_scanned(_scanner: Box<QrcodeScanner>) {}

pub struct QrcodeScanner {
    decoder: bardecoder::Decoder<DynamicImage, GrayImage, String>,
    tx: Option<oneshot::Sender<String>>,
}

impl QrcodeScanner {
    pub fn new(tx: oneshot::Sender<String>) -> Self {
        Self {
            decoder: bardecoder::default_decoder(),
            tx: Some(tx),
        }
    }

    pub fn detect(&mut self, buffer: CMSampleBufferRef) -> Result<bool> {
        let image = decode_cm_sample_buffer(buffer)?;
        let image = DynamicImage::ImageLuma8(image);
        for res in self.decoder.decode(&image) {
            if let Ok(qrcode) = res {
                if let Some(tx) = self.tx.take() {
                    tx.send(qrcode).ok();
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn decode_cm_sample_buffer(buffer: CMSampleBufferRef) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
    let image_buffer = unsafe { CMSampleBufferGetImageBuffer(buffer) };
    unsafe { CVPixelBufferLockBaseAddress(image_buffer, 0) };
    let (width, height, format, buffer) = unsafe {
        let width = CVPixelBufferGetWidth(image_buffer) as u32;
        let height = CVPixelBufferGetHeight(image_buffer) as u32;
        let format = CVPixelBufferGetPixelFormatType(image_buffer);
        let ptr = CVPixelBufferGetBaseAddress(image_buffer) as *mut u8;
        let length = CVPixelBufferGetDataSize(image_buffer) as usize;
        let buffer = std::slice::from_raw_parts_mut(ptr, length);
        (width, height, format, buffer)
    };
    let result = (|| {
        #[allow(non_upper_case_globals)]
        let format = match format {
            kCMVideoCodecType_422YpCbCr8 | kCMPixelFormat_422YpCbCr8_yuvs => FrameFormat::YUYV,
            kCMPixelFormat_8IndexedGray_WhiteIsZero => FrameFormat::GRAY,
            kCMPixelFormat_24RGB => FrameFormat::RGB,
            kCMPixelFormat_32BGRA => FrameFormat::BGRA,
            _ => anyhow::bail!("unsupported format {}", format),
        };
        decode_image(width, height, format, buffer)
    })();
    unsafe { CVPixelBufferUnlockBaseAddress(image_buffer, 0) };
    result
}

fn decode_image(
    width: u32,
    height: u32,
    format: FrameFormat,
    buffer: &[u8],
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
    let buffer = match format {
        FrameFormat::YUYV => {
            anyhow::ensure!(buffer.len() % 4 == 0);
            buffer
                .chunks_exact(4)
                .flat_map(|yuyv422| {
                    let yuyv444 = yuyv422_to_yuyv444(yuyv422.try_into().unwrap());
                    [
                        rgb_to_gray(yuyv444_to_rgb(yuyv444[0])),
                        rgb_to_gray(yuyv444_to_rgb(yuyv444[1])),
                    ]
                })
                .collect()
        }
        FrameFormat::RGB => {
            anyhow::ensure!(buffer.len() % 3 == 0);
            buffer
                .chunks_exact(3)
                .map(|rgb| rgb_to_gray(rgb.try_into().unwrap()))
                .collect()
        }
        FrameFormat::BGRA => {
            anyhow::ensure!(buffer.len() % 4 == 0);
            buffer
                .chunks_exact(4)
                .map(|bgra| bgra_to_gray(bgra.try_into().unwrap()))
                .collect()
        }
        FrameFormat::GRAY => buffer.to_vec(),
    };
    Ok(ImageBuffer::from_raw(width, height, buffer).unwrap())
}

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
/// - YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)
/// - MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
/// - GRAY is a grayscale image format, usually for specialized cameras such as IR Cameras.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum FrameFormat {
    YUYV,
    GRAY,
    RGB,
    BGRA,
}

#[inline]
fn yuyv422_to_yuyv444(yuyv: [u8; 4]) -> [[u8; 3]; 2] {
    [[yuyv[0], yuyv[1], yuyv[3]], [yuyv[2], yuyv[1], yuyv[3]]]
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGB888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
#[inline]
fn yuyv444_to_rgb(yuv: [u8; 3]) -> [u8; 3] {
    let c298 = (yuv[0] as i32 - 16) * 298;
    let d = yuv[1] as i32 - 128;
    let e = yuv[2] as i32 - 128;
    let r = ((c298 + 409 * e + 128) >> 8) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8) as u8;
    let b = ((c298 + 516 * d + 128) >> 8) as u8;
    [r, g, b]
}

#[inline]
fn rgb_to_gray(rgb: [u8; 3]) -> u8 {
    ((rgb[0] as u16 + rgb[1] as u16 + rgb[2] as u16) / 3) as u8
}

#[inline]
fn bgra_to_gray(bgra: [u8; 4]) -> u8 {
    rgb_to_gray([bgra[2], bgra[1], bgra[0]])
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct __CVBuffer {
    _unused: [u8; 0],
}

type CVBufferRef = *mut __CVBuffer;
type CVImageBufferRef = CVBufferRef;
type CVPixelBufferRef = CVImageBufferRef;
type CVPixelBufferLockFlags = u64;
type CVReturn = i32;

#[allow(non_snake_case)]
#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    fn CMSampleBufferGetImageBuffer(sbuf: CMSampleBufferRef) -> CVImageBufferRef;

    fn CVPixelBufferLockBaseAddress(
        pixelBuffer: CVPixelBufferRef,
        lockFlags: CVPixelBufferLockFlags,
    ) -> CVReturn;

    fn CVPixelBufferUnlockBaseAddress(
        pixelBuffer: CVPixelBufferRef,
        unlockFlags: CVPixelBufferLockFlags,
    ) -> CVReturn;

    fn CVPixelBufferGetWidth(pixelBuffer: CVPixelBufferRef) -> isize;

    fn CVPixelBufferGetHeight(pixelBuffer: CVPixelBufferRef) -> isize;

    fn CVPixelBufferGetDataSize(pixelBuffer: CVPixelBufferRef) -> std::os::raw::c_ulong;

    fn CVPixelBufferGetBaseAddress(pixelBuffer: CVPixelBufferRef) -> *mut std::os::raw::c_void;

    fn CVPixelBufferGetPixelFormatType(pixelBuffer: CVPixelBufferRef) -> FourCharCode;
}

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    static AVLayerVideoGravityResizeAspect: &'static NSString;
}
