use core::mem::size_of;
use alloc::vec::Vec;
use crate::mm::{PageTable, VirtAddr};
use crate::config::PAGE_SIZE;

/// A special class to handle references to userspace data structures in kernel space, with permission check
pub struct UserRef<T> {
    user_ptr: VirtAddr,
    token: usize,
    len: usize,
    phantom: core::marker::PhantomData<T>
}

impl<T> UserRef<T> {
    /// Create a new `UserRef` from a userspace reference.
    pub fn new(user_ptr: *const T, token: usize) -> Self {
        Self {
            user_ptr: VirtAddr::from(user_ptr as usize),
            token: token,
            len: size_of::<T>(),
            phantom: core::marker::PhantomData,
        }
    }

    /// Read from the userspace memory immediately.
    pub fn read(self) -> Option<T> {
        let page_table = PageTable::from_token(self.token);
        let mut buffer = Vec::with_capacity(self.len);
        let mut offset = 0;
        while offset < self.len {
            let start = usize::from(self.user_ptr) + offset;
            let current_vpn = VirtAddr(start).floor();
            match page_table.translate(current_vpn) {
                Some(pte) if pte.is_valid() && pte.readable() && pte.exposed_to_user() => {
                    let current_ppn = pte.ppn();
                    let page_offset = VirtAddr(start).page_offset();
                    let copy_len = (PAGE_SIZE - page_offset).min(self.len - offset);
                    let src = &current_ppn.get_bytes_array()[page_offset..page_offset + copy_len];
                    buffer.extend_from_slice(src);
                    offset += copy_len;
                }
                _ => {
                    return None;
                }
            }
        }
        
        if buffer.len() == self.len {
            Some(unsafe { core::ptr::read(buffer.as_ptr() as *const T) })
        } else {
            None
        }
    }

    /// Write to the userspace memory immediately.
    pub fn write(self, value: T) -> Result<(), ()> {
        let page_table = PageTable::from_token(self.token);
        let value_bytes = unsafe {
            core::slice::from_raw_parts(&value as *const T as *const u8, self.len)
        };
        let mut offset = 0;

        while offset < self.len {
            let start = usize::from(self.user_ptr) + offset;
            let current_vpn = VirtAddr(start).floor();
            match page_table.translate(current_vpn) {
                Some(pte) if pte.is_valid() && pte.writable() && pte.exposed_to_user() => {
                    let current_ppn = pte.ppn();
                    let page_offset = VirtAddr(start).page_offset();
                    let copy_len = (PAGE_SIZE - page_offset).min(self.len - offset);
                    let dest = &mut current_ppn.get_bytes_array()[page_offset..page_offset + copy_len];
                    dest.copy_from_slice(&value_bytes[offset..offset + copy_len]);
                    offset += copy_len;
                }
                _ => {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}