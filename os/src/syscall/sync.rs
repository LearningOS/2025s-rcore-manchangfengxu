use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task, current_task_tid};
use crate::timer::{add_timer, get_time_ms};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        process_inner.mutex_deadlock_detect.available[id] = 1;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_deadlock_detect.available.push(1);
        process_inner.mutex_list.len() as isize - 1
    }
}
fn is_safe(
    available: &[usize],
    allocation: &[Vec<usize>],
    need: &[Vec<usize>],
    t_num: usize,
) -> bool {
    let mut work = available.to_owned();
    let mut all = allocation.to_owned();
    let mut finish = vec![false; all.len()];

    loop {
        let mut flag = false;

        for i in 0..t_num {
            if !finish[i] {
                let need_res: Vec<_> = need[i]
                    .iter()
                    .enumerate()
                    .filter(|(_, res)| **res > 0)
                    .map(|(id, _)| id)
                    .collect();

                if need_res.is_empty() || need_res.iter().all(|&res| need[i][res] <= work[res]) {
                    // 处理资源分配
                    for (id_res, item) in work.iter_mut().enumerate() {
                        *item += all[i][id_res];
                        all[i][id_res] = 0;
                    }
                    finish[i] = true;
                    flag = true;
                }
            }
        }

        // 退出条件检测
        if !flag {
            break;
        }
    }
    let a: Vec<_> = finish
        .iter()
        .enumerate()
        .filter(|(t_id, _)| *t_id < t_num)
        .collect();
    println!("finish:{:?}", a);
    finish
        .iter()
        .enumerate()
        .filter(|(t_id, _)| *t_id < t_num)
        .all(|(_, &item)| item)
}

/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let t_num = process_inner.tasks.len();
    // 死锁检测逻辑
    if process_inner.enable_deadlock_detect {
        let banker = &mut process_inner.mutex_deadlock_detect;
        // 记录need
        let tid = current_task_tid();
        if banker.allocation.len() <= tid {
            banker
                .allocation
                .resize(tid + 1, vec![0; banker.available.len()]);
        }
        banker.need[tid][mutex_id] += 1;
        // 安全性检查
        if !is_safe(&banker.available, &banker.allocation, &banker.need, t_num) {
            banker.need[tid][mutex_id] -= 1;
            return -0xdead;
        }
    }
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    process_inner = process.inner_exclusive_access();
    mutex.lock();
    if process_inner.enable_deadlock_detect {
        let banker = &mut process_inner.mutex_deadlock_detect;
        let tid = current_task_tid();
        banker.allocation[tid][mutex_id] += 1;
        banker.need[tid][mutex_id] -= 1;
        banker.available[mutex_id] -= 1;
    }
    drop(process_inner);
    drop(process);
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    mutex.unlock();
    if process_inner.enable_deadlock_detect {
        let banker = &mut process_inner.mutex_deadlock_detect;
        let tid = current_task_tid();
        banker.allocation[tid][mutex_id] -= 1;
        banker.available[mutex_id] += 1;
    }
    drop(process_inner);
    drop(process);
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        process_inner.semaphore_deadlock_detect.available[id] = res_count;
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner
            .semaphore_deadlock_detect
            .available
            .push(res_count);
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    sem.up();
    // 回收
    if process_inner.enable_deadlock_detect {
        let banker = &mut process_inner.semaphore_deadlock_detect;
        let tid = current_task_tid();
        banker.allocation[tid][sem_id] -= 1;
        banker.available[sem_id] += 1;
    }
    drop(process_inner);
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let t_num = process_inner.tasks.len();
    if process_inner.enable_deadlock_detect {
        let banker = &mut process_inner.semaphore_deadlock_detect;
        let tid = current_task_tid();
        if banker.allocation.len() <= tid {
            banker
                .allocation
                .resize(tid + 1, vec![0; banker.available.len()]);
        }
        // 安全检测
        banker.need[tid][sem_id] += 1;
        if !is_safe(&banker.available, &banker.allocation, &banker.need, t_num) {
            banker.need[tid][sem_id] -= 1;
            return -0xdead;
        }
    }
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    process_inner = process.inner_exclusive_access();
    if process_inner.enable_deadlock_detect {
        // down完刷新数据
        let banker = &mut process_inner.semaphore_deadlock_detect;
        let tid = current_task_tid();
        banker.allocation[tid][sem_id] += 1;
        banker.need[tid][sem_id] -= 1;
        banker.available[sem_id] -= 1;
    }
    drop(process_inner);
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.enable_deadlock_detect = _enabled == 1;
    0
}
