# ox6502

MOS 6502 / CMOS W65C02 CPU 模拟器，附带 NES 系统模拟，Rust 编写。

## 功能特性

- **CPU**：完整 6502/65C02 指令集，247/256 NMOS 非法操作码（SST 通过率 96.5%），全部 13 种寻址模式，交互式调试器
- **NES PPU**：背景 + 精灵渲染，loopy 地址系统，OAM DMA，NMI 时序
- **NES APU**：脉冲 1/2、三角波、噪声通道，帧计数器与音频输出
- **Mapper**：NROM（0）、MMC1（1）
- **输入**：键盘映射为 NES 手柄
- **显示**：SDL2 实时窗口（3x）或离线 PPM 渲染器

## 构建与运行

```bash
cargo build

# CPU 测试 ROM
cargo run -- tests/roms/6502_functional_test.bin
cargo run -- tests/roms/6502_functional_test.bin --debug

# NES 游戏（需安装 SDL2：brew install sdl2）
cargo run --bin nes_sdl -- <game.nes>
cargo run --bin nes_render -- <game.nes> [帧数]
```

## 键盘控制

| 按键 | NES 按钮 |
|------|----------|
| A | A |
| S | B |
| Backspace | Select |
| Enter | Start |
| ↑ ↓ ← → | 方向键 |
| Esc | 退出 |

## 调试器命令

`s` 单步 · `c` 继续 · `r` 寄存器 · `d [地址] [数量]` 反汇编 · `m [地址] [长度]` 内存查看 · `b <地址>` 断点 · `bc <id>` 清除断点 · `bl` 列出断点 · `t [数量]` 追踪 · `h` 帮助 · `q` 退出

## 关键设计

- **泛型总线**：`Cpu<B: Bus>` 不耦合具体总线实现，测试用 `TestBus`，CLI 用 `SimpleBus`
- **时钟模型**：CPU 为主时钟，每条指令执行后通过 `bus.tick()` 将消耗的周期广播给 PPU（×3 dots）和 APU
- **JSR/RTS**：JSR 推入 `PC-1`，RTS 弹出地址 +1，这是正确的 6502 行为
- **JMP (ind) 页边界 bug**：NMOS 6502 指针跨页时高字节从同页读取
- **NMI 不可屏蔽**：I 标志不影响 NMI

## SST 测试结果

247/256 操作码通过（96.5%）。剩余 9 个为不稳定操作码，行为因 CPU 版本而异。

测试文件位于 `tests/sst_tests.rs`，使用 `tests/sst_tests/nes6502/v1/` 下的 JSON 固定数据，每个操作码 10,000 组随机测试用例。

运行单个 SST 测试：`cargo test test_sst_00`（或任意十六进制操作码如 `test_sst_ff`）。

## 已实现 vs 缺失

**已实现**：完整 load/store、传送、标志位、跳转、增减、比较、分支、逻辑运算、移位/旋转、ADC/SBC（仅二进制，NMOS）、BRK、JMP 页边界 bug、247/256 NMOS 非法操作码。

**缺失**：9 个不稳定非法操作码（XAA、AHX、TAS、SHY、SHX、LAX#、AXS、SBC#），page-crossing 周期惩罚，分支跳转 +1 周期，精确周期级时序。

## 参考资料

- [W65C02S 数据手册](https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf)
- [6502 功能测试](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [NES Dev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)

## 许可证

MIT
