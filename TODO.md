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

## 待实现

### 算术指令
- [ ] ADC（加法，含 BCD 模式）
- [ ] SBC（减法，含 BCD 模式）

### 系统指令
- [ ] BRK（软件中断）
- [ ] WAI / STP（CMOS 特有）

### 测试
- [ ] 加载 Klaus2m5 功能测试 ROM
- [ ] 运行 6502_functional_test
- [ ] 运行 65C02_extended_opcodes_test
- [ ] 运行 6502_decimal_test

### 精确周期模拟
- [ ] 跨页检测（absolute_x/y 跨页 +1 周期）
- [ ] 分支跳转成功 +1 周期
- [ ] 所有指令精确周期表
