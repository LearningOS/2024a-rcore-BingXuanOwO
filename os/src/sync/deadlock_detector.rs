//! Dead lock detector for mutex and semaphore.

use alloc::vec::Vec;
use alloc::vec;

/// Used to detect deadlock
pub struct DeadlockDetector {
    available: Vec<usize>,
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>,
    /// thread count of current detector
    pub thread_count: usize,
    /// resource count of current detector
    pub resource_count: usize,
}

impl DeadlockDetector {
    /// Create a new deadlock detector instance.
    /// `tc`: total thread count, `available`: all resource's count
    pub fn new() -> Self {
        let mut result = Self {
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
            thread_count: 0,
            resource_count: 0,
        };
        result.add_thread(0);
        result
    }
    /// Continusly add thread count until detector's total thread count above th_i
    pub fn add_thread(&mut self, th_i: usize) {
        for _i in self.thread_count..th_i + 1 {
            self.allocation.push(vec![0;self.resource_count]);
            self.need.push(vec![0;self.resource_count]);
            self.thread_count += 1;
        }
    }
    /// Continusly add resource count until detector's total resource count above rc_i
    pub fn add_resource(&mut self, rc_i: usize, count: usize) {
        for _j in self.resource_count..rc_i + 1 {
            self.available.push(count);
        }

        for alloc in self.allocation.iter_mut() {
            for _j in self.resource_count..rc_i + 1 {
                alloc.push(0);
            }
        }

        for need in self.need.iter_mut() {
            for _j in self.resource_count..rc_i + 1 {
                need.push(0);
            }
        }

        self.resource_count = rc_i + 1;
    }
    /// Try to acquire a resource as needed,
    /// change will only emit when result is `Ok()`;
    pub fn try_acquire(&mut self, th_i: usize, rc_i: usize, count: usize) -> Result<(), ()> {
        let mut need = self.need.clone();
        let mut work = self.available.clone();
        let mut finish = vec![false;self.thread_count];
        
        // add needed count of resource
        need[th_i][rc_i] += count;

        // main loop
        loop {
            let mut found = false;
            for i in 0..self.thread_count {
                if finish[i] == true { continue; };
                let mut skip = false;

                for _rc_i in 0..self.resource_count {
                    if !(need[i][_rc_i] <= work[_rc_i]) {
                        skip = true;
                        break;
                    }
                }
                if skip { continue; };

                for _rc_i in 0..self.resource_count {
                    work[_rc_i] += self.allocation[i][_rc_i];
                }
                found = true;
                finish[i] = true;
                
            }
            if !found {
                break;
            }
        }

        // returning
        for fi in finish {
            if !fi {
                return Err(())
            }
        }

        self.need = need;
        Ok(())
    }

    /// allocate a resource for given count.
    /// Assuming change won't issue deadlock.
    pub fn alloc(&mut self, th_i: usize, rc_i: usize, count: usize) {
        assert!(self.available[rc_i] >= count);
        assert!(self.need[th_i][rc_i] >= count);
        self.available[rc_i] -= count;
        self.allocation[th_i][rc_i] += count;
        self.need[th_i][rc_i] -= count;
    }

    /// Release a resource for given count.
    /// If release more than allocated count of resource,
    /// will add new count of resource to system.
    pub fn release(&mut self, th_i: usize, rc_i: usize, count: usize) {
        self.available[rc_i] += count;

        if self.allocation[th_i][rc_i] >= count {
            self.allocation[th_i][rc_i] -= count;
        } else {
            self.allocation[th_i][rc_i] = 0;
        }
    }
}