# TODO

## 已完成
- [x] 项目结构（main.rs, cpu.rs, bus/, opcodes.rs, addressing.rs, instructions.rs）
- [x] Bus trait 定义
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
- [x] NMOS 6502 非法指令（SST 测试 247/256 通过，96.5%）
  - [x] LAX（0xA3, 0xA7, 0xAB, 0xAF, 0xB3, 0xB7, 0xBF）
  - [x] SAX（0x83, 0x87, 0x8F, 0x97）
  - [x] DCP（0xC3, 0xC7, 0xCF, 0xD3, 0xD7, 0xDB, 0xDF）
  - [x] ISC（0xE3, 0xE7, 0xEF, 0xF3, 0xF7, 0xFB, 0xFF）
  - [x] SLO/RLA/SRE/RRA 全系列
  - [x] ANC, ALR, ARR, XAA, LAS
  - [x] AHX, TAS, SHY, SHX
  - [x] KIL（CPU 锁死，PC 不前进）
  - [x] 所有 NOP 变体
- [x] CMOS 指令映射为 NMOS 行为（STZ/BRA/PHX/PHY/PLX/PLY/INC A/DEC A → NOP）
- [x] 交互式监视器（monitor/）
  - [x] 反汇编器（lookup 表 + disassemble_at）
  - [x] 命令解析（parse）和分发（execute）
  - [x] 单步执行（step）+ 寄存器显示
  - [x] 继续执行（continue）+ 断点检测
  - [x] 寄存器查看（regs）+ flags 显示
  - [x] 反汇编命令（disassemble）
  - [x] 内存转储（hexdump）
  - [x] 断点管理（break/clear/list，按编号删除）
  - [x] 执行历史（trace）
  - [x] 空回车重复上一条命令
  - [x] main.rs 集成（--debug 参数）

## 待实现

### SST 未通过的 opcode（9 个，行为因 CPU 版本而异）
- [ ] XAA（0x8B）— 不稳定，行为因 CPU 版本而异
- [ ] AHX（0x93, 0x9F）— H 值不稳定
- [ ] TAS（0x9B）— H 值不稳定
- [ ] SHY（0x9C）— 地址计算因版本而异
- [ ] SHX（0x9E）— 地址计算因版本而异
- [ ] LAX #（0xAB）— 不稳定，行为因 CPU 版本而异
- [ ] AXS/CMP（0xCB）— 需要调整标志位
- [ ] SBC #（0xEB）— 替代编码，需要验证标志位

### 精确周期模拟
- [ ] 跨页检测（absolute_x/y 跨页 +1 周期）
- [ ] 分支跳转成功 +1 周期
- [ ] 所有指令精确周期表
