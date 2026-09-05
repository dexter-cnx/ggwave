mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub use api::*;
use flutter_rust_bridge::frb;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

#[cfg(target_os = "android")]
mod android_context {
    use std::{ffi::c_void, sync::OnceLock};

    use jni::{
        objects::{GlobalRef, JObject},
        JNIEnv,
    };

    static ANDROID_CONTEXT_REF: OnceLock<GlobalRef> = OnceLock::new();

    #[allow(non_snake_case)]
    #[no_mangle]
    pub extern "system" fn Java_com_dextercnx_ggwave_GgwaveRsFlutterPlugin_nativeInitializeAndroidContext(
        mut env: JNIEnv,
        _this: JObject,
        context: JObject,
    ) {
        if ANDROID_CONTEXT_REF.get().is_some() {
            return;
        }

        let global = env
            .new_global_ref(&context)
            .expect("failed to create Android Context global reference");
        let vm = env
            .get_java_vm()
            .expect("failed to obtain JavaVM for Android context");
        let vm_ptr = vm.get_java_vm_pointer() as *mut c_void;
        let context_ptr = global.as_obj().as_raw() as *mut c_void;

        unsafe {
            ndk_context::initialize_android_context(vm_ptr, context_ptr);
        }

        let _ = ANDROID_CONTEXT_REF.set(global);
    }
}
