use std::env;
use std::fs;
use std::process;

// ─────────────────────────────────────────────────────
// 📐 OPCODES EXACTS (8 bits)
// ─────────────────────────────────────────────────────
const OP_ADD:    u8 = 0x01;
const OP_SUB:    u8 = 0x03;
const OP_OR:     u8 = 0x05;
const OP_AND:    u8 = 0x07;
const OP_XOR:    u8 = 0x09;
const OP_AND1B:  u8 = 0x0B;
const OP_FLAG:   u8 = 0x0D;
const OP_LATCH:  u8 = 0x0F;
const OP_RESET:  u8 = 0x11;
const OP_JMP:    u8 = 0x13;
const OP_LOAD:   u8 = 0x19;
const OP_STORE:  u8 = 0x1B;
const OP_NOP:    u8 = 0x00;

const SHIFT_OP: u32 = 24;
const SHIFT_D1: u32 = 12;

fn pack(op: u8, d1: u16, d2: u16) -> u32 {
    ((op as u32) << SHIFT_OP) |
    ((d1 as u32 & 0xFFF) << SHIFT_D1) |
    (d2 as u32 & 0xFFF)
}

fn patch_d2(instruction: &mut u32, target_value: u16) {
    *instruction = (*instruction & 0xFFFFF000) | (target_value as u32 & 0xFFF);
}

fn compile(source: &str) -> Result<Vec<u32>, String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut clean_instructions: Vec<(usize, Vec<String>)> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let clean_line = line.split('#').next().unwrap().split("//").next().unwrap().trim();
        if clean_line.is_empty() { continue; }
        
        let tokens: Vec<String> = clean_line.split_whitespace()
            .filter(|t| !matches!(*t, "+" | "-" | "&" | "|" | "^"))
            .map(|t| t.to_uppercase())
            .collect();
            
        if !tokens.is_empty() {
            clean_instructions.push((line_idx + 1, tokens));
        }
    }

    let mut binary: Vec<u32> = Vec::new();
    let mut reg_locked = false;

    let mut if_stack: Vec<usize> = Vec::new();       
    let mut while_stack: Vec<usize> = Vec::new();    
    let mut while_flag_stack: Vec<usize> = Vec::new(); 

    for (line_num, tokens) in &clean_instructions {
        let op_str = &tokens[0];

        if op_str == "IF" {
            if tokens.len() < 3 {
                return Err(format!("Ligne {}: Syntaxe 'IF valeur1 valeur2' incorrecte", line_num));
            }
            let v1 = tokens[1].parse::<u16>().map_err(|_| format!("Ligne {}: Nombre invalide", line_num))?;
            let v2 = tokens[2].parse::<u16>().map_err(|_| format!("Ligne {}: Nombre invalide", line_num))?;
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé (LATCH)", line_num)); }
            
            binary.push(pack(OP_AND1B, v1, v2)); 
            if_stack.push(binary.len());
            binary.push(pack(OP_FLAG, 0, 0)); 
            continue;
        }

        if op_str == "ENDIF" {
            if let Some(flag_index) = if_stack.pop() {
                let target_address = binary.len();
                let offset = (target_address as i32 - flag_index as i32) & 0xFFF;
                patch_d2(&mut binary[flag_index], offset as u16);
            } else {
                return Err(format!("Ligne {}: 'ENDIF' sans 'IF' correspondant", line_num));
            }
            continue;
        }

        if op_str == "WHILE" {
            if tokens.len() < 3 {
                return Err(format!("Ligne {}: Syntaxe 'WHILE valeur1 valeur2' incorrecte", line_num));
            }
            let v1 = tokens[1].parse::<u16>().map_err(|_| format!("Ligne {}: Nombre invalide", line_num))?;
            let v2 = tokens[2].parse::<u16>().map_err(|_| format!("Ligne {}: Nombre invalide", line_num))?;
            if reg_locked { return Err(format!("Ligne {}: Registre verrouillé (LATCH)", line_num)); }
            
            while_stack.push(binary.len());
            binary.push(pack(OP_AND1B, v1, v2));
            while_flag_stack.push(binary.len());
            binary.push(pack(OP_FLAG, 0, 0));
            continue;
        }

        if op_str == "ENDWHILE" {
            if let (Some(start_index), Some(flag_index)) = (while_stack.pop(), while_flag_stack.pop()) {
                let current_idx = binary.len();
                let jump_back_offset = (start_index as i32 - current_idx as i32) & 0xFFF;
                binary.push(pack(OP_JMP, 0, jump_back_offset as u16));
                
                let target_address = binary.len();
                let jump_forward_offset = (target_address as i32 - flag_index as i32) & 0xFFF;
                patch_d2(&mut binary[flag_index], jump_forward_offset as u16);
            } else {
                return Err(format!("Ligne {}: 'ENDWHILE' sans 'WHILE' correspondant", line_num));
            }
            continue;
        }

        if op_str == "RESET" { reg_locked = false; binary.push(pack(OP_RESET, 0, 0)); continue; }
        if op_str == "LATCH" { reg_locked = true; binary.push(pack(OP_LATCH, 0, 0)); continue; }
        if reg_locked { return Err(format!("Ligne {}: Instruction bloquée par LATCH", line_num)); }

        let op_code = match op_str.as_str() {
            "ADD" => OP_ADD, "SUB" => OP_SUB, "OR" => OP_OR, "AND" => OP_AND, "XOR" => OP_XOR,
            "AND1B" => OP_AND1B, "FLAG" => OP_FLAG, "JMP" => OP_JMP, "LOAD" => OP_LOAD, 
            "STORE" => OP_STORE, "NOP" => OP_NOP,
            _ => return Err(format!("Ligne {}: Opcode inconnu '{}'", line_num, op_str)),
        };

        let parse_val = |s: &String| -> Result<u16, String> {
            s.parse::<u16>().map_err(|_| format!("Ligne {}: Nombre invalide '{}'", line_num, s))
        };

        let d1 = if tokens.len() > 1 { parse_val(&tokens[1])? } else { 0 };
        let d2 = if tokens.len() > 2 { parse_val(&tokens[2])? } else { 0 };

        let is_alu = matches!(op_code, OP_ADD | OP_SUB | OP_OR | OP_AND | OP_XOR | OP_AND1B);
        let arg_present = (tokens.len() > 1 && tokens[1] != "0") || (tokens.len() > 2 && tokens[2] != "0");

        if is_alu && arg_present {
            binary.push(pack(OP_LOAD, d1, d2));
            binary.push(pack(op_code, 0, 0));
        } else {
            binary.push(pack(op_code, d1, d2));
        }
    }

    if !if_stack.is_empty() { return Err("Erreur : Un bloc 'IF' n'a pas été fermé par un 'ENDIF'".into()); }
    if !while_stack.is_empty() { return Err("Erreur : Un bloc 'WHILE' n'a pas été fermé par un 'ENDWHILE'".into()); }

    Ok(binary)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("🔧 MSL5 Compiler v5.4 — All-In-One Ultra Fast Mode");
        println!("Usage: cargo run -- input.msl5 output.bin");
        process::exit(1);
    }
    
    let source = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("❌ Erreur lecture fichier source : {}", e);
        process::exit(1);
    });

    match compile(&source) {
        Ok(binary) => {
            // 1. Sauvegarde du binaire brut
            let mut bin_bytes = Vec::new();
            for w in &binary { bin_bytes.extend_from_slice(&w.to_le_bytes()); }
            fs::write(&args[2], &bin_bytes).expect("Échec écriture du binaire");

            // 2. Préparation des flux pour le splitter automatique
            let mut txt_strings = Vec::new();
            let mut opc_content = String::new();
            let mut d1_content = String::new();
            let mut d2_content = String::new();
            
            for w in &binary {
                let op = (w >> 24) & 0xFF;
                let d1 = (w >> 12) & 0xFFF;
                let d2 = w & 0xFFF;

                // Extension de signe automatique
                let d1_32 = if d1 >= 0x800 { d1 | 0xFFFFF000 } else { d1 };
                let d2_32 = if d2 >= 0x800 { d2 | 0xFFFFF000 } else { d2 };

                // Chaînes formatées
                let op_str = format!("{:02X}", op);
                let d1_str = format!("{:08X}", d1_32);
                let d2_str = format!("{:08X}", d2_32);

                // Fichier de debug principal Code.txt
                txt_strings.push(format!("{}{}{}", op_str, d1_str, d2_str));

                // Remplissage des fichiers bruts continus (sans espace ni saut de ligne pour ton CPU)
                opc_content.push_str(&op_str);
                d1_content.push_str(&d1_str);
                d2_content.push_str(&d2_str);
            }
            
            // 3. Écriture immédiate de TOUS les fichiers requis
            let txt_filename = args[2].replace(".bin", ".txt");
            fs::write(&txt_filename, txt_strings.join("\n")).expect("Échec Code.txt");
            fs::write("OpC", opc_content).expect("Échec écriture OpC");
            fs::write("D1", d1_content).expect("Échec écriture D1");
            fs::write("D2", d2_content).expect("Échec écriture D2");

            println!("✅ Compilation et découpe 72-bit terminées en quelques millisecondes !");
            println!("📂 Fichiers générés : {}, {}, OpC, D1, D2", args[2], txt_filename);
        }
        Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
    }
}