//! Deadlock detect related
//! Implemented a modified version of the banker's algorithm

use alloc::collections::BTreeMap;

// use crate::task::process::{ProcessControlBlock, ProcessControlBlockInner};
// use crate::task::process::ProcessControlBlockInner;

// The banker contains the following fields:
// Available:
// M * available resources, from `mutex_list`, `semaphore_list`, and `condvar_list`
// Lifecycle:
// - Created / Updated at
// Allocation: N * M, N from ProcessControlBlockInner.tasks
// Updated by one when a thread requests / release a resource
// Need: the same.

// Once created, a mutex / semaphore / condvar lasts with the whole process
/// Track of resource status of current process
#[derive(Debug)]
pub struct ResourceStatus {
    /// Available resources, 1:1 to mutex_list + smaphore_list
    available: BTreeMap<usize, usize>,
    // for example [1, 0, 4, 0]
    /// Allocated resources, 1st layer 1:1 to `ProcessControlBlockInner.tasks`, 2nd to available
    allocated: BTreeMap<usize, BTreeMap<usize, usize>>,
    // for example [
    // [0, 0, 1, 0],
    //  ^mtx |^smf
    // [1, 0, 1, 3],
    // [0, 0, 1, 0]
    // ]
    /// Needed resources, the same as allocated
    needed: BTreeMap<usize, BTreeMap<usize, usize>>,
    // if we are running check each time we allocate something:
    // for example [
    // [0, 0, 1, 0],
    // [0, 0, 0, 0],
    // [0, 0, 0, 0]
    // ]
}

impl ResourceStatus {
    /// Create an empty resource status.
    pub fn new() -> ResourceStatus {
        ResourceStatus {
            available: BTreeMap::new(),
            allocated: BTreeMap::from([(0, BTreeMap::new())]),
            needed: BTreeMap::from([(0, BTreeMap::new())]),
        }
    }

    /// Track new task
    /// Insert a new row in needed & allocated
    pub fn track_task(&mut self, tid: usize) {
        assert!(!self.allocated.contains_key(&tid)); // ensure not already tracked
        self.allocated.insert(
            tid,
            self.allocated[&0].keys().map(|&k| (k.clone(), 0)).collect(),
        );
        self.needed.insert(
            tid,
            self.needed[&0].keys().map(|&k| (k.clone(), 0)).collect(),
        );
    }

    /// Untrack new task
    pub fn untrack_task(&mut self, tid: usize) {
        assert!(self.allocated.contains_key(&tid)); // ensure tracked
        self.allocated.remove(&tid);
        self.needed.remove(&tid);
    }

    /// Track new resource
    /// Insert a new column in needed & allocated or add to existing
    pub fn update_resource(&mut self, resource_id: usize, diff: isize) {
        trace!(
            "Updating resource {} with diff {}",
            resource_id,
            diff
        );
        if let Some(available) = self.available.get_mut(&resource_id) {
            *available = (*available as isize + diff) as usize;
        } else {
            self.available.insert(resource_id, diff as usize);
        }

        self.allocated.iter_mut().for_each(|(_, v)| {
            if let None = v.get_mut(&resource_id) {
                v.insert(resource_id, 0);
            }
        });
        self.needed.iter_mut().for_each(|(_, v)| {
            if let None = v.get_mut(&resource_id) {
                v.insert(resource_id, 0);
            }
        });
    }

    /// Untrack resource
    pub fn forget_resource(&mut self, resource_id: usize) {
        self.available.remove(&resource_id);
        self.allocated.iter_mut().for_each(|(_, v)| {
            v.remove(&resource_id);
        });
        self.needed.iter_mut().for_each(|(_, v)| {
            v.remove(&resource_id);
        });
    }

    /// Add new need
    pub fn need(&mut self, task_id: usize, resource_id: usize, res_count: usize) {
        self.needed
            .get_mut(&task_id)
            .unwrap()
            .insert(resource_id, res_count);
    }

    /// Add new need
    pub fn try_need(&mut self, task_id: usize, resource_id: usize, res_count: usize) -> Option<()> {
        trace!(
            "Trying to need {} resources for task {} on resource {}",
            res_count,
            task_id,
            resource_id
        );
        trace!("Resource Status: {:?}", self);
        let old_val = self.needed[&task_id][&resource_id];
        self.needed
            .get_mut(&task_id)
            .unwrap()
            .insert(resource_id, old_val + res_count);
        match self.will_deadlock() {
            true => {
                self.needed
                    .get_mut(&task_id)
                    .unwrap()
                    .insert(resource_id, old_val);
                None
            }
            false => Some(()),
        }
    }

    /// Allocate
    pub fn allocate(&mut self, task_id: usize, resource_id: usize, res_count: usize) {
        trace!(
            "Allocating {} resources for task {} on resource {}",
            res_count,
            task_id,
            resource_id
        );
        *(self
            .needed
            .get_mut(&task_id)
            .unwrap()
            .get_mut(&resource_id)
            .unwrap()) -= res_count;
        *(self
            .allocated
            .get_mut(&task_id)
            .unwrap()
            .get_mut(&resource_id)
            .unwrap()) += res_count;
        *(self.available.get_mut(&resource_id).unwrap()) -= res_count;
        trace!("Resource Status: {:?}", self);
    }

    /// Release
    pub fn release(&mut self, task_id: usize, resource_id: usize, res_count: usize) -> Option<()> {
        trace!(
            "Releasing {} resources for task {} on resource {}",
            res_count,
            task_id,
            resource_id
        );
        trace!("Resource Status: {:?}", self);
        if let Some(task_allocated) = self.allocated.get_mut(&task_id) {
            if let Some(resource_allocated) = task_allocated.get_mut(&resource_id) {
                *resource_allocated -= res_count;
                *(self.available.get_mut(&resource_id).unwrap()) += res_count;
                Some(())
            } else {
                return None
            }
        } else {
            return None
        }
    }

    /// Check if the current resource status is safe.
    pub fn will_deadlock(&self) -> bool {
        trace!("Checking deadlock: {:?}", self);
        let mut work = self.available.clone();
        let mut finish = BTreeMap::from_iter(self.allocated.keys().map(|&k| (k, false)));

        let mut can_finish = true;
        while can_finish {
            trace!("Find next task to finish from {:?}", finish);
            trace!("Work status: {:?}", work);
            if let Some((&tid, _)) = finish
                .iter()
                .filter(|(_, &v)| !v)
                .find(|(&tid, _)| {
                    self.needed[&tid]
                        .iter()
                        .all(|(resource_id, &res_count)| work[resource_id] >= res_count)
                })
                .clone()
            {
                finish.insert(tid, true);
                self.allocated[&tid]
                    .iter()
                    .for_each(|(resource_id, &res_count)| {
                        work.insert(resource_id.clone(), work[resource_id] + res_count);
                    });
            } else {
                can_finish = false;
            }

            if finish.values().all(|&v| v) {
                break;
            }
        }

        trace!("May finish: {:?}", can_finish);
        !can_finish
    }
}
