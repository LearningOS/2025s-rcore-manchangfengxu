# 功能实现
- stride算法:
    * 为TCB加上schedule块(struct), 同时预留了pass设置的接口
    * 为sys_set_priority加入了对priority的设置 
    * 将TaskManager块改为了用binaryheap存储, 并为TCB分配了Ord特性,每次选取都会取stride最小的调度

- 向前兼容
    * 重写mmap和munmap(用到了remove_area_with_start_vpn)
    * 重写了sys_get_time,用到了translate_va

# 问答
- 实际情况是轮到 p1 执行吗？为什么？
是p2执行后应是260,但是8bit最大为255, 溢出p2为260 mod 2^8 = 4, 所以p2执行
-  为什么？尝试简单说明（不要求严格证明）。
每个进程的 pass 值由 pass = BIG_STRIDE / priority 计算得出。如果所有进程的 priority >= 2 ，那么它们各自的 pass 值必然满足 pass <= BIG_STRIDE / 2 。这意味着任何进程单次执行后，其 stride 值的最大增量不会超过 BIG_STRIDE / 2 。
- 补全代码
```
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff = self.0.wrapping_sub(other.0) as i64;
        if diff < 0 {
            Some(Ordering::Less) // self 更小
        } else {
            Some(Ordering::Greater) // self 更大
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

# 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

与群友就stride溢出处理进行讨论

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

chatgpt

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
