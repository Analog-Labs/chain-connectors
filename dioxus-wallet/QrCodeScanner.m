@import AVFoundation;
@import UIKit;

@interface PreviewView: UIView
@property(nonatomic, readonly, strong) AVCaptureVideoPreviewLayer* previewLayer;
@property(nonatomic, retain, nullable) AVCaptureSession* session;
@end

@implementation PreviewView
+(Class)layerClass {
    return AVCaptureVideoPreviewLayer.class;
}

-(AVCaptureVideoPreviewLayer*)previewLayer {
    return (AVCaptureVideoPreviewLayer*)super.layer;
}

-(AVCaptureSession*)session {
    return self.previewLayer.session;
}

-(void)setSession:(AVCaptureSession*)newValue {
    self.previewLayer.session = newValue;
}
@end

@interface QrCodeScanner: NSObject <AVCaptureVideoDataOutputSampleBufferDelegate> {
    AVCaptureSession* captureSession;
    AVCaptureDevice* device;
    AVCaptureVideoDataOutput* videoOutput;
    uintptr_t scanner;
    uint32_t(*onImageBuffer)(uintptr_t, CMSampleBufferRef);
    void(*onQrcodeScanned)(uintptr_t);
}
-(AVCaptureSession*)session;
-(void)captureOutput:(AVCaptureOutput*)output
didOutputSampleBuffer:(CMSampleBufferRef)sampleBuffer
fromConnection:(AVCaptureConnection*)connection;
@end

@implementation QrCodeScanner

-(id)initWithScanner:(uintptr_t)scanner2
    onImageBuffer:(uint32_t(*)(uintptr_t, CMSampleBufferRef))onImageBuffer2
    onQrcodeScanned:(void(*)(uintptr_t))onQrcodeScanned2
{
    self = [super init];
    scanner = scanner2;
    onImageBuffer = onImageBuffer2;
    onQrcodeScanned = onQrcodeScanned2;

    captureSession = [[AVCaptureSession alloc] init];
    captureSession.sessionPreset = AVCaptureSessionPresetMedium;

    device = [AVCaptureDevice defaultDeviceWithMediaType: AVMediaTypeVideo];
    NSError* outError = nil;
    AVCaptureDeviceInput *cameraInput = [AVCaptureDeviceInput deviceInputWithDevice: device error:&outError];
    if (outError != nil) {
        @throw outError;
    }
    [captureSession addInput: cameraInput];

    videoOutput = [[AVCaptureVideoDataOutput alloc] init];
    NSString *key = (NSString*)kCVPixelBufferPixelFormatTypeKey;
    NSNumber *value = [NSNumber numberWithInt: kCVPixelFormatType_32BGRA];
    videoOutput.videoSettings = @{key : value};
    dispatch_queue_t queue = dispatch_queue_create("dioxus-wallet.qrcodescanner", DISPATCH_QUEUE_SERIAL);
    [videoOutput setSampleBufferDelegate:self queue: queue];
    [videoOutput setAlwaysDiscardsLateVideoFrames: YES];
    [captureSession addOutput: videoOutput];

    [captureSession startRunning];

    return self;
}

-(AVCaptureSession*)session {
    return captureSession;
}

-(void)captureOutput:(AVCaptureOutput*)output
didOutputSampleBuffer:(CMSampleBufferRef)sampleBuffer
fromConnection:(AVCaptureConnection*)connection
{
    if (onImageBuffer(scanner, sampleBuffer) > 0) {
        [captureSession stopRunning];
        onQrcodeScanned(scanner);
    }
}
@end
