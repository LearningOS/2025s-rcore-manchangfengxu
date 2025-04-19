//! Process management syscalls
use crate::mm::{MapPermission, VirtAddr, VirtPageNum};
use crate::syscall::SYSCALL_USED_TABLE;
use crate::task::TASK_MANAGER;
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_us;
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let task_id = TASK_MANAGER.get_current_task();
    let vaddr: VirtAddr = (_ts as usize).into();
    let vpn: VirtPageNum = vaddr.floor();
    let idx: [usize; 3] = vpn.indexes();
    if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
        let ts: usize = pte.bits >> 9 << 9 | idx[3];

        let us = get_time_us();
        unsafe {
            *(ts as *mut TimeVal) = TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            };
        };
        return 0;
    } else {
        return -1;
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let task_id: usize = TASK_MANAGER.get_current_task();
    match _trace_request {
        0 => {
            println!("start get bit");
            let vaddr: VirtAddr = _id.into();
            let vpn: VirtPageNum = vaddr.floor();
            let idx: [usize; 3] = vpn.indexes();
            if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
                if !pte.readable() {
                    return -1;
                }
                let ptr: usize = pte.bits >> 9 << 9 | idx[3];
                println!("{:?}", ptr);
                unsafe {
                    let bit = *(ptr as *const u8) as isize;
                    return bit;
                }
            } else {
                return -1;
            }
        }
        1 => {
            println!("start set bit");
            let vaddr: VirtAddr = _id.into();
            let vpn: VirtPageNum = vaddr.floor();
            let idx: [usize; 3] = vpn.indexes();
            if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
                if !pte.writable() {
                    return -1;
                }
                let ptr: usize = pte.bits >> 9 << 9 | idx[3];
                println!("{:?}", ptr);
                unsafe {
                    *(ptr as *mut u8) = _data as u8;
                    0
                }
            } else {
                return -1;
            }
        }
        2 => {
            println!("start get task_sys_num");
            let a = unsafe { SYSCALL_USED_TABLE[task_id][_id] as isize };
            println!("current task:{} syscall:{} num:{}", id, _id, a);
            return a;
        }
        _ => {
            return -1;
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let mut vstart: VirtAddr = _start.into();
    let mut vend: VirtAddr = (vstart.0 + _len).into();
    vstart.to_floor_aligned();
    vend.to_floor_aligned();
    let task_id = TASK_MANAGER.get_current_task();
    let permission = MapPermission::from_bits((((_port << 1) + 1) | 16) as u8).unwrap();
    println!("start:{:?} end:{:?} permission:{:?}", vstart, vend, permission);
    TASK_MANAGER.insert_maparea(task_id, vstart, vend, permission)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let task_id = TASK_MANAGER.get_current_task();
    let mut vstart: VirtAddr = _start.into();
    let mut vend: VirtAddr = (vstart.0 + _len).into();
    vstart.to_floor_aligned();
    vend.to_floor_aligned();
    TASK_MANAGER.delete_maparea(task_id, vstart, vend)
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
