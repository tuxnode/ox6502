# TODO

## 已完成
- [x] 项目结构（main.rs, cpu.rs, bus/, opcodes.rs, addressing.rs, instructions.rs）
- [x] Bus trait 定义
- [x] SimpleBus 实现（64KB 内存 + load 方法）
- [x] CPU 结构体（寄存器 A/X/Y/SP/PC/Status）
- [x] 状态寄存器 flag 常量（FLAG_C/Z/I/D/B/V/N）
- [x] 构造函数 new()（读取复位向量 $FFFC）
- [x] set_flag / get_flag / update_nz
- [x] fetch / fetch_u16
- [x] read / write（委托给 Bus）
- [x] 周期计数器 cycles
- [x] step() 返回周期数
- [x] run() 自动累加周期
- [x] Opcodes 常量表（65C02 全部指令）
- [x] 基础寻址模式（immediate, zeropage, zeropage_x, zeropage_y, absolute, absolute_x）

## 待实现

### 寻址模式
- [ ] absolute_y
- [ ] indirect（JMP 间接）
- [ ] indirect_x（零页间接变址 X）
- [ ] indirect_y（零页间接变址 Y）
- [ ] relative（分支相对寻址）
- [ ] accumulator（累加器模式）

### 栈操作
- [ ] push（入栈）
- [ ] pop（出栈）

### 指令实现
- [ ] NOP（已完成，周期=2）
- [ ] LDA（全部寻址模式）
- [ ] LDX / LDY
- [ ] STA / STX / STY / STZ
- [ ] TAX / TAY / TXA / TYA / TSX / TXS
- [ ] ADC / SBC（含 BCD 模式）
- [ ] AND / ORA / EOR
- [ ] CMP / CPX / CPY
- [ ] INC / DEC / INX / INY / DEX / DEY
- [ ] ASL / LSR / ROL / ROR
- [ ] BIT / TRB / TSB
- [ ] BCC / BCS / BEQ / BNE / BMI / BPL / BVC / BVS / BRA
- [ ] JMP / JSR / RTS / RTI
- [ ] PHA / PLA / PHP / PLP / PHX / PLX / PHY / PLY
- [ ] CLC / CLD / CLI / CLV / SEC / SED / SEI
- [ ] BRK（中断）
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
