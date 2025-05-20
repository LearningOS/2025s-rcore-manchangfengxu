# 实现功能
- 在ProcessControlBlockInner加入了对mutex和sem的死锁检查块(all[], ava[], need[])
- 检测前对相应资源的need[] + 1
- 实现is_safe检测函数, 对finish==false和need <= work的块, 回收allocation和finish=true,对标记flag=true, 当finish没有任何改变, 即本次循环flag==false时退出loop, 利用闭包all,检测finish所有线程是否全是true
- 若为unsafe, 则回退need, 返回-0xdead
- 若为safe, 则在down和lock之前drop(process_inner),防止线程堵塞无法释放资源, 在down和lock之后同时更新检查块中的矩阵
- 为up和unlock加上检查块的更新

# 问答题
## 资源问题
- 线程级资源：
    - 内核栈（KernelStack::kstack_alloc分配的物理页）, 包括线程的
    - 用户资源（TaskUserRes中的用户栈和trap上下文）
    - 任务上下文（TaskContext）
- 进程级资源：
    - 页表（MemorySet）
    - 文件描述符（fd_table）
    - 同步资源（mutex_list/semaphore_list等）
    - 子进程引用（children）
    - 信号标志（signals）
- 引用位置
    - 需要释放:
        - 进程块中的tasks
        - wait_queue中
    - 不需要
        - 其它线程的引用

## Mutex实现问题
- Mutex1的lock里,会一直尝试获取锁, 具体逻辑为当无法获得锁时,直接阻塞,让出cpu,直到被唤醒, 再重新尝试获得锁, unlock中释放锁,并且唤醒一个线程去竞争这个锁.
- Mutex的lock,在无法获得锁时,直接堵塞,在unlock时,只有等待队列为空才释放锁.
        - 这里的unlock本质是锁资源的转移, A不释放锁, 而是唤醒一个直接使用这个资源的B线程(它醒来后直接运行临界区后的代码)
# 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

与群友做了讨论

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

chatgpt

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。