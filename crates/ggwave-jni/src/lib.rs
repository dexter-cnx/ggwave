use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use ggwave_core::{Codec, Protocol, Tuning};
use jni::objects::{JByteArray, JClass, JFloatArray};
use jni::sys::{jboolean, jfloatArray, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use once_cell::sync::Lazy;

enum Command {
    SetFrequency(f32, Sender<Result<(), String>>),
    Encode(Vec<u8>, i32, i32, Sender<Result<Vec<f32>, String>>),
    Decode(Vec<f32>, Sender<Result<Option<Vec<u8>>, String>>),
}

static ENGINE: Lazy<Sender<Command>> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("ggwave-jni".into())
        .spawn(move || worker(rx))
        .expect("spawn ggwave JNI worker");
    tx
});

fn worker(rx: Receiver<Command>) {
    let mut tuning = Tuning::default();
    let mut codec = Codec::new(tuning).expect("initialize ggwave codec");
    while let Ok(command) = rx.recv() {
        match command {
            Command::SetFrequency(hz, reply) => {
                tuning.ultrasonic_hz = hz;
                let result = Codec::new(tuning)
                    .map(|next| codec = next)
                    .map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
            Command::Encode(data, protocol_id, volume, reply) => {
                let result = codec
                    .encode(&data, Protocol::from_app_id(protocol_id), volume)
                    .map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
            Command::Decode(samples, reply) => {
                let result = codec.decode(&samples).map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
        }
    }
}

fn throw(env: &mut JNIEnv, message: impl AsRef<str>) {
    let _ = env.throw_new("java/lang/IllegalStateException", message.as_ref());
}

#[no_mangle]
pub extern "system" fn Java_io_github_dextercnx_ggwave_GgWave_nativeSetUltrasonicFrequency(
    mut env: JNIEnv,
    _class: JClass,
    hz: f32,
) -> jboolean {
    let (tx, rx) = mpsc::channel();
    if ENGINE.send(Command::SetFrequency(hz, tx)).is_err() {
        throw(&mut env, "ggwave worker unavailable");
        return JNI_FALSE;
    }
    match rx.recv().unwrap_or_else(|_| Err("ggwave worker stopped".into())) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            throw(&mut env, error);
            JNI_FALSE
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_dextercnx_ggwave_GgWave_nativeEncode(
    mut env: JNIEnv,
    _class: JClass,
    data: JByteArray,
    protocol_id: i32,
    volume: i32,
) -> jfloatArray {
    let payload = match env.convert_byte_array(&data) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };
    let (tx, rx) = mpsc::channel();
    if ENGINE.send(Command::Encode(payload, protocol_id, volume, tx)).is_err() {
        throw(&mut env, "ggwave worker unavailable");
        return std::ptr::null_mut();
    }
    match rx.recv().unwrap_or_else(|_| Err("ggwave worker stopped".into())) {
        Ok(samples) => match env.new_float_array(samples.len() as i32) {
            Ok(array) => {
                if let Err(error) = env.set_float_array_region(&array, 0, &samples) {
                    throw(&mut env, error.to_string());
                    std::ptr::null_mut()
                } else {
                    array.into_raw()
                }
            }
            Err(error) => {
                throw(&mut env, error.to_string());
                std::ptr::null_mut()
            }
        },
        Err(error) => {
            throw(&mut env, error);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_dextercnx_ggwave_GgWave_nativeDecode(
    mut env: JNIEnv,
    _class: JClass,
    samples: JFloatArray,
) -> jni::sys::jbyteArray {
    let length = match env.get_array_length(&samples) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };
    let mut input = vec![0.0f32; length as usize];
    if let Err(error) = env.get_float_array_region(&samples, 0, &mut input) {
        throw(&mut env, error.to_string());
        return std::ptr::null_mut();
    }
    let (tx, rx) = mpsc::channel();
    if ENGINE.send(Command::Decode(input, tx)).is_err() {
        throw(&mut env, "ggwave worker unavailable");
        return std::ptr::null_mut();
    }
    match rx.recv().unwrap_or_else(|_| Err("ggwave worker stopped".into())) {
        Ok(Some(payload)) => env.byte_array_from_slice(&payload).map_or_else(
            |error| {
                throw(&mut env, error.to_string());
                std::ptr::null_mut()
            },
            |array| array.into_raw(),
        ),
        Ok(None) => std::ptr::null_mut(),
        Err(error) => {
            throw(&mut env, error);
            std::ptr::null_mut()
        }
    }
}
