//! Implements stride scheduling
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use super::task::TaskControlBlock;


#[allow(unused)]
static BIG_STRIDE: u128 = 2e10 as u128;

struct StrideWrapper<T> {
    task: T,
}

impl StrideWrapper<Arc<TaskControlBlock>> {
    fn new(task: Arc<TaskControlBlock>) -> Self {
        Self { task }
    }
    fn scheduled(&mut self) {
        let mut pcb_inner = self.task.inner_exclusive_access();
        pcb_inner.stride += BIG_STRIDE / pcb_inner.priority as u128;
    }
}

impl PartialEq for StrideWrapper<Arc<TaskControlBlock>> {
    fn eq(&self, other: &Self) -> bool {
        let pcb_inner = self.task.inner_exclusive_access();
        let other_inner = other.task.inner_exclusive_access();
        pcb_inner.stride == other_inner.stride
    }
}

impl PartialOrd for StrideWrapper<Arc<TaskControlBlock>> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let pcb_inner = self.task.inner_exclusive_access();
        let other_inner = other.task.inner_exclusive_access();
        Some(pcb_inner.stride.partial_cmp(&other_inner.stride)?.reverse())
        // smaller stride value is prioritized 
    }
}

impl Eq for StrideWrapper<Arc<TaskControlBlock>> {}

impl Ord for StrideWrapper<Arc<TaskControlBlock>> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let pcb_inner = self.task.inner_exclusive_access();
        let other_inner = other.task.inner_exclusive_access();
        pcb_inner.stride.cmp(&other_inner.stride).reverse()
        // smaller stride value is prioritized 
    }
}

pub struct StrideQueue<T> {
    queue: BinaryHeap<StrideWrapper<T>>,
}

impl StrideQueue<Arc<TaskControlBlock>> {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::<StrideWrapper<Arc<TaskControlBlock>>>::new(),
        }
    }

    pub fn push(&mut self, task: Arc<TaskControlBlock>) {
        self.queue.push(StrideWrapper::new(task));
    }

    pub fn pop(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.queue
            .pop()
            .map(|mut wrapper| {
                wrapper.scheduled();
                wrapper.task
            })
    }
}

