//! Android activity callbacks. Only owned data and atomics cross the UI/game threads.
use super::motion::MotionInput;
use ironjump::input::GameInput;
use macroquad::miniquad::native::android::ndk_sys::{
    JNIEnv, jboolean, jclass, jfloat, jint, jlong, jstring,
};
use std::{
    path::PathBuf,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

// Sensor callbacks and the render loop run on different threads. Keep each sample
// and its arrival time together; never hold the lock while calling Java or drawing.
#[derive(Default)]
struct SensorState {
    motion: MotionInput,
    last_arrival: Option<(Instant, f64)>,
}
static SENSOR: OnceLock<Mutex<SensorState>> = OnceLock::new();
fn sensor() -> &'static Mutex<SensorState> {
    SENSOR.get_or_init(|| Mutex::new(SensorState::default()))
}

pub fn motion_input() -> GameInput {
    let state = sensor().lock().unwrap_or_else(|error| error.into_inner());
    match state.last_arrival {
        Some((arrival, timestamp)) => state
            .motion
            .input(timestamp + arrival.elapsed().as_secs_f64()),
        None => GameInput::NONE,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_ironjump_game_MainActivity_ironjumpAcceleration(
    _env: *mut JNIEnv,
    _class: jclass,
    x: jfloat,
    y: jfloat,
    rotation: jint,
    timestamp: jlong,
) {
    if !resumed()
        || timestamp < 0
        || !x.is_finite()
        || !y.is_finite()
        || !(0..=3).contains(&rotation)
    {
        return;
    }
    let mut state = sensor().lock().unwrap_or_else(|error| error.into_inner());
    let time = timestamp as f64 * 1e-9;
    if state
        .last_arrival
        .is_some_and(|(_, previous)| time <= previous)
    {
        return;
    }
    state.motion.sample(x, y, rotation, time);
    state.last_arrival = Some((Instant::now(), time));
}

static FILES_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static TRACE_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static RESUMED: AtomicBool = AtomicBool::new(false);
static RESET_CLOCK: AtomicBool = AtomicBool::new(true);
static BACK: AtomicBool = AtomicBool::new(false);

pub fn progress_path() -> Result<PathBuf, String> {
    FILES_DIRECTORY
        .get()
        .map(|path| path.join("ironjump-progress.txt"))
        .ok_or_else(|| "Android activity did not supply its files directory".to_owned())
}

#[cfg(feature = "frame-profile")]
pub fn trace_path() -> Result<PathBuf, String> {
    TRACE_DIRECTORY
        .get()
        .map(|path| path.join("ironjump-trace.txt"))
        .ok_or_else(|| "Android activity did not supply its trace directory".to_owned())
}

pub fn resumed() -> bool {
    RESUMED.load(Ordering::Acquire)
}

pub fn take_clock_reset() -> bool {
    RESET_CLOCK.swap(false, Ordering::AcqRel)
}

pub fn take_back() -> bool {
    BACK.swap(false, Ordering::AcqRel)
}

// Called by MainActivity on the JVM thread. Copy the Java string before returning;
// neither the JNI environment nor local references may escape this callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_org_ironjump_game_MainActivity_ironjumpInit(
    env: *mut JNIEnv,
    _class: jclass,
    directory: jstring,
    trace_directory: jstring,
) {
    unsafe {
        if let Some(path) = copy_path(env, directory) {
            let _ = FILES_DIRECTORY.set(path);
        }
        if let Some(path) = copy_path(env, trace_directory) {
            let _ = TRACE_DIRECTORY.set(path);
        }
    }
}

unsafe fn copy_path(env: *mut JNIEnv, directory: jstring) -> Option<PathBuf> {
    if directory.is_null() {
        return None;
    }
    unsafe {
        let length = ((**env).GetStringLength.unwrap())(env, directory);
        let chars = ((**env).GetStringChars.unwrap())(env, directory, std::ptr::null_mut());
        if chars.is_null() {
            return None;
        }
        let path = String::from_utf16_lossy(std::slice::from_raw_parts(chars, length as usize));
        ((**env).ReleaseStringChars.unwrap())(env, directory, chars);
        Some(PathBuf::from(path))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_ironjump_game_MainActivity_ironjumpLifecycle(
    _env: *mut JNIEnv,
    _class: jclass,
    resumed: jboolean,
) {
    *sensor().lock().unwrap_or_else(|error| error.into_inner()) = SensorState::default();
    RESET_CLOCK.store(true, Ordering::Release);
    RESUMED.store(resumed != 0, Ordering::Release);
    if resumed == 0 {
        BACK.store(false, Ordering::Release);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_ironjump_game_MainActivity_ironjumpBack(
    _env: *mut JNIEnv,
    _class: jclass,
) {
    BACK.store(true, Ordering::Release);
}
