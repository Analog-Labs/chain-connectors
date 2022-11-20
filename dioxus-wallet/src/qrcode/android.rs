use anyhow::Result;
use dioxus::prelude::*;
use futures::channel::oneshot;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jlong;
use jni::{JNIEnv, JavaVM};
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub fn scan_qrcode(_: &ScopeState) -> impl Future<Output = Result<String>> {
    async move {
        let context = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }?;
        let env = vm.attach_current_thread()?;
        let ctx = unsafe { JObject::from_raw(context.context().cast()) };
        let (tx, rx) = oneshot::channel();
        let tx = Box::into_raw(Box::new(tx));
        env.call_method(ctx, "scanQrCode", "(J)V", &[JValue::Long(tx as jlong)])?;
        Ok(rx.await?)
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_example_dioxus_1wallet_MainActivity_onQrCodeScanned(
    env: JNIEnv,
    _class: JClass,
    tx: Box<oneshot::Sender<String>>,
    url: JString,
) {
    if let Err(err) = catch_unwind(AssertUnwindSafe(|| {
        let url = env.get_string(url).unwrap();
        let url = url.to_str().unwrap();
        tx.send(url.into()).ok();
    })) {
        eprintln!(
            "attempt to unwind out of `onIntentActionView` with err: {:?}",
            err
        );
    }
}
