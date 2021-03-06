use std::{
    ffi::{CStr, CString},
    intrinsics::copy_nonoverlapping,
    lazy::SyncLazy,
    sync::Mutex,
};

use libc::{c_char, c_double, c_int, c_void};

use crate::{
    util::{t6_date_to_epoch_timestamp, timestamp_to_datetime},
    Date, Var, T6,
};

#[no_mangle]
#[allow(unused_variables)]
extern "system" fn DllMain(dll_module: c_void, call_reason: c_void, reserved: c_void) -> bool {
    true
}

type FpType = extern "C" fn(*const c_char) -> c_int;

#[derive(Debug, Clone)]
pub(crate) struct State {
    pub(crate) handle: Option<extern "C" fn(*const c_char) -> c_int>,
    pub(crate) subscriptions: Vec<String>,
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

pub(crate) static STATE: SyncLazy<Mutex<State>> = SyncLazy::new(|| {
    Mutex::new(State {
        handle: None,
        subscriptions: vec![],
    })
});

pub(crate) fn log(msg: &str) {
    if let Ok(myref) = STATE.try_lock() {
        if let Some(func) = myref.handle {
            let cstr = CString::new(format!("{}\n", msg)).unwrap();
            func(cstr.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn BrokerCommand(command: c_int, data: c_int) -> Var {
    0.0 // Return 0 for all unimplemented commands
}

#[no_mangle]
pub extern "C" fn BrokerOpen(Name: *const c_char, fpError: FpType, fpProgress: FpType) -> c_int {
    2 // Return 2 if successful open
}

#[no_mangle]
pub extern "C" fn BrokerLogin(
    User: *const c_char,
    Pwd: *const c_char,
    Type: *const c_char,
    Accounts: *const c_char,
) -> c_int {
    0 // Return 1 for a successful login, 0 otherwise
}

#[no_mangle]
pub extern "C" fn BrokerAsset(
    Asset: *const c_char,
    pPrice: *mut c_double,
    pSpread: *mut c_double,
    pVolume: *mut c_double,
    pPip: *mut c_double,
    pPipCost: *mut c_double,
    pLotAmount: *mut c_double,
    pMarginCost: *mut c_double,
    pRollLong: *mut c_double,
    pRollShort: *mut c_double,
) -> c_int {
    unsafe {
        let asset_str = CStr::from_ptr(Asset).to_str();
        if asset_str.is_err() {
            log("Could not convert asset to a string");
            return 0;
        }

        if !STATE
            .lock()
            .unwrap()
            .subscriptions
            .contains(&asset_str.unwrap().to_owned())
        {
            // we haven't subscribed yet. Subscribe and return.
            STATE
                .lock()
                .unwrap()
                .subscriptions
                .push(asset_str.unwrap().to_owned());
            *pPrice = 0 as f64;
        }

        1
    }
}

#[no_mangle]
pub extern "C" fn BrokerHistory2(
    Asset: *const c_char,
    tStart: Date,
    tEnd: Date,
    nTickMinutes: c_int,
    nTicks: c_int,
    ticks: *mut T6,
) -> c_int {
    let start = timestamp_to_datetime(t6_date_to_epoch_timestamp(tStart as f64));
    let end = timestamp_to_datetime(t6_date_to_epoch_timestamp(tEnd));
    let asset_str = unsafe { CStr::from_ptr(Asset).to_str().unwrap() };
    // let t6_candles_ptr: *const T6 = t6_candles.as_ptr();
    // let t6_candles_mut_ptr: *mut T6 = t6_candles.clone().as_mut_ptr();
    // unsafe { copy_nonoverlapping(t6_candles_ptr, ticks, t6_candles.len()) }
    // t6_candles.len() as i32
    0 // Return number of ticks
}
