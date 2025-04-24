//! Process management syscalls
use crate::mm::{MapPermission, PhysAddr,VirtAddr, VirtPageNum};
use crate::syscall::SYSCALL_USED_TABLE;
use crate::task::TASK_MANAGER;
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_us;
const VA_WIDTH_SV39: usize = 39;
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
    // println!("get time");
    let task_id = TASK_MANAGER.get_current_task();
    let vaddr: VirtAddr = (_ts as usize).into();
    if vaddr.0 >= (1 << VA_WIDTH_SV39) {
        return -1;
    }
    let vpn: VirtPageNum = vaddr.floor();

    if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
        if !pte.writable() {
            return -1;
        }
        let phy: PhysAddr = pte.ppn().into();
        let mut ptr: usize = phy.into();
        // println!("{:#b}, {:#b}", ptr, vaddr.page_offset());
        ptr += vaddr.page_offset();
        // println!("{:#b}", ptr);
        let us = get_time_us();
        // println!("{:?}", us);
        unsafe {
            *(ptr as *mut TimeVal) = TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            };
        };
        0
    } else {
        -1
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
            if _id >= (1 << VA_WIDTH_SV39) {
                println!("_id over");
                return -1;
            }
            let vaddr: VirtAddr = _id.into();
            let vpn: VirtPageNum = vaddr.floor();
            if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
                if !pte.readable() {
                    println!("_id not readable");
                    return -1;
                }
                let phy: PhysAddr = pte.ppn().into();
                let mut ptr: usize = phy.into();
                // println!("{:#b}, {:#b}", ptr, vaddr.page_offset());
                ptr += vaddr.page_offset();
                // println!("ptr{:?}", ptr);
                unsafe {
                    let bit = *(ptr as *const u8) as isize;
                    println!("bit {}", bit);
                    bit
                }
            } else {
                println!("_id not found");
                -1
            }
        }
        1 => {
            println!("start set bit");
            if _id >= (1 << VA_WIDTH_SV39) {
                println!("_id over");
                return -1;
            }
            let vaddr: VirtAddr = _id.into();
            let vpn: VirtPageNum = vaddr.floor();
            if let Some(pte) = TASK_MANAGER.get_pte(task_id, vpn) {
                if !pte.writable() {
                    println!("_id not writable");
                    return -1;
                }
                let phy: PhysAddr = pte.ppn().into();
                let mut ptr: usize = phy.into();
                // println!("{:#b}, {:#b}", ptr, vaddr.page_offset());
                ptr += vaddr.page_offset();
                println!("ptr{:x}", ptr);
                unsafe {
                    *(ptr as *mut u8) = _data as u8;
                    0
                }
            } else {
                println!("_id not found");
                -1
            }
        }
        2 => {
            println!("start get task_sys_num");
            unsafe { SYSCALL_USED_TABLE[task_id][_id] as isize }
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    println!("start mmap");
    println!("start:{:x} len:{:x} port:{:b}", _start, _len, _port);
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if ((_port >> 3) > 0) || ((_port & 7) == 0) {
        println!("port error");
        return -1;
    }
    let vstart: VirtAddr = _start.into();
    if !vstart.aligned() {
        println!("vstart aligned error");
        return -1;
    }

    let vend:VirtAddr = (_start + _len).into();

    if vend.0 >= (1 << VA_WIDTH_SV39) {
        println!("vend over error");
        return -1;
    }
    let task_id = TASK_MANAGER.get_current_task();
    for i in vstart.floor().0..vend.ceil().0 {
        if i == vend.ceil().0 {
            break;
        }
        let i: VirtPageNum = i.into();
        if let Some(pte) = TASK_MANAGER.get_pte(task_id, i) {
            if pte.is_valid() {
                println!("ptee error:{:b}, i:{:x}", pte.bits, i.0);
                return -1;
            }

        }
    }

    let permission = MapPermission::from_bits(((_port & 0b111 | 0b1000) << 1) as u8).unwrap();
    println!("start:{:b} end:{:b} permission:{:b}", vstart.0, vend.0, permission.bits());
    TASK_MANAGER.insert_maparea(task_id, vstart, vend, permission);
    println!("mmap success");
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    println!("start munmap");
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let vstart: VirtAddr = _start.into();
    if !vstart.aligned() {
        return -1;
    }

    let vend: VirtAddr = (vstart.0 + _len).into();
    if vend.0 >= (1 << VA_WIDTH_SV39) {
        return -1;
    }

    let task_id = TASK_MANAGER.get_current_task();
    for i in vstart.floor().0..vend.ceil().0 {
        let i: VirtPageNum = i.into();
        if let Some(pte) = TASK_MANAGER.get_pte(task_id, i) {
            if !pte.is_valid() {
                return -1;
            }
        }
    }
    TASK_MANAGER.delete_maparea(task_id, vstart, vend);
    println!("munmap success");
    0
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
