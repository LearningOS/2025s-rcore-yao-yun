# rCore-Tutorial-2025S Lab1 报告 

用于 https://opencamp.cn/os2edu/camp/2025spring。

## Changelog 

- 在`crate::task` 中
  - 扩展 TaskManager，新增可变字段 `task_syscall_stat`，是一 `MAX_APP_NUM` * `SYSCALL_MAX` 的二维`usize`数组，用以统计各任务系统调用次数。
  - 新增 TaskManager 的 `get_current_task_syscall_stat` 和 `update_current_task_syscall_stat` 方法，供查询及通过闭包更新。在mod中简单封装后暴露。
- 在 `crate::syscall` 中
  - 新增`SYSCALL_MAX`以提供系统调用统计数组的长度
  - 在 `syscall` 函数中，在系统调用前更新（+1）调用次数统计。
    - 注：不合法的系统调用仍然会被统计；从任务来看这是未定义行为。
  - 完成 `sys_trace` 的三种功能实现：读/写当前任务 id 一字节，及获取当前任务系统调用次数。

## 问答题

### 1

> 版本信息：
> RustSBI: `RustSBI-QEMU Version 0.2.0-alpha.3`
> QEMU: `9.2.3`

- `ch2b_bad_address.rs`: PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.  
  - U模式下，该程序试图越界写入 `0x0` 地址
  - 该地址位于RustSBI启动时配置的PMP保护段中
  - 该越权操作触发了`StoreFault`Trap，转入内核模式处理。
- `ch2b_bad_instructions.rs`: IllegalInstruction in application, kernel killed it.
  - U模式下，该程序试图使用`sret`指令，该指令为特权指令
  - 触发`IllegalInstruction`Trap，转入内核模式处理。
- `ch2b_bad_register.rs`: IllegalInstruction in application, kernel killed it.
  - U模式下，该程序试图访问`sstatus`这一CSR寄存器之一
  - 触发`IllegalInstruction`Trap，转入内核模式处理。

### 2

1. sp 的值：此时 sp 指向 内核栈顶，即保存的 TrapContext 结构体的起始地址（包含所有寄存器的保存值）。 两种使用情景包括 (a) 从 Trap 处理返回：处理完中断/异常后，恢复用户态继续执行; (b) 任务切换后恢复：切换到另一个任务时，恢复其保存的上下文。
2. 特殊处理了
   - sstatus：保存陷阱发生前的 CPU 状态（如中断使能位、权限模式等）。
   - sepc：保存陷阱发生时的程序计数器（PC），即返回用户态后执行的指令地址。
   - sscratch：保存用户栈指针，用于后续恢复用户栈。
    
    其中，sstatus 和 sepc 是 sret 指令返回用户态的必要条件。sscratch 在切换栈时用于保存内核栈指针，确保下次 Trap 能正确切换栈。
3. 对于x2，用户栈指针已通过 sscratch 恢复（见 L47），直接恢复会导致内核栈被覆盖。对于 x4，线程指针通常由内核管理，用户程序无权修改，因此无需恢复。
4. 执行前，sp 指向内核栈顶（已释放 TrapContext）；sscratch 保存用户栈指针。执行后，sp 指向用户栈指针；sscratch 保存内核栈指针，为下次 Trap 做准备。
5. 发生在sret。sret 会根据 sstatus 恢复权限模式。同时，sepc 的值会被写入 PC，CPU 跳转到用户态继续执行。
6. 执行前，sp 指向用户栈指针（触发 Trap 时的栈）；sscratch 保存内核栈指针。执行后，sp 指向内核栈指针；sscratch 保存用户栈指针，确保后续能恢复用户栈。
7. 发生在ecall。



## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。