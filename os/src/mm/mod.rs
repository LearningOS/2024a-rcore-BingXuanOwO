//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use core::mem::size_of;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};

use crate::task::current_task;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}

/// map a area into current user's memory set
pub fn map_area_from_token(start: usize, len: usize, map_perm: MapPermission) -> Result<(), ()> {
    // let mut pte = PageTable::from_token(token);
    // let start_vpn: VirtPageNum  = VirtAddr(start).floor();
    // let end_vpn: VirtPageNum = VirtAddr(start + len).ceil();
    // let pte_flags = PTEFlags::from_bits(map_perm.bits()).unwrap();
    let task_opt = current_task();
    if task_opt.is_none() {
        return Err(());
    }

    let memory_set = task_opt.unwrap().get_memory_set();

    unsafe {
        (*memory_set).map(start.into(), (start+len).into(), map_perm)
    }
}

/// unmap a area 
pub fn unmap_area_from_token(start: usize, len: usize) -> Result<(), ()> {
    // let token = current_user_token();
    // let mut pte: PageTable = PageTable::from_token(token);
    // let start_vpn: VirtPageNum  = VirtAddr(start).floor();
    // let end_vpn: VirtPageNum = VirtAddr(start + len).ceil();

    let task_opt = current_task();
    if task_opt.is_none() {
        return Err(());
    }

    let memory_set = task_opt.unwrap().get_memory_set();

    // unsafe {
    //     memory_set.areas
    // }

    // for vpn in VPNRange::new(start_vpn, end_vpn) {
    //     pte.unmap(vpn);
    // }
    unsafe {
       (*memory_set).unmap(start.into(), (start+len).into())
    }
}

/// map a pointer from current user space to kernel
pub fn map_user_ptr_to_kernel<T>(ptr: *mut T) -> Result<*mut T, ()> {
    let task_opt = current_task();
    if task_opt.is_none() {
        return Err(());
    }

    let type_size = size_of::<T>();
    let user_addr = VirtAddr(ptr as usize);

    let offset: usize = user_addr.page_offset();

    let ret_addr_begin: VirtPageNum = VirtPageNum(10000);
    let ret_addr_begin_addr: VirtAddr = ret_addr_begin.into();
    let addr_end = VirtAddr(ret_addr_begin_addr.0 + offset + type_size);
    let mut kernel_space = KERNEL_SPACE.exclusive_access();

    let current_user_memory_set = task_opt.unwrap().get_memory_set();

    let mut kernel_vpn_iter = VPNRange::new(ret_addr_begin, addr_end.ceil()).into_iter();

    for vpn in VPNRange::new(VirtAddr(ptr as usize).floor(), VirtAddr(ptr as usize + type_size).ceil()) {
        let ppn_opt = unsafe { (*current_user_memory_set).translate(vpn)};

        if let Some(ppn) = ppn_opt {
            if let Some(kernel_vpn) = kernel_vpn_iter.next() {
                kernel_space.map_one(kernel_vpn, ppn.ppn(), MapPermission::R | MapPermission::W);
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }

    drop(kernel_space);

    Ok((ret_addr_begin_addr.0 + offset) as *mut T)
}

/// free temp ptr that used to map a user ptr to kernel
pub fn unmap_user_ptr_to_kernel_from_kernel_addr<T>(ptr: *mut T) {
    let type_size = size_of::<T>();
    let addr = VirtAddr(ptr as usize);

    let start = addr.floor();
    let end = VirtAddr(ptr as usize + type_size).ceil();

    let mut kernel_space = KERNEL_SPACE.exclusive_access();

    for vpn in VPNRange::new(start, end) {
        kernel_space.unmap_one(vpn);
    }
}
