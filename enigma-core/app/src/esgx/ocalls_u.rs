use db::dal::CRUDInterface;
use db::{DeltaKey, Stype, DATABASE};
use esgx::general;
use std::ptr;
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn ocall_get_home(output: *mut u8, result_len: &mut usize) {
    let path = general::storage_dir();
    let path_str = path.to_str().unwrap();
    ptr::copy_nonoverlapping(path_str.as_ptr(), output, path_str.len());
    *result_len = path_str.len();
}

#[no_mangle]
pub unsafe extern "C" fn ocall_update_state(id: &[u8; 32], enc_state: *const u8, state_len: usize) -> i8 {
    let encrypted_state = slice::from_raw_parts(enc_state, state_len);

    let key = DeltaKey::new(*id, Stype::State);

    match DATABASE.lock().expect("Database mutex is poison").force_update(&key, encrypted_state) {
        Ok(_) => (), // No Error
        Err(e) => {
            println!("Failed creating key in db: {:?} with: \"{}\" ", &key, &e);
            return 17; // according to errno.h and errno-base.h (maybe use https://docs.rs/nix/0.11.0/src/nix/errno.rs.html, or something else)
        }
    };
    //    println!("logging: saving state {:?} in {:?}", key, encrypted_state);
    0
}

#[no_mangle]
pub unsafe extern "C" fn ocall_new_delta(enc_delta: *const u8, delta_len: usize, delta_hash: [u8; 32],
                                         _delta_index: *const u32) -> i8 {
    let delta_index = ptr::read(_delta_index);
    let encrypted_delta = slice::from_raw_parts(enc_delta, delta_len);
    let key = DeltaKey::new(delta_hash, Stype::Delta(delta_index));
    // TODO: How should we handle the already existing error?
    match DATABASE.lock().expect("Database mutex is poison").create(&key, encrypted_delta) {
        Ok(_) => (), // No Error
        Err(e) => {
            println!("Failed creating key in db: {:?} with: \"{}\" ", &key, &e);
            return 17; // according to errno.h and errno-base.h (maybe use https://docs.rs/nix/0.11.0/src/nix/errno.rs.html, or something else)
        }
    }
    println!("logging: saving delta {:?} in {:?}", key, encrypted_delta);
    0
}

#[no_mangle]
pub unsafe extern "C" fn ocall_save_to_memory(data_ptr: *const u8, data_len: usize) -> u64 {
    let data = slice::from_raw_parts(data_ptr, data_len).to_vec();
    let ptr = Box::into_raw(Box::new(data.into_boxed_slice())) as *const u8;
    ptr as u64
}