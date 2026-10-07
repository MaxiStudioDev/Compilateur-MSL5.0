use std::env;
use std::fs;
use std::process;
use std::collections::HashMap;

const OP_ADD:    u16 = 0x001;
const OP_SUB:    u16 = 0x003;
const OP_OR:     u16 = 0x005;
const OP_AND:    u16 = 0x007;
const OP_XOR:    u16 = 0x009;
const OP_AND1B:  u16 = 0x00B;
const OP_FLAG:   u16 = 0x00D;
const OP_FLAG_GT:u16 = 0x00D | (1 << 9); // 0x40D — bit 10 = 1 pour GT/LT
const OP_LATCH:  u16 = 0x00F;
const OP_JMP:    u16 = 0x02D;
const OP_NOP:    u16 = 0x011;
const OP_STOP:   u16 = 0x031;
const OP_IN:     u16 = 0x015;
const OP_OUT:    u16 = 0x013;
const OP_INFLAG: u16 = 0x020;
const OP_GT:     u16 = 0b001000010111;
const OP_REG:    u16 = 0b000000001111;

const READ_BIT: u16 = 1 << 8;
const SM_BIT:   u16 = 1 << 10;

const SHIFT_OP: u32 = 20;
const SHIFT_D1: u32 = 10;

const REG_READ:        u8 = 0b100;
const REG_READ_VAR:    u8 = 0b110;
const REG_READ_VAR_D2: u8 = 0b111;

fn pin_index_to_bits(index: u8, line_num: usize) -> Result<u8, String> {
    match index {
        1 => Ok(0b001),
        2 => Ok(0b011),
        3 => Ok(0b101),
        4 => Ok(0b111),
        _ => Err(format!(
            "Ligne {}: Index de pin invalide '{}' (1, 2, 3, 4)",
            line_num, index
        )),
    }
}

fn encode_reg_index(i: u8) -> u8 {
    (i << 1) | 1
}

fn encode_reg_targeted(
    index: u8,
    rs:    u8,
    w:     bool,
    si:    bool,
    sm:    bool,
    rir:   bool,
) -> u32 {
    let enc_idx = encode_reg_index(index);
      (enc_idx as u32 & 0x3F)
    | ((rs  as u32 & 0x7) << 6)
    | ((w   as u32)        << 9)
    | ((si  as u32)        << 10)
    | ((sm  as u32)        << 11)
    | ((rir as u32)        << 12)
}

fn encode_reg_global(mode_bits: u8) -> u16 {
    let decoder: u16 = 0b01111;
    let bits = (mode_bits as u16 & 0x7) << 6;
    bits | decoder
}

struct RegTracker {
    vars:      HashMap<String, (u8, u32)>,
    next_free: u8,
    free_list: Vec<u8>,
}

impl RegTracker {
    fn new() -> Self {
        Self {
            vars:      HashMap::new(),
            next_free: 0,
            free_list: Vec::new(),
        }
    }

    fn alloc_index(&mut self) -> Result<u8, String> {
        if let Some(idx) = self.free_list.pop() {
            return Ok(idx);
        }
        if self.next_free >= 30 {
            return Err("Trop de variables — max 30 registres".to_string());
        }
        let idx = self.next_free;
        self.next_free += 1;
        Ok(idx)
    }

    fn set(&mut self, name: &str, val: u32) -> Result<u8, String> {
        if let Some(entry) = self.vars.get_mut(name) {
            entry.1 = val;
            return Ok(entry.0);
        }
        let idx = self.alloc_index()?;
        self.vars.insert(name.to_string(), (idx, val));
        Ok(idx)
    }

    fn get_index(&self, name: &str) -> Option<u8> {
        self.vars.get(name).map(|(i, _)| *i)
    }

    fn get_value(&self, name: &str) -> Option<u32> {
        self.vars.get(name).map(|(_, v)| *v)
    }

    fn is_var(&self, tok: &str) -> bool {
        self.vars.contains_key(tok)
    }

    fn resolve_literal(&self, tok: &str, line_num: usize) -> Result<u32, String> {
        if let Ok(v) = tok.parse::<u32>() {
            return Ok(v);
        }
        self.get_value(tok).ok_or_else(|| {
            format!("Ligne {}: Variable inconnue '{}'", line_num, tok)
        })
    }
}

struct LatchState { pins: [bool; 5] }

impl LatchState {
    fn new() -> Self { Self { pins: [false; 5] } }
    fn set(&mut self, pin: u8, val: bool) {
        if pin >= 1 && pin <= 4 { self.pins[pin as usize] = val; }
    }
    fn get(&self, pin: u8) -> bool {
        if pin >= 1 && pin <= 4 { self.pins[pin as usize] } else { false }
    }
}

struct HexInstruction { op: u16, d1: u32, d2: u32 }

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockType {
    IfBlock(usize),
    WhileBlock(usize, usize),
    LoopBlock(usize),
    InIfBlock(usize, bool),
    GtBlock(usize),
}

fn pack(op: u16, d1: u16, d2: u16) -> u32 {
    ((op as u32 & 0xFFF) << SHIFT_OP) |
    ((d1 as u32 & 0x3FF) << SHIFT_D1) |
    (d2  as u32 & 0x3FF)
}

fn patch_jump_targets(
    binary: &mut [u32], hex_instructions: &mut [HexInstruction],
    idx: usize, true_val: u32, false_val: u32,
) {
    hex_instructions[idx].d1 = true_val;
    hex_instructions[idx].d2 = false_val;
    let op = (binary[idx] >> SHIFT_OP) & 0xFFF;
    binary[idx] = ((op as u32) << SHIFT_OP)
                | ((true_val  & 0x3FF) << SHIFT_D1)
                | (false_val  & 0x3FF);
}

fn patch_d1_target(
    binary: &mut [u32], hex_instructions: &mut [HexInstruction],
    idx: usize, target_val: u32,
) {
    hex_instructions[idx].d1 = target_val;
    let op = (binary[idx] >> SHIFT_OP) & 0xFFF;
    let d2 = binary[idx] & 0x3FF;
    binary[idx] = ((op as u32) << SHIFT_OP)
                | ((target_val & 0x3FF) << SHIFT_D1)
                | d2;
}

fn emit_reg_write_d1(
    binary:  &mut Vec<u32>,
    hex_out: &mut Vec<HexInstruction>,
    index:   u8,
    value:   u32,
) {
    let targeted = encode_reg_targeted(index, 0b001, true, true, false, false);
    binary.push(pack(OP_REG, 0, 0));
    hex_out.push(HexInstruction { op: OP_REG, d1: targeted, d2: value });
}

fn emit_reg_read(
    binary:    &mut Vec<u32>,
    hex_out:   &mut Vec<HexInstruction>,
    index:     u8,
    mode_bits: u8,
) {
    let global_op = encode_reg_global(mode_bits);
    let targeted  = encode_reg_targeted(index, 0b001, false, false, false, false);
    binary.push(pack(global_op, 0, 0));
    hex_out.push(HexInstruction { op: global_op, d1: targeted, d2: 0 });
}

fn emit_alu_with_var(
    binary:  &mut Vec<u32>,
    hex_out: &mut Vec<HexInstruction>,
    alu_op:  u16,
    num_val: u32,
) {
    let op = alu_op | READ_BIT | SM_BIT;
    binary.push(pack(op, 0, 0));
    hex_out.push(HexInstruction { op, d1: 0, d2: num_val });
}

#[derive(Debug)]
enum GpioCmd {
    In(u8),
    Out { pin: u8, val: u8, impulse: bool, cut_all: bool },
    IfIn { pin: u8, expected: bool },
}

fn extract_parens(s: &str) -> Option<&str> {
    let start = s.find('(')?;
    let end   = s.find(')')?;
    if end > start { Some(&s[start+1..end]) } else { None }
}

fn try_parse_gpio(raw_line: &str, line_num: usize) -> Result<Option<GpioCmd>, String> {
    let trimmed = raw_line.trim();
    let upper   = trimmed.to_uppercase();

    if upper.starts_with("IF IN(") {
        let after_if = trimmed[3..].trim();
        let pin_str  = extract_parens(after_if).ok_or_else(|| {
            format!("Ligne {}: Syntaxe invalide pour 'if In(...)'", line_num)
        })?;
        let pin = pin_str.trim().parse::<u8>().map_err(|_| {
            format!("Ligne {}: Index pin invalide dans if In()", line_num)
        })?;
        let upper_after = after_if.to_uppercase();
        let expected = if upper_after.contains("== TRUE") { true }
                       else if upper_after.contains("== FALSE") { false }
                       else {
                           return Err(format!(
                               "Ligne {}: 'if In()' doit être suivi de '== true' ou '== false'",
                               line_num
                           ));
                       };
        return Ok(Some(GpioCmd::IfIn { pin, expected }));
    }

    if upper.starts_with("IN(") {
        let pin_str = extract_parens(trimmed).ok_or_else(|| {
            format!("Ligne {}: Syntaxe invalide pour 'In(...)'", line_num)
        })?;
        let pin = pin_str.trim().parse::<u8>().map_err(|_| {
            format!("Ligne {}: Index pin invalide dans In()", line_num)
        })?;
        return Ok(Some(GpioCmd::In(pin)));
    }

    if upper.starts_with("OUT(") {
        let paren_end = trimmed.find(')').ok_or_else(|| {
            format!("Ligne {}: ')' manquant dans Out()", line_num)
        })?;
        let pin_str = &trimmed[4..paren_end];
        let pin = pin_str.trim().parse::<u8>().map_err(|_| {
            format!("Ligne {}: Index pin invalide dans Out()", line_num)
        })?;

        if pin == 0 {
            let rest_upper = trimmed[paren_end+1..].trim().to_uppercase();
            if rest_upper.starts_with(".CUTALL(") {
                return Ok(Some(GpioCmd::Out {
                    pin: 0, val: 0, impulse: false, cut_all: true,
                }));
            } else {
                return Err(format!("Ligne {}: Out(0) réservé pour .CutAll()", line_num));
            }
        }
        if pin < 1 || pin > 4 {
            return Err(format!("Ligne {}: Index pin invalide '{}' (1-4)", line_num, pin));
        }

        let rest       = &trimmed[paren_end+1..];
        let rest_upper = rest.trim().to_uppercase();
        let mut cursor = rest_upper.as_str();

        let mut val: u8 = 1;
        if cursor.starts_with(".SET(") {
            let after = &cursor[5..];
            let close = after.find(')').ok_or_else(|| {
                format!("Ligne {}: ')' manquant après .Set()", line_num)
            })?;
            val = after[..close].trim().parse::<u8>().map_err(|_| {
                format!("Ligne {}: Valeur invalide dans .Set()", line_num)
            })?;
            if val > 1 {
                return Err(format!("Ligne {}: .Set() accepte 0 ou 1", line_num));
            }
            cursor = after[close+1..].trim_start();
        }

        let mut mode: u8 = 2;
        if cursor.starts_with(".SETMODE(") {
            let after_mode = &cursor[9..];
            let close = after_mode.find(')').ok_or_else(|| {
                format!("Ligne {}: ')' manquant après .SetMode()", line_num)
            })?;
            mode = after_mode[..close].trim().parse::<u8>().map_err(|_| {
                format!("Ligne {}: Mode invalide dans .SetMode()", line_num)
            })?;
            if mode != 1 && mode != 2 {
                return Err(format!("Ligne {}: .SetMode() accepte 1 ou 2", line_num));
            }
        }

        return Ok(Some(GpioCmd::Out {
            pin, val, impulse: mode == 2, cut_all: false,
        }));
    }

    Ok(None)
}

fn unroll_for_loops(
    instructions: Vec<(usize, Vec<String>)>,
) -> Result<Vec<(usize, Vec<String>)>, String> {
    let mut current = instructions;
    loop {
        let mut loop_idx = None;
        for i in (0..current.len()).rev() {
            let op = &current[i].1[0];
            if op == "FOR" || (op == "LOOP" && current[i].1.len() > 1) {
                loop_idx = Some(i); break;
            }
        }
        let idx = match loop_idx { Some(i) => i, None => break };

        let mut brace_count = 1;
        let mut end_idx = None;
        for j in (idx+1)..current.len() {
            let op = &current[j].1[0];
            if matches!(op.as_str(), "IF"|"WHILE"|"LOOP"|"FOR"|"GT") {
                brace_count += 1;
            } else if op == "}" {
                brace_count -= 1;
                if brace_count == 0 { end_idx = Some(j); break; }
            }
        }

        let e_idx = match end_idx {
            Some(j) => j,
            None => return Err(format!(
                "Ligne {}: Bloc '{}' non fermé par '}}'",
                current[idx].0, current[idx].1[0]
            )),
        };

        let op_str = current[idx].1[0].clone();
        let count = if op_str == "LOOP" {
            if current[idx].1.len() != 2 {
                return Err(format!("Ligne {}: Syntaxe LOOP incorrecte", current[idx].0));
            }
            current[idx].1[1].parse::<usize>().map_err(|_| {
                format!("Ligne {}: Nombre invalide", current[idx].0)
            })?
        } else {
            let tlen = current[idx].1.len();
            let range_str = if tlen == 4 && current[idx].1[2] == "IN" {
                current[idx].1[3].clone()
            } else if tlen == 2 {
                current[idx].1[1].clone()
            } else {
                return Err(format!("Ligne {}: Syntaxe FOR incorrecte", current[idx].0));
            };

            if range_str.contains("..") {
                let parts: Vec<&str> = range_str.split("..").collect();
                if parts.len() != 2 {
                    return Err(format!("Ligne {}: Plage FOR invalide", current[idx].0));
                }
                let start = parts[0].parse::<usize>().map_err(|_| {
                    format!("Ligne {}: Début plage invalide", current[idx].0)
                })?;
                let end = parts[1].parse::<usize>().map_err(|_| {
                    format!("Ligne {}: Fin plage invalide", current[idx].0)
                })?;
                if end < start { 0 } else { end - start }
            } else {
                range_str.parse::<usize>().map_err(|_| {
                    format!("Ligne {}: Nombre invalide", current[idx].0)
                })?
            }
        };

        let body = current[(idx+1)..e_idx].to_vec();
        let mut unrolled = Vec::new();
        for _ in 0..count { unrolled.extend(body.iter().cloned()); }
        current.splice(idx..=e_idx, unrolled);
    }
    Ok(current)
}

#[derive(Debug)]
struct LetExpr {
    name:     String,
    val1:     u32,
    val2:     u32,
    alu:      Option<u16>,
    from_var: Option<String>,
}

fn try_parse_let(
    tokens:   &[String],
    line_num: usize,
    reg:      &RegTracker,
) -> Result<Option<LetExpr>, String> {

    if tokens[0] != "LET" { return Ok(None); }
    if tokens.len() < 4 || tokens[2] != "=" {
        return Err(format!(
            "Ligne {}: Syntaxe let incorrecte. Exemple: let x = 42", line_num
        ));
    }

    let name = tokens[1].to_lowercase();

    if tokens.len() == 4 {
        let tok = tokens[3].to_lowercase();
        if let Ok(v) = tok.parse::<u32>() {
            return Ok(Some(LetExpr {
                name, val1: v, val2: 0, alu: None, from_var: None,
            }));
        }
        if let Some(val) = reg.get_value(&tok) {
            return Ok(Some(LetExpr {
                name, val1: val, val2: 0, alu: None, from_var: Some(tok),
            }));
        }
        return Err(format!("Ligne {}: Variable inconnue '{}' dans let", line_num, tok));
    }

    if tokens.len() == 6 {
        let v1 = reg.resolve_literal(&tokens[3].to_lowercase(), line_num)?;
        let v2 = reg.resolve_literal(&tokens[5].to_lowercase(), line_num)?;
        let alu = match tokens[4].as_str() {
            "+" => OP_ADD,
            "-" => OP_SUB,
            "|" => OP_OR,
            "&" => OP_AND,
            "^" => OP_XOR,
            op  => return Err(format!("Ligne {}: Opérateur inconnu '{}'", line_num, op)),
        };
        return Ok(Some(LetExpr {
            name, val1: v1, val2: v2, alu: Some(alu), from_var: None,
        }));
    }

    Err(format!("Ligne {}: Syntaxe let incorrecte", line_num))
}

fn compile(source: &str) -> Result<(Vec<u32>, Vec<HexInstruction>), String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut clean_instructions: Vec<(usize, Vec<String>)> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num   = line_idx + 1;
        let clean_line = line
            .split('#').next().unwrap()
            .split("//").next().unwrap()
            .trim();
        if clean_line.is_empty() { continue; }
        if clean_line.to_uppercase().starts_with("FN ") { continue; }

        if let Some(cmd) = try_parse_gpio(clean_line, line_num)? {
            match cmd {
                GpioCmd::In(pin) => {
                    let pb = pin_index_to_bits(pin, line_num)?;
                    clean_instructions.push((line_num,
                        vec!["__IN__".to_string(), pb.to_string()]));
                }
                GpioCmd::Out { pin, val, impulse, cut_all } => {
                    clean_instructions.push((line_num, vec![
                        "__OUT__".to_string(), pin.to_string(), val.to_string(),
                        (impulse as u8).to_string(), (cut_all as u8).to_string(),
                    ]));
                }
                GpioCmd::IfIn { pin, expected } => {
                    let pb = pin_index_to_bits(pin, line_num)?;
                    clean_instructions.push((line_num, vec![
                        "__IFIN__".to_string(), pb.to_string(),
                        (expected as u8).to_string(),
                    ]));
                }
            }
            continue;
        }

        let standardized = clean_line
            .replace('(', " ").replace(')', " ")
            .replace(',', " ")
            .replace('{', " ")
            .replace('}', " } ")
            .replace("==", " == ")
            .replace("<=", " <= ")
            .replace(">=", " >= ")
            .replace('<', " < ")
            .replace('>', " > ")
            .replace('+', " + ")
            .replace('-', " - ")
            .replace('&', " & ")
            .replace('|', " | ")
            .replace('^', " ^ ");

        let mut tokens: Vec<String> = standardized
            .split_whitespace()
            .map(|t| t.to_uppercase())
            .collect();

        if !tokens.is_empty() {
            let op_str = tokens[0].clone();
            if tokens.len() >= 4
                && matches!(op_str.as_str(), "ADD"|"SUB"|"AND"|"OR"|"XOR")
                && matches!(tokens[2].as_str(), "+"|"-"|"&"|"|"|"^")
            {
                tokens.remove(2);
            }
            clean_instructions.push((line_num, tokens));
        }
    }

    clean_instructions = unroll_for_loops(clean_instructions)?;

    let mut binary:      Vec<u32>            = Vec::new();
    let mut hex_out:     Vec<HexInstruction> = Vec::new();
    let mut reg_locked                       = false;
    let mut block_stack: Vec<BlockType>      = Vec::new();
    let mut break_stack: Vec<Vec<usize>>     = Vec::new();
    let mut latch_state                      = LatchState::new();
    let mut reg_tracker                      = RegTracker::new();

    for (line_num, tokens) in &clean_instructions {
        let op_str = &tokens[0];

        if op_str == "__IN__" {
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }
            let d1 = tokens[1].parse::<u16>().unwrap();
            binary.push(pack(OP_IN, d1, 0));
            hex_out.push(HexInstruction { op: OP_IN, d1: d1 as u32, d2: 0 });
            continue;
        }

        if op_str == "__IFIN__" {
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }
            let pin_bits = tokens[1].parse::<u16>().unwrap();
            let expected = tokens[2].parse::<u8>().unwrap() == 1;
            binary.push(pack(OP_IN, pin_bits, 0));
            hex_out.push(HexInstruction { op: OP_IN, d1: pin_bits as u32, d2: 0 });
            let flag_index = binary.len();
            block_stack.push(BlockType::InIfBlock(flag_index, !expected));
            binary.push(pack(OP_INFLAG, 0, 0));
            hex_out.push(HexInstruction { op: OP_INFLAG, d1: 0, d2: 0 });
            continue;
        }

        if op_str == "__OUT__" {
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }
            let pin            = tokens[1].parse::<u8>().unwrap();
            let val            = tokens[2].parse::<u8>().unwrap();
            let impulse        = tokens[3].parse::<u8>().unwrap() == 1;
            let cut_all_forced = tokens[4].parse::<u8>().unwrap() == 1;

            if pin == 0 && cut_all_forced {
                let d1: u8 = 0b100000;
                binary.push(pack(OP_OUT, d1 as u16, 0));
                hex_out.push(HexInstruction { op: OP_OUT, d1: d1 as u32, d2: 0 });
                continue;
            }

            let need_cut = impulse && latch_state.get(pin);
            if !impulse { latch_state.set(pin, val == 1); }

            let pin_bits = pin_index_to_bits(pin, *line_num)?;
            let bit_cut  = (need_cut || cut_all_forced) as u8;
            let d1: u8   = pin_bits
                         | (val            << 3)
                         | ((impulse as u8) << 4)
                         | (bit_cut         << 5);

            binary.push(pack(OP_OUT, d1 as u16, 0));
            hex_out.push(HexInstruction { op: OP_OUT, d1: d1 as u32, d2: 0 });
            continue;
        }

        if op_str == "LET" {
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }

            let expr = try_parse_let(tokens, *line_num, &reg_tracker)?
                .ok_or_else(|| format!("Ligne {}: Syntaxe let invalide", line_num))?;

            let final_val = if let Some(alu_op) = expr.alu {
                match alu_op {
                    OP_ADD => expr.val1.wrapping_add(expr.val2),
                    OP_SUB => expr.val1.wrapping_sub(expr.val2),
                    OP_OR  => expr.val1 | expr.val2,
                    OP_AND => expr.val1 & expr.val2,
                    OP_XOR => expr.val1 ^ expr.val2,
                    _      => expr.val1,
                }
            } else {
                expr.val1
            };

            let idx = reg_tracker.set(&expr.name, final_val)?;

            if let Some(ref src_name) = expr.from_var {
                let src_idx = reg_tracker.get_index(src_name)
                    .ok_or_else(|| format!(
                        "Ligne {}: Variable source inconnue '{}'", line_num, src_name
                    ))?;
                emit_reg_read(&mut binary, &mut hex_out, src_idx, REG_READ);
            }

            emit_reg_write_d1(&mut binary, &mut hex_out, idx, expr.val1);

            if expr.val2 != 0 {
                let targeted_d2 = encode_reg_targeted(idx, 0b010, true, false, false, false);
                binary.push(pack(OP_REG, 0, 0));
                hex_out.push(HexInstruction { op: OP_REG, d1: targeted_d2, d2: expr.val2 });
            }

            continue;
        }

        if op_str == "REG" {
            if tokens.len() < 3 || tokens[1] != "READ" {
                return Err(format!(
                    "Ligne {}: Syntaxe incorrecte. Exemple: reg read x", line_num
                ));
            }
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }

            let var_name = tokens[2].to_lowercase();
            let idx = reg_tracker.get_index(&var_name)
                .ok_or_else(|| format!(
                    "Ligne {}: Variable inconnue '{}' dans reg read", line_num, var_name
                ))?;

            let mode_bits = if tokens.len() >= 4 {
                match tokens[3].to_uppercase().as_str() {
                    "VAR"   => REG_READ_VAR,
                    "VARD2" => REG_READ_VAR_D2,
                    _       => REG_READ,
                }
            } else {
                REG_READ
            };

            emit_reg_read(&mut binary, &mut hex_out, idx, mode_bits);
            continue;
        }

        if op_str == "}" {
            if let Some(active_block) = block_stack.pop() {
                match active_block {
                    BlockType::IfBlock(fi) | BlockType::GtBlock(fi) => {
                        let true_dst  = (fi + 1) as u32;
                        let false_dst = binary.len() as u32;
                        patch_jump_targets(&mut binary, &mut hex_out, fi, true_dst, false_dst);
                    }
                    BlockType::InIfBlock(fi, is_inverted) => {
                        let true_dst  = (fi + 1) as u32;
                        let false_dst = binary.len() as u32;
                        if is_inverted {
                            patch_jump_targets(&mut binary, &mut hex_out, fi, false_dst, true_dst);
                        } else {
                            patch_jump_targets(&mut binary, &mut hex_out, fi, true_dst, false_dst);
                        }
                    }
                    BlockType::WhileBlock(start_index, fi) => {
                        binary.push(pack(OP_JMP, start_index as u16, 0));
                        hex_out.push(HexInstruction {
                            op: OP_JMP, d1: start_index as u32, d2: 0,
                        });
                        let true_dst  = (fi + 1) as u32;
                        let false_dst = binary.len() as u32;
                        patch_jump_targets(&mut binary, &mut hex_out, fi, true_dst, false_dst);
                        if let Some(breaks) = break_stack.pop() {
                            for bi in breaks {
                                patch_d1_target(&mut binary, &mut hex_out, bi, false_dst);
                            }
                        }
                    }
                    BlockType::LoopBlock(start_index) => {
                        binary.push(pack(OP_JMP, start_index as u16, 0));
                        hex_out.push(HexInstruction {
                            op: OP_JMP, d1: start_index as u32, d2: 0,
                        });
                        let false_dst = binary.len() as u32;
                        if let Some(breaks) = break_stack.pop() {
                            for bi in breaks {
                                patch_d1_target(&mut binary, &mut hex_out, bi, false_dst);
                            }
                        }
                    }
                }
            }
            continue;
        }

        if op_str == "LOOP" {
            if tokens.len() != 1 {
                return Err(format!("Ligne {}: Syntaxe LOOP incorrecte", line_num));
            }
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }
            let start_index = binary.len();
            block_stack.push(BlockType::LoopBlock(start_index));
            break_stack.push(Vec::new());
            continue;
        }

        // ── IF ───────────────────────────────────────
        //
        // IF == → OP_AND1B + OP_FLAG      (bit 10 = 0)
        // IF >  → OP_GT   + OP_FLAG_GT    (bit 10 = 1)
        // IF <  → OP_GT inversé + OP_FLAG_GT (bit 10 = 1)
        // ─────────────────────────────────────────────
        if op_str == "IF" {
            if tokens.len() != 4 || !matches!(tokens[2].as_str(), "==" | ">" | "<") {
                return Err(format!(
                    "Ligne {}: Syntaxe IF incorrecte. Exemple: if x == 1 {{", line_num
                ));
            }
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }

            let tok1    = tokens[1].to_lowercase();
            let tok2    = tokens[3].to_lowercase();
            let is_var1 = reg_tracker.is_var(&tok1);
            let is_var2 = reg_tracker.is_var(&tok2);

            if is_var1 && is_var2 {
                return Err(format!(
                    "Ligne {}: IF ne peut pas comparer deux variables directement", line_num
                ));
            }

            if is_var1 {
                let idx = reg_tracker.get_index(&tok1).unwrap();
                emit_reg_read(&mut binary, &mut hex_out, idx, REG_READ);
            }
            if is_var2 {
                let idx = reg_tracker.get_index(&tok2).unwrap();
                emit_reg_read(&mut binary, &mut hex_out, idx, REG_READ);
            }

            let v1 = reg_tracker.resolve_literal(&tok1, *line_num)?;
            let v2 = reg_tracker.resolve_literal(&tok2, *line_num)?;

            match tokens[2].as_str() {
                "==" => {
                    // AND1B + FLAG (bit 10 = 0)
                    binary.push(pack(OP_AND1B, v1 as u16, v2 as u16));
                    hex_out.push(HexInstruction { op: OP_AND1B, d1: v1, d2: v2 });
                    let fi = binary.len();
                    block_stack.push(BlockType::IfBlock(fi));
                    binary.push(pack(OP_FLAG, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG, d1: 0, d2: 0 });
                }
                ">" => {
                    // GT + FLAG_GT (bit 10 = 1)
                    binary.push(pack(OP_GT, v1 as u16, v2 as u16));
                    hex_out.push(HexInstruction { op: OP_GT, d1: v1, d2: v2 });
                    let fi = binary.len();
                    block_stack.push(BlockType::GtBlock(fi));
                    binary.push(pack(OP_FLAG_GT, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG_GT, d1: 0, d2: 0 });
                }
                "<" => {
                    // GT inversé + FLAG_GT (bit 10 = 1)
                    binary.push(pack(OP_GT, v2 as u16, v1 as u16));
                    hex_out.push(HexInstruction { op: OP_GT, d1: v2, d2: v1 });
                    let fi = binary.len();
                    block_stack.push(BlockType::GtBlock(fi));
                    binary.push(pack(OP_FLAG_GT, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG_GT, d1: 0, d2: 0 });
                }
                _ => unreachable!()
            }
            continue;
        }

        // ── WHILE ────────────────────────────────────
        if op_str == "WHILE" {
            if tokens.len() != 4 || !matches!(tokens[2].as_str(), "==" | ">" | "<") {
                return Err(format!("Ligne {}: Syntaxe WHILE incorrecte.", line_num));
            }
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }

            let tok1    = tokens[1].to_lowercase();
            let tok2    = tokens[3].to_lowercase();
            let is_var1 = reg_tracker.is_var(&tok1);
            let is_var2 = reg_tracker.is_var(&tok2);

            if is_var1 && is_var2 {
                return Err(format!(
                    "Ligne {}: WHILE ne peut pas comparer deux variables directement", line_num
                ));
            }

            if is_var1 {
                let idx = reg_tracker.get_index(&tok1).unwrap();
                emit_reg_read(&mut binary, &mut hex_out, idx, REG_READ);
            }
            if is_var2 {
                let idx = reg_tracker.get_index(&tok2).unwrap();
                emit_reg_read(&mut binary, &mut hex_out, idx, REG_READ);
            }

            let v1 = reg_tracker.resolve_literal(&tok1, *line_num)?;
            let v2 = reg_tracker.resolve_literal(&tok2, *line_num)?;
            let start_index = binary.len();

            match tokens[2].as_str() {
                "==" => {
                    binary.push(pack(OP_AND1B, v1 as u16, v2 as u16));
                    hex_out.push(HexInstruction { op: OP_AND1B, d1: v1, d2: v2 });
                    let fi = binary.len();
                    block_stack.push(BlockType::WhileBlock(start_index, fi));
                    binary.push(pack(OP_FLAG, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG, d1: 0, d2: 0 });
                }
                ">" => {
                    binary.push(pack(OP_GT, v1 as u16, v2 as u16));
                    hex_out.push(HexInstruction { op: OP_GT, d1: v1, d2: v2 });
                    let fi = binary.len();
                    block_stack.push(BlockType::WhileBlock(start_index, fi));
                    binary.push(pack(OP_FLAG_GT, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG_GT, d1: 0, d2: 0 });
                }
                "<" => {
                    binary.push(pack(OP_GT, v2 as u16, v1 as u16));
                    hex_out.push(HexInstruction { op: OP_GT, d1: v2, d2: v1 });
                    let fi = binary.len();
                    block_stack.push(BlockType::WhileBlock(start_index, fi));
                    binary.push(pack(OP_FLAG_GT, 0, 0));
                    hex_out.push(HexInstruction { op: OP_FLAG_GT, d1: 0, d2: 0 });
                }
                _ => unreachable!()
            }

            break_stack.push(Vec::new());
            continue;
        }

        if op_str == "BREAK" {
            if tokens.len() != 1 {
                return Err(format!("Ligne {}: BREAK ne prend pas d'arguments", line_num));
            }
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé", line_num)); }
            let in_loop = block_stack.iter().any(|b| {
                matches!(b, BlockType::WhileBlock(_, _) | BlockType::LoopBlock(_))
            });
            if !in_loop || break_stack.is_empty() {
                return Err(format!("Ligne {}: BREAK en dehors de toute boucle", line_num));
            }
            let break_jmp_idx = binary.len();
            if let Some(list) = break_stack.last_mut() { list.push(break_jmp_idx); }
            binary.push(pack(OP_JMP, 0, 0));
            hex_out.push(HexInstruction { op: OP_JMP, d1: 0, d2: 0 });
            continue;
        }

        if op_str == "LATCH" {
            if tokens.len() != 1 {
                return Err(format!("Ligne {}: LATCH ne prend pas d'arguments", line_num));
            }
            reg_locked = true;
            binary.push(pack(OP_LATCH, 0, 0));
            hex_out.push(HexInstruction { op: OP_LATCH, d1: 0, d2: 0 });
            continue;
        }

        if op_str == "RESET" {
            if tokens.len() != 1 {
                return Err(format!("Ligne {}: RESET ne prend pas d'arguments", line_num));
            }
            reg_locked = false;
            binary.push(pack(OP_LATCH, 0, 0));
            hex_out.push(HexInstruction { op: OP_LATCH, d1: 0, d2: 0 });
            continue;
        }

        if reg_locked {
            return Err(format!("Ligne {}: Instruction bloquée ! LATCH actif.", line_num));
        }

        let op_code: u16 = match op_str.as_str() {
            "ADD"   => OP_ADD,
            "SUB"   => OP_SUB,
            "OR"    => OP_OR,
            "AND"   => OP_AND,
            "XOR"   => OP_XOR,
            "AND1B" => OP_AND1B,
            "FLAG"  => OP_FLAG,
            "JMP"   => OP_JMP,
            "NOP"   => OP_NOP,
            "STOP"  => OP_STOP,
            _ => return Err(format!("Ligne {}: Opcode inconnu '{}'", line_num, op_str)),
        };

        if op_code == OP_STOP || op_code == OP_NOP {
            if tokens.len() != 1 {
                return Err(format!(
                    "Ligne {}: '{}' ne prend aucun argument", line_num, op_str
                ));
            }
            binary.push(pack(op_code, 0, 0));
            hex_out.push(HexInstruction { op: op_code, d1: 0, d2: 0 });
            continue;
        }

        if tokens.len() != 3 {
            return Err(format!(
                "Ligne {}: '{}' attend 2 arguments", line_num, op_str
            ));
        }

        let tok1    = tokens[1].to_lowercase();
        let tok2    = tokens[2].to_lowercase();
        let is_var1 = reg_tracker.is_var(&tok1);
        let is_var2 = reg_tracker.is_var(&tok2);

        if is_var1 && is_var2 {
            return Err(format!(
                "Ligne {}: Deux variables dans ALU interdites — fusion de bits !",
                line_num
            ));
        }

        if is_var1 || is_var2 {
            let num = if is_var1 {
                tok2.parse::<u32>().map_err(|_| {
                    format!("Ligne {}: Valeur invalide '{}'", line_num, tok2)
                })?
            } else {
                tok1.parse::<u32>().map_err(|_| {
                    format!("Ligne {}: Valeur invalide '{}'", line_num, tok1)
                })?
            };
            emit_alu_with_var(&mut binary, &mut hex_out, op_code, num);
        } else {
            let d1 = tok1.parse::<u32>().map_err(|_| {
                format!("Ligne {}: Valeur invalide '{}'", line_num, tok1)
            })?;
            let d2 = tok2.parse::<u32>().map_err(|_| {
                format!("Ligne {}: Valeur invalide '{}'", line_num, tok2)
            })?;
            binary.push(pack(op_code, d1 as u16, d2 as u16));
            hex_out.push(HexInstruction { op: op_code, d1, d2 });
        }
    }

    if !block_stack.is_empty() {
        return Err("Erreur : blocs non fermés par '}'".into());
    }

    Ok((binary, hex_out))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("🔧 MSL5 Compiler v11.0 — Registre + GT + LT + Variables");
        println!("Usage: cargo run -- input.msl5 output.bin");
        process::exit(1);
    }

    let source = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("❌ Erreur lecture fichier source : {}", e);
        process::exit(1);
    });

    match compile(&source) {
        Ok((binary, hex_out)) => {
            let mut bin_bytes = Vec::new();
            for w in &binary { bin_bytes.extend_from_slice(&w.to_le_bytes()); }
            fs::write(&args[2], &bin_bytes).expect("Échec écriture binaire");

            let mut txt_lines = Vec::new();
            let mut opc_lines = Vec::new();
            let mut d1_lines  = Vec::new();
            let mut d2_lines  = Vec::new();

            for h in &hex_out {
                let op_s = format!("{:03X}", h.op);
                let d1_s = format!("{:08X}", h.d1);
                let d2_s = format!("{:08X}", h.d2);
                txt_lines.push(format!("{}{}{}", op_s, d1_s, d2_s));
                opc_lines.push(op_s);
                d1_lines.push(d1_s);
                d2_lines.push(d2_s);
            }

            let txt_filename = args[2].replace(".bin", ".txt");
            fs::write(&txt_filename, txt_lines.join("\n")).expect("Échec écriture .txt");
            fs::write("OpC", opc_lines.join("\n")).expect("Échec écriture OpC");
            fs::write("D1",  d1_lines.join("\n")).expect("Échec écriture D1");
            fs::write("D2",  d2_lines.join("\n")).expect("Échec écriture D2");

            println!("✅ Compilation réussie");
            println!("📂 Fichiers mis à jour.");
        }
        Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
    }
}