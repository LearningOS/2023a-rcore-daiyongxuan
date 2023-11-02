//! Process management syscalls
use core::mem::size_of;

use crate::mm::{translated_byte_buffer, VirtAddr};

use crate::timer::get_time_us;
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token,
    },
};

use crate::task::{map_a_piece_of_virtal_address, unmap_area, query_task_info};

#[repr(C)]
#[derive(Debug)]
/// Timeval
pub struct TimeVal {
    /// sec
    pub sec: usize,
    /// usec
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Debug)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
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
    // _ts is in the virtual address, so we can not modify
    // it directly. we should translate it into kernel virtual
    // address. And beacause kernel virtual address is eqaul to
    // physical address, so we can use translated_byte_buffer 
    let _ts_translated = translated_byte_buffer(current_user_token(), _ts as *mut u8, size_of::<TimeVal>());
    let _trans_to_timeval = _ts_translated[0].as_ptr() as *mut u8 as *mut TimeVal;
    let current_time = get_time_us();
    unsafe {
        *_trans_to_timeval = TimeVal {
            sec: current_time / 1_000_000,
            usec: current_time % 1_000_000,
        }
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info!");
    let _ti_translated = translated_byte_buffer(current_user_token(), _ti as *mut u8, size_of::<TaskInfo>());
    let _trans_to_taskinfo = _ti_translated[0].as_ptr() as *mut TaskInfo;
    let mut cur_task_info = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: [0; MAX_SYSCALL_NUM],
        time: 0,
    };
    query_task_info(&mut cur_task_info);
    println!("time: {}", cur_task_info.time);
    unsafe {
        *_trans_to_taskinfo = TaskInfo {
            ..cur_task_info
        };
    }
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap!");
    let map_result = map_a_piece_of_virtal_address(VirtAddr::from(_start), _len, _port);
    match map_result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap!");
    let unmap_result = unmap_area(VirtAddr::from(_start), _len);
    match unmap_result {
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
