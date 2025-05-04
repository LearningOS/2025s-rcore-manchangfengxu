//! File and filesystem-related syscalls
use crate::fs::{
    open_file, root_add_dir_entry, root_get_id, root_remove_dir_entry, OpenFlags, Stat,
};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, VirtAddr};
use crate::task::{current_task, current_user_token};
const VA_WIDTH_SV39: usize = 39;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );

    let vaddr: VirtAddr = (_st as usize).into();
    if vaddr.0 >= (1 << VA_WIDTH_SV39) {
        return -1;
    }
    
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if let Some(file) = &inner.fd_table[_fd] {
        let stat = file.stat();
        if let Some(paddr) = inner.memory_set.translate_va(vaddr) {
            drop(inner);
            unsafe {
                *(paddr.0 as *mut Stat) = stat;
                0
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let _old_name = translated_str(token, _old_name);
    let _new_name = translated_str(token, _new_name);
    if _old_name == _new_name {
        return -1;
    }
    if let Some(old_inode) = open_file(_old_name.as_str(), OpenFlags::RDWR) {
        let old = old_inode.inner_inode_exclusive_access();
        if old.is_dir() {
            drop(old);
            return -1;
        }
        match root_get_id(_old_name.as_str()) {
            Some(old_id) => {
                old.inode_change_by_id(old_id, |inode| {
                    inode.nlink += 1;
                });
                drop(old);
                root_add_dir_entry(_new_name.as_str(), old_id);
            }
            None => return -1,
        }
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let name = translated_str(token, _name);
    if name.is_empty() {
        return -1;
    }
    root_remove_dir_entry(name.as_str());
    0
}
