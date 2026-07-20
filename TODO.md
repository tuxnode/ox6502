# TODO

## 已完成

### CPU 核心
- [x] 项目结构（main.rs, cpu.rs, bus/, opcodes.rs, addressing.rs, instructions.rs）
- [x] Bus trait 定义（cpu_read/write, ppu_read/write, tick）
- [x] SimpleBus 实现（64KB 内存 + load 方法）
- [x] CPU 结构体（寄存器 A/X/Y/SP/PC/Status）
- [x] 状态寄存器 flag 常量（FLAG_C/Z/I/D/B/V/N）
- [x] 构造函数 new()（读取复位向量 $FFFC）
- [x] set_flag / get_flag / update_nz / compare
- [x] fetch / fetch_u16
- [x] read / write（委托给 Bus）
- [x] 周期计数器 cycles
- [x] step() 返回周期数
- [x] run() 自动累加周期
- [x] Opcodes 常量表（65C02 全部指令）
- [x] 所有寻址模式（immediate, zeropage, zeropage_x/y, absolute, absolute_x/y, indirect, pre_indexed_x/y, post_indexed_x/y, relative）
- [x] 栈操作 push / pop
- [x] NOP 指令
- [x] Load/Store 指令（LDA/LDX/LDY/STA/STX/STY/STZ）
- [x] Transfer 指令（TAX/TAY/TXA/TYA/TSX/TXS）
- [x] Flag 指令（CLC/CLD/CLI/CLV/SEC/SED/SEI）
- [x] Jump/Call 指令（JMP/JSR/RTS/RTI）
- [x] Increment/Decrement 指令（INC/DEC/INX/INY/DEX/DEY）
- [x] Compare 指令（CMP/CPX/CPY）
- [x] Branch 指令（BCC/BCS/BEQ/BNE/BMI/BPL/BVC/BVS/BRA）
- [x] Logic 指令（AND/ORA/EOR/BIT/TRB/TSB）
- [x] Shift/Rotate 指令（ASL/LSR/ROL/ROR）
- [x] ADC/SBC（二进制模式，NMOS 6502 D flag 无效）
- [x] BRK（软件中断）
- [x] Klaus2m5 功能测试 ROM（6502_functional_test.bin）
- [x] JMP (ind) NMOS 页边界 bug
- [x] 跨页检测（absolute_x/y、post_indexed_y 跨页 +1 周期）
- [x] 分支精确周期（未跳转 2，同页 3，跨页 4）
- [x] NMOS 6502 非法指令（SST 测试 247/256 通过，96.5%）
  - [x] LAX, SAX, DCP, ISC, SLO/RLA/SRE/RRA
  - [x] ANC, ALR, ARR, XAA, LAS, AHX, TAS, SHY, SHX
  - [x] KIL（CPU 锁死，PC 不前进）
  - [x] 所有 NOP 变体
- [x] CMOS 指令映射为 NMOS 行为（STZ/BRA/PHX/PHY/PLX/PLY/INC A/DEC A → NOP）

### 交互式监视器
- [x] 反汇编器（lookup 表 + disassemble_at）
- [x] 命令解析和分发
- [x] 单步执行 + 寄存器显示
- [x] 继续执行 + 断点检测
- [x] 寄存器查看 + flags 显示
- [x] 反汇编、内存转储、断点管理、执行历史
- [x] 空回车重复上一条命令
- [x] main.rs 集成（--debug 参数）

### NES 系统
- [x] iNES 卡带解析器（header flags, PRG ROM, CHR ROM, mirroring）
- [x] NES Bus 地址路由（$0000-$1FFF RAM, $2000-$3FFF PPU, $4014 DMA, $6000-$7FFF PRG RAM, $8000-$FFFF ROM）
- [x] PPU 寄存器接口（$2000-$2007）+ loopy 地址系统（v/t/x/w）
- [x] PPU 内存读写（pattern tables, nametables, palette）
- [x] OAM DMA（$4014 端口，513/514 周期惩罚）
- [x] NMI 中断处理（$FFFA-$FFFB 向量）
- [x] 扫描线渲染循环（262 扫描线/帧）
- [x] 背景渲染（nametable → attribute → pattern → palette）
- [x] 精灵渲染（OAM 评估 → flip → priority）
- [x] 帧缓冲输出（256×240 RGB）
- [x] NMI 时序（scanline 241 触发，scanline 262 清除）
- [x] Mapper trait + NROM（mapper 0）
- [x] MMC1（mapper 1）— bank switching + 可切换镜像
- [x] 游戏手柄输入（$4016/$4017，键盘映射）

### APU
- [x] APU 框架（frame counter 4-step/5-step, status register $4015, mixing）
- [x] 矩形波通道 Pulse 1/2（duty cycle, envelope, sweep, length counter）
- [x] 三角波通道 Triangle（linear counter, length counter）
- [x] 噪声通道 Noise（mode flag, LFSR, length counter）
- [x] 音频采样输出（44100 Hz, 采样混合）
- [x] SDL2 音频播放（AudioQueue）

### 系统集成
- [x] NES 主循环（CPU + PPU + APU 同步运行，60 FPS 帧率限制）
- [x] SDL2 实时显示窗口（3x 缩放，Esc 退出）
- [x] 离线 PPM 渲染器（nes_render）
- [x] .nes 文件加载（iNES 格式）

## 待实现

### PPU 精确渲染
- [ ] Sprite 0 Hit 检测
- [ ] 精灵溢出检测
- [ ] 扫描线级渲染（当前为帧级 tile 渲染，非逐扫描线）
- [ ] 滚动实现（coarse/fine X/Y + nametable 切换在渲染中生效）

### APU
- [ ] DMC 通道（Delta Modulation）

### CPU 精确行为
- [ ] 9 个不稳定 illegal opcode（行为因 CPU 版本而异，SST 无法稳定测试）
  - XAA（0x8B）, AHX（0x93, 0x9F）, TAS（0x9B）
  - SHY（0x9C）, SHX（0x9E）, LAX #（0xAB）
  - AXS/CMP（0xCB）, SBC #（0xEB）
