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


