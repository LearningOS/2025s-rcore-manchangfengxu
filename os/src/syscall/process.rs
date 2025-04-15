//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER},
    timer::get_time_us,
};

use super::TASK_SYSCALL_NUM;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    let id:usize = TASK_MANAGER.get_current_task();
    match _trace_request {
        0 => {
            // println!("start get bit");
            unsafe {
                let ptr = ( _id) as *const u8;
                let bit = *ptr as isize;
                return bit;
            }
        }
        1 => {
            unsafe {
                let ptr = ( _id) as *mut u8;
                *ptr = _data as u8;
                // println!("set bit{}", _data);
                return 0; // success cod
            }
        }
        2 => {
            // println!("start get task_sys_num");
            let a = unsafe { TASK_SYSCALL_NUM[id][_id] as isize};
            // println!("current task:{} syscall:{} num:{}", id, _id, a);
            return a;
        }
        _ => {
            return -1;
        }
    }
    
}
