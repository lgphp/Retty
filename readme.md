### Introduction

Retty is a High performance I/O framework written by Rust inspired by Netty

基于mio的IO多路复用高并发、高性能IO开发框架

### Feature

- IO多路复用模型
- 内置Bytebuf数据容器
- ChannelPipeline 模型
- 默认支持TCP 未来支持UDP

还没写完，刚实现了一部分功能。 我会努力的。。。。。。

- 2022-1-28 : 完成出入站handler分离

> Channel_Handler_Context 分离
>
> Channel_Handler_Context_Pipeline 分离
>
> 包装TCPStream 和 Channel
>
>

// todo: implement

- 内置固定消息长度字段解码器
- 内置HTTP 协议解码器
- 内置WebSocket 协议解码器
- 内置flatBuffer 解码器
- 内置protoBuffer 解码器

### Quick Start

```rust 

参见 main.rs

```

### 鸣谢

**创造使我快乐

广交天下在高性能实时通信编程方面的朋友: lgphp2019@gmail.com


