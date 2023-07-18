# difftest

为了让我自己的模拟器作为ref和一些标准实现dut, 比如qemu, spike等已有的被验证是正确的模拟器进行比较, 我们需要一个工具来进行比较. 这个工具就是difftest.

difftest的原理是, 在模拟器中, 每次执行一条指令后, 将模拟器的状态(寄存器, 内存等)和ref的状态进行比较, 如果不一致, 则输出错误信息.

可选的difftest实现有很多种, 在这里我的实现思路是, 让qemu作为ref, 在qemu中运行riscv64-unknown-elf-objdump生成的二进制文件, 在我的模拟器中运行同样的二进制文件, 然后在模拟器中运行一条指令后, 将模拟器的状态和qemu的状态进行比较, 如果不一致, 则输出错误信息.

# 使用rust的gdbstub库和qemu进行difftest的详细过程

通过gdbstub向qemu 发送gdb指令, 从而实现qemu的控制. 这一过程通过网络进行通信, 通信协议是gdb remote protocol.
1. 先起一个qemu进程, 通过 `qemu-system-($ISA) -S -gdb tcp::$(port) -serial none -monitor none -nograph` 启动qemu, 这样qemu会监听一个端口, 等待gdbstub连接.

各个参数的含义如下:
- -S: 启动qemu后暂停, 等待gdbstub连接.
- -gdb tcp::$(port): 监听端口$(port), 等待gdbstub连接.
- -serial none: 不使用串口. 因为我们直接使用网络进行通信, 所以不需要串口. 如果不使用网络通信, 则需要使用串口. 这里的串口指的是一种虚拟串口, 我们也可以使用串口来进行通信, 但是这里不使用串口.
- -monitor none: 不使用monitor. 如果使用monitor, 在运行qemu之后会有一个命令行的交互界面:
```bash
qemu-system-riscv64 -S -gdb tcp::1145 -serial none -nographic
QEMU 7.0.0 monitor - type 'help' for more information
(qemu) 
```
但这次实践中, 我们不使用这个交互界面, 所以不使用monitor.

- -nograph: 不使用图形界面. 否则会弹出一个图形界面, 显示qemu的运行情况.

2. gdbstub链接qemu.
我们需要实现gdbstub的一些trait, ![gdbstub crate文档](https://docs.rs/gdbstub/latest/gdbstub/)

3. gdbstub根据协议内容向qemu发送gdb指令, 从而控制qemu.