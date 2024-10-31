## 实现的功能

修改 `TaskManager` 部分逻辑，以二叉树形式存储待调度进程，并实现 stride 算法调度进程，以及设置进程优先级的 `set_priority` 和直接生成新子进程的 `spawn` 两个系统调用。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   ​    无

2. 此外，我也参考了以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   ​    无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

## 简答作业

- 不是，因为 `p2` 会溢出

- 假设某一情况下 `STRIDE_MAX – STRIDE_MIN > BigStride / 2`，则在执行的上一步时当前的 `STRIDE_MAX` 对应的 `STRIDE_MIN` 会有 `STRIDE_MAX_NOW - STRIDE_MIN_PREV`  对应的  `pass > BigStride / 2`，而由于进程优先级永远 `>= 2`，则对应的 pass 永远 `<= BigStride / 2`，那么 `STRIDE_MAX – STRIDE_MIN > BigStride / 2` 的情况不成立。

- ```rust
  use core::cmp::{max, min, Ordering};
  
  struct Stride(u64);
  
  impl PartialOrd for Stride {
      fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
          if max(self.0, other.0) - min(self.0, other.0) > BIG_STRIDE {
              other.0.partial_cmp(&self.0)
          } else {
              self.0.partial_cmp(&other.0)
          }
      }
  }
  
  impl PartialEq for Stride {
      fn eq(&self, other: &Self) -> bool {
          false
      }
  }
  ```