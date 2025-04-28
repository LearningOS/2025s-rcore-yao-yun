//! Process management syscalls
use crate::{
    mm::{MapPermission, UserRef, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_syscall_stat, mmap_current_task, munmap_current_task, suspend_current_and_run_next
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// current task gives up resources for other tasks
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let user_ref = UserRef::<TimeVal>::new(ts, current_user_token());
    let time_us = get_time_us();
    match user_ref.write(TimeVal { sec: time_us / 1_000_000, usec: time_us % 1_000_000 }) {
        Ok(_) => 0,
        Err(_) => -1,   
    } 
}

/// Trace syscall
/// 0: read
/// 1: write
/// 2: get syscall statistics
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let user_ref = UserRef::<u8>::new(id as *const u8, current_user_token());
            match user_ref.read() {
                Some(value) => value as isize,
                None => -1,
            }
        }
        1 => {
            let user_ref = UserRef::<u8>::new(id as *const u8, current_user_token());
            match user_ref.write(data as u8) {
                Ok(()) => 0,
                Err(()) => -1,
            }
        }
        2 => get_current_task_syscall_stat(id) as isize,
        _ => -1,
    }
}

/// Map a `len` virtual memory area starting from `start` with permission `prot` for current task
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap {:x?} {:x?} {:0?}", start, len, prot);
    if prot & 0x7 == 0 || prot & !0x7 != 0 {
        return -1;
    }
    if !VirtAddr(start).aligned() {
        return -1;
    }
    let permission = MapPermission::from_bits_truncate((prot as u8) << 1 | 0x10); // add PTE_U
    trace!("kernel: mapping with permission {:?}", permission);
    match mmap_current_task(start, len, permission) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Unmap a `len` virtual memory area starting from `start` for current task
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap {:x?} {:x?}", start, len);
    if !VirtAddr(start).aligned() {
        return -1;
    }
    match munmap_current_task(start, len) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
