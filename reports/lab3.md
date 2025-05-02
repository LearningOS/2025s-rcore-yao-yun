# rCore-Tutorial-2025S Lab3 报告 

用于 https://opencamp.cn/os2edu/camp/2025spring。

## Changelog 

1. 移植lab2的 `mmap`/`munmap`/`sys_get_time` 实现
   1. 在 `TaskControlBlock` 中新增 `mmap` 和 `munmap` 方法，支持任务级别的内存管理。
2. 引入步进调度（Stride Scheduling），替换原有的简单队列调度器。
   1. 在 `TaskControlBlock` 中新增 `stride` 和 `priority` 字段，支持步进调度。
   2. 实现 `sys_set_priority` 系统调用，支持动态调整任务优先级。

## 问答题

### 问题 1

1. 不会轮到 p1 执行。因为 p2 执行一个时间片后，其 stride 值会溢出，从 255 变为 4，此时 p2 的 stride 值比 p1 的 stride 值小，因此调度器会再次选择 p2 执行。
2. 在优先级 >= 2 的情况下，每次调度时 stride 的增量至少为 `BIG_STRIDE / 2`。因此，任意两个进程的 stride 差值在每次调度后都会逐渐缩小，最终保持在 `BIG_STRIDE / 2` 以内。
3. 
    ```rust
    use core::cmp::Ordering;

    struct Stride(u64);

    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let diff = self.0.wrapping_sub(other.0);
            if diff < (u64::MAX / 2) {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        }
    }

    impl PartialEq for Stride {
        fn eq(&self, other: &Self) -> bool {
            false
        }
    }
    ```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。