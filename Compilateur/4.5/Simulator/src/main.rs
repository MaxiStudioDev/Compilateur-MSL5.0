#![windows_subsystem = "windows"]

use native_windows_gui as nwg;
use std::{cell::RefCell, fs, rc::Rc, time::Instant};

// ═══════════════════════════════════════════
//  OPCODES
// ═══════════════════════════════════════════
const OP_ADD:     u32 = 0x001;
const OP_SUB:     u32 = 0x003;
const OP_OR:      u32 = 0x005;
const OP_AND:     u32 = 0x007;
const OP_XOR:     u32 = 0x009;
const OP_AND1B:   u32 = 0x00B;
const OP_FLAG:    u32 = 0x00D;
const OP_FLAG_GT: u32 = 0x40D;
const OP_LATCH:   u32 = 0x00F;
const OP_JMP:     u32 = 0x02D;
const OP_NOP:     u32 = 0x011;
const OP_STOP:    u32 = 0x031;
const OP_IN:      u32 = 0x015;
const OP_OUT:     u32 = 0x013;
const OP_INFLAG:  u32 = 0x020;
const OP_GT:      u32 = 0x217;

// ═══════════════════════════════════════════
//  INSTRUCTION
// ═══════════════════════════════════════════
#[derive(Clone, Debug)]
struct Instruction { op: u32, d1: u32, d2: u32 }

impl Instruction {
    fn parse(line: &str) -> Option<Self> {
        let l = line.trim();
        if l.len() < 19 { return None; }
        let op = u32::from_str_radix(&l[0..3],   16).ok()?;
        let d1 = u32::from_str_radix(&l[3..11],  16).ok()?;
        let d2 = u32::from_str_radix(&l[11..19], 16).ok()?;
        Some(Self { op, d1, d2 })
    }

    fn mnemonic(&self) -> &'static str {
        if self.op == OP_LATCH {
            return if self.d1 != 0 { "REG" } else { "LATCH" };
        }
        match self.op {
            OP_ADD     => "ADD",    OP_SUB     => "SUB",
            OP_OR      => "OR",     OP_AND     => "AND",
            OP_XOR     => "XOR",    OP_AND1B   => "AND1B",
            OP_FLAG    => "FLAG",   OP_FLAG_GT => "FLAG_GT",
            OP_JMP     => "JMP",    OP_NOP     => "NOP",
            OP_STOP    => "STOP",   OP_IN      => "IN",
            OP_OUT     => "OUT",    OP_INFLAG  => "INFLAG",
            OP_GT      => "GT",     _          => "???",
        }
    }
}

// ═══════════════════════════════════════════
//  CPU
// ═══════════════════════════════════════════
struct Cpu {
    regs:      [u32; 32],
    acc:       u32,
    flag:      bool,
    pc:        usize,
    latch:     bool,
    cycle:     u64,
    running:   bool,
    halted:    bool,
    gpio_in:   [bool; 5],
    gpio_out:  [bool; 5],
    program:   Vec<Instruction>,
    mhz:       f64,
    last_tick: Option<Instant>,
    budget:    f64,
}

impl Cpu {
    fn new() -> Self {
        Self {
            regs: [0;32], acc: 0, flag: false,
            pc: 0, latch: false, cycle: 0,
            running: false, halted: false,
            gpio_in: [false;5], gpio_out: [false;5],
            program: vec![], mhz: 1.0,
            last_tick: None, budget: 0.0,
        }
    }

    fn reset(&mut self) {
        let prog = self.program.clone();
        let mhz  = self.mhz;
        let gin  = self.gpio_in;
        *self    = Self::new();
        self.program = prog;
        self.mhz     = mhz;
        self.gpio_in = gin;
    }

    fn load(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let prog: Vec<Instruction> = content.lines()
            .filter_map(Instruction::parse).collect();
        if prog.is_empty() { return Err("Aucune instruction trouvée".into()); }
        self.program = prog;
        self.reset();
        Ok(())
    }

    fn pin_bits_to_index(b: u8) -> u8 {
        match b & 7 { 1=>1, 3=>2, 5=>3, 7=>4, _=>0 }
    }

    fn step_one(&mut self) -> Option<String> {
        if self.halted || self.pc >= self.program.len() { return None; }
        let ins = self.program[self.pc].clone();
        let (op, d1, d2, pc) = (ins.op, ins.d1, ins.d2, self.pc);
        self.cycle += 1;

        let msg = match op {
            OP_ADD => {
                self.acc = d1.wrapping_add(d2); self.pc += 1;
                format!("ADD   {} + {} = {}", d1, d2, self.acc)
            }
            OP_SUB => {
                self.acc = d1.wrapping_sub(d2); self.pc += 1;
                format!("SUB   {} - {} = {}", d1, d2, self.acc)
            }
            OP_OR => {
                self.acc = d1 | d2; self.pc += 1;
                format!("OR    {} | {} = {}", d1, d2, self.acc)
            }
            OP_AND => {
                self.acc = d1 & d2; self.pc += 1;
                format!("AND   {} & {} = {}", d1, d2, self.acc)
            }
            OP_XOR => {
                self.acc = d1 ^ d2; self.pc += 1;
                format!("XOR   {} ^ {} = {}", d1, d2, self.acc)
            }
            OP_AND1B => {
                self.flag = d1 == d2; self.pc += 1;
                format!("AND1B {} == {} → flag={}", d1, d2, self.flag)
            }
            OP_GT => {
                self.flag = d1 > d2; self.pc += 1;
                format!("GT    {} > {} → flag={}", d1, d2, self.flag)
            }
            OP_FLAG | OP_FLAG_GT => {
                let dst = if self.flag { d1 as usize } else { d2 as usize };
                self.pc = dst;
                format!("{} {} → PC={}",
                    if op==OP_FLAG {"FLAG   "} else {"FLAG_GT"},
                    if self.flag {"TRUE "} else {"FALSE"}, dst)
            }
            OP_JMP => {
                self.pc = d1 as usize;
                format!("JMP   → {}", d1)
            }
            OP_LATCH if d1 != 0 => {
                let enc = (d1 & 0x3F) as u8;
                let w   = ((d1 >> 9) & 1) == 1;
                let idx = (enc >> 1) & 0x1F;
                if w {
                    self.regs[idx as usize] = d2; self.pc += 1;
                    format!("REG   WRITE R{} = {}", idx, d2)
                } else {
                    self.acc = self.regs[idx as usize]; self.pc += 1;
                    format!("REG   READ  R{} → ACC={}", idx, self.acc)
                }
            }
            OP_LATCH => {
                self.latch = !self.latch; self.pc += 1;
                format!("LATCH verrouillé={}", self.latch)
            }
            OP_IN => {
                let pin = Self::pin_bits_to_index((d1 & 7) as u8);
                self.flag = pin>=1 && pin<=4 && self.gpio_in[pin as usize];
                self.pc  += 1;
                format!("IN    pin{} → flag={}", pin, self.flag)
            }
            OP_OUT => {
                let pin = Self::pin_bits_to_index((d1&7) as u8);
                let val = ((d1>>3)&1) as u8;
                let cut = ((d1>>5)&1)==1;
                self.pc += 1;
                if cut {
                    for p in 1..=4 { self.gpio_out[p]=false; }
                    "OUT   CutAll=0".into()
                } else {
                    if pin>=1&&pin<=4 { self.gpio_out[pin as usize]=val==1; }
                    format!("OUT   pin{}={}", pin, val)
                }
            }
            OP_INFLAG => {
                let dst = if self.flag {d1 as usize} else {d2 as usize};
                self.pc = dst;
                format!("INFLAG {} → PC={}", if self.flag{"TRUE"}else{"FALSE"}, dst)
            }
            OP_NOP  => { self.pc += 1; "NOP".into() }
            OP_STOP => {
                self.halted=true; self.running=false;
                format!("STOP  halted après {} cycles", self.cycle)
            }
            _ => { self.pc += 1; format!("???   0x{:03X}", op) }
        };

        Some(format!("[{:>8}] PC={:>4}  {}", self.cycle, pc, msg))
    }

    fn tick(&mut self) -> Vec<String> {
        if !self.running || self.halted { return vec![]; }
        let now = Instant::now();
        let dt  = self.last_tick
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(0.0);
        self.last_tick = Some(now);
        self.budget   += dt * self.mhz * 1_000_000.0;
        let max = ((self.mhz * 1_000_000.0 / 60.0).ceil() as u64).min(100_000);
        let mut logs = vec![];
        let mut n    = 0u64;
        while self.budget >= 1.0 && n < max {
            match self.step_one() {
                Some(l) => logs.push(l),
                None    => { self.running = false; break; }
            }
            self.budget -= 1.0;
            n           += 1;
        }
        logs
    }
}

// ═══════════════════════════════════════════
//  FENETRE SETUP
// ═══════════════════════════════════════════
fn run_setup() -> Option<Cpu> {
    let mut window = nwg::Window::default();
    nwg::Window::builder()
        .size((500, 300))
        .position((300, 200))
        .title("MSL5 Simulator — Configuration")
        .build(&mut window).unwrap();
    let window_handle = window.handle;

    let mut lbl_f = nwg::Label::default();
    nwg::Label::builder()
        .text("Fichier programme (.txt) :")
        .size((460, 22)).position((20, 20))
        .parent(&window).build(&mut lbl_f).unwrap();

    let mut inp_file = nwg::TextInput::default();
    nwg::TextInput::builder()
        .text("programme.txt")
        .size((350, 28)).position((20, 46))
        .parent(&window).build(&mut inp_file).unwrap();

    let mut btn_browse = nwg::Button::default();
    nwg::Button::builder()
        .text("Parcourir")
        .size((100, 28)).position((378, 46))
        .parent(&window).build(&mut btn_browse).unwrap();

    let mut dlg = nwg::FileDialog::default();
    nwg::FileDialog::builder()
        .title("Ouvrir programme MSL5")
        .action(nwg::FileDialogAction::Open)
        .filters("Texte (*.txt)|*.txt")
        .build(&mut dlg).unwrap();

    let mut lbl_m = nwg::Label::default();
    nwg::Label::builder()
        .text("Fréquence MHz (ex: 1.0 = 1MHz, 0.001 = 1kHz) :")
        .size((460, 22)).position((20, 90))
        .parent(&window).build(&mut lbl_m).unwrap();

    let mut inp_mhz = nwg::TextInput::default();
    nwg::TextInput::builder()
        .text("1.0")
        .size((460, 28)).position((20, 116))
        .parent(&window).build(&mut inp_mhz).unwrap();

    let mut lbl_err = nwg::Label::default();
    nwg::Label::builder()
        .text("")
        .size((460, 22)).position((20, 158))
        .parent(&window).build(&mut lbl_err).unwrap();

    let mut btn_start = nwg::Button::default();
    nwg::Button::builder()
        .text("Demarrer la simulation")
        .size((220, 36)).position((140, 192))
        .parent(&window).build(&mut btn_start).unwrap();

    // Résultat partagé
    let result: Rc<RefCell<Option<Cpu>>> = Rc::new(RefCell::new(None));
    let result_clone = Rc::clone(&result);

    let handler = nwg::full_bind_event_handler(
        &window_handle,
        move |evt, _data, handle| {
            match evt {
                // ── Browse ──────────────────────────────────
                nwg::Event::OnButtonClick if handle == btn_browse.handle => {
                    if dlg.run(Some(window_handle)) {
                        if let Ok(p) = dlg.get_selected_item() {
                            inp_file.set_text(&p.to_string_lossy());
                        }
                    }
                }

                // ── Démarrer ─────────────────────────────────
                nwg::Event::OnButtonClick if handle == btn_start.handle => {
                    let path = inp_file.text();
                    let mhz  = inp_mhz.text().parse::<f64>().unwrap_or(1.0);
                    let mut cpu = Cpu::new();
                    cpu.mhz = mhz;
                    match cpu.load(&path) {
                        Ok(()) => {
                            *result_clone.borrow_mut() = Some(cpu);
                            nwg::stop_thread_dispatch();
                        }
                        Err(e) => {
                            lbl_err.set_text(&format!("Erreur: {}", e));
                        }
                    }
                }

                nwg::Event::OnWindowClose => {
                    nwg::stop_thread_dispatch();
                }

                _ => {}
            }
        }
    );

    window.set_visible(true);
    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);

    Rc::try_unwrap(result).ok().expect("Unexpected multiple result references").into_inner()
}

// ═══════════════════════════════════════════
//  FENETRE SIMULATEUR
// ═══════════════════════════════════════════
fn run_simulator(cpu_init: Cpu) {
    let cpu_cell: RefCell<Cpu> = RefCell::new(cpu_init);

    let mut window = nwg::Window::default();
    nwg::Window::builder()
        .size((1260, 820))
        .position((30, 30))
        .title("MSL5 CPU Simulator")
        .build(&mut window).unwrap();

    // ── Toolbar ──────────────────────────────────
    let mut btn_run  = nwg::Button::default();
    let mut btn_step = nwg::Button::default();
    let mut btn_rst  = nwg::Button::default();
    let mut inp_mhz  = nwg::TextInput::default();
    let mut lbl_stat = nwg::Label::default();
    let mut lbl_cyc  = nwg::Label::default();

    nwg::Button::builder().text("Run")
        .size((80,28)).position((8,8))
        .parent(&window).build(&mut btn_run).unwrap();
    nwg::Button::builder().text("Step")
        .size((80,28)).position((94,8))
        .parent(&window).build(&mut btn_step).unwrap();
    nwg::Button::builder().text("Reset")
        .size((80,28)).position((180,8))
        .parent(&window).build(&mut btn_rst).unwrap();

    let mut lbl_mhz_l = nwg::Label::default();
    nwg::Label::builder().text("MHz:")
        .size((40,28)).position((278,12))
        .parent(&window).build(&mut lbl_mhz_l).unwrap();
    nwg::TextInput::builder()
        .text(&format!("{}", cpu_cell.borrow().mhz))
        .size((100,28)).position((320,8))
        .parent(&window).build(&mut inp_mhz).unwrap();

    nwg::Label::builder().text("PAUSED")
        .size((160,28)).position((436,12))
        .parent(&window).build(&mut lbl_stat).unwrap();
    nwg::Label::builder().text("Cycles: 0")
        .size((220,28)).position((608,12))
        .parent(&window).build(&mut lbl_cyc).unwrap();

    // ── Colonne gauche — CPU state ────────────────
    let make_label = |txt: &str, x: i32, y: i32, w: i32,
                       parent: &nwg::Window| -> nwg::Label {
        let mut l = nwg::Label::default();
        nwg::Label::builder().text(txt).size((w,20))
            .position((x,y)).parent(parent).build(&mut l).unwrap();
        l
    };

    let _lpc  = make_label("PC :",    4, 48,  60, &window);
    let _lacc = make_label("ACC :",   4, 72,  60, &window);
    let _lflg = make_label("FLAG :",  4, 96,  60, &window);
    let _llat = make_label("LATCH :", 4, 120, 60, &window);

    let mut out_pc   = nwg::TextInput::default();
    let mut out_acc  = nwg::TextInput::default();
    let mut out_flag = nwg::TextInput::default();
    let mut out_lat  = nwg::TextInput::default();

    for (out, y) in [
        (&mut out_pc,   48i32),
        (&mut out_acc,  72),
        (&mut out_flag, 96),
        (&mut out_lat,  120),
    ] {
        nwg::TextInput::builder().text("0")
            .size((178,20)).position((66,y))
            .readonly(true).parent(&window)
            .build(out).unwrap();
    }

    // Registres — TextBox multiligne (plus rapide à updater)
    let _lreg = make_label("Registres :", 4, 148, 120, &window);
    let mut txt_regs = nwg::TextBox::default();
    nwg::TextBox::builder()
        .text("")
        .size((248, 630)).position((4, 168))
        .readonly(true)
        .parent(&window).build(&mut txt_regs).unwrap();

    // ── Log central ──────────────────────────────
    let _llg = make_label("Log execution :", 260, 44, 200, &window);
    let mut txt_log = nwg::TextBox::default();
    nwg::TextBox::builder()
        .text("")
        .size((710, 756)).position((260, 64))
        .readonly(true)
        .parent(&window).build(&mut txt_log).unwrap();

    // ── GPIO IN ───────────────────────────────────
    let _lgi = make_label("GPIO IN (clic = toggle) :", 984, 44, 260, &window);
    let mut gin_btns: Vec<nwg::Button> = Vec::new();
    for i in 0..4usize {
        let mut b = nwg::Button::default();
        nwg::Button::builder()
            .text(&format!("Pin {}  LOW", i+1))
            .size((256, 42)).position((984, 68 + i as i32 * 50))
            .parent(&window).build(&mut b).unwrap();
        gin_btns.push(b);
    }

    let _lgo = make_label("GPIO OUT (lecture) :", 984, 282, 260, &window);
    let mut gout_lbl: Vec<nwg::Label> = Vec::new();
    for i in 0..4usize {
        let mut l = nwg::Label::default();
        nwg::Label::builder()
            .text(&format!("Pin {}  LOW", i+1))
            .size((256, 42)).position((984, 306 + i as i32 * 50))
            .parent(&window).build(&mut l).unwrap();
        gout_lbl.push(l);
    }

    // ── Timer AnimationTimer ──────────────────────
    // ✅ On utilise AnimationTimer (pas Timer déprécié)
    let mut timer = nwg::AnimationTimer::default();
    nwg::AnimationTimer::builder()
        .interval(std::time::Duration::from_millis(16))
        .parent(&window)
        .build(&mut timer).unwrap();
    timer.start();

    // ── Event handler ─────────────────────────────
    // ✅ Le 3ème param "handle" = le contrôle qui a déclenché l'event
    let handler = nwg::full_bind_event_handler(
        &window.handle,
        move |evt, _data, handle| {
            match evt {

                // ── Tick CPU ─────────────────────────────────
                nwg::Event::OnTimerTick => {
                    let logs = cpu_cell.borrow_mut().tick();

                    if !logs.is_empty() {
                        let mut current = txt_log.text();
                        for line in &logs {
                            current.push_str(line);
                            current.push('\n');
                        }
                        // Garde max ~300 lignes
                        let lines: Vec<&str> = current.lines().collect();
                        let start = if lines.len() > 300 {
                            lines.len() - 300
                        } else { 0 };
                        let trimmed = lines[start..].join("\n");
                        txt_log.set_text(&trimmed);
                        // Scroll to bottom
                        txt_log.set_selection(
                            trimmed.len() as u32..trimmed.len() as u32
                        );
                    }

                    // Mise à jour état CPU
                    let c = cpu_cell.borrow();
                    out_pc.set_text(&c.pc.to_string());
                    out_acc.set_text(&format!("{} (0x{:08X})", c.acc, c.acc));
                    out_flag.set_text(&c.flag.to_string());
                    out_lat.set_text(&c.latch.to_string());
                    lbl_cyc.set_text(&format!("Cycles: {}", c.cycle));

                    if c.halted {
                        lbl_stat.set_text("!!! HALTED !!!");
                    }

                    // Registres — reconstruit le texte complet
                    let mut reg_txt = String::new();
                    for i in 0..32usize {
                        let v = c.regs[i];
                        reg_txt.push_str(&format!(
                            "R{:<2}  {:>10}  0x{:08X}\r\n", i, v, v
                        ));
                    }
                    txt_regs.set_text(&reg_txt);

                    // GPIO OUT
                    for i in 0..4usize {
                        let state = c.gpio_out[i+1];
                        gout_lbl[i].set_text(&format!(
                            "Pin {}  {}", i+1,
                            if state { "HIGH ✅" } else { "LOW  [ ]" }
                        ));
                    }
                }

                // ── Run / Pause ──────────────────────────────
                nwg::Event::OnButtonClick if handle == btn_run.handle => {
                    let mut c = cpu_cell.borrow_mut();
                    if c.halted { return; }
                    c.running = !c.running;
                    if c.running {
                        c.last_tick = Some(Instant::now());
                        c.budget    = 0.0;
                        btn_run.set_text("Pause");
                        lbl_stat.set_text("RUNNING");
                    } else {
                        btn_run.set_text("Run");
                        lbl_stat.set_text("PAUSED");
                    }
                }

                // ── Step ─────────────────────────────────────
                nwg::Event::OnButtonClick if handle == btn_step.handle => {
                    let mut c = cpu_cell.borrow_mut();
                    if c.running || c.halted { return; }
                    if let Some(line) = c.step_one() {
                        let mut txt = txt_log.text();
                        txt.push_str(&line);
                        txt.push('\n');
                        txt_log.set_text(&txt);
                        txt_log.set_selection(txt.len() as u32..txt.len() as u32);
                    }
                }

                // ── Reset ─────────────────────────────────────
                nwg::Event::OnButtonClick if handle == btn_rst.handle => {
                    cpu_cell.borrow_mut().reset();
                    txt_log.set_text("");
                    lbl_stat.set_text("PAUSED");
                    btn_run.set_text("Run");
                    lbl_cyc.set_text("Cycles: 0");
                    out_pc.set_text("0");
                    out_acc.set_text("0");
                    out_flag.set_text("false");
                    out_lat.set_text("false");
                    for i in 0..4usize {
                        gin_btns[i].set_text(&format!("Pin {}  LOW", i+1));
                    }
                }

                // ── MHz texte changé ──────────────────────────
                nwg::Event::OnTextInput if handle == inp_mhz.handle => {
                    if let Ok(v) = inp_mhz.text().parse::<f64>() {
                        cpu_cell.borrow_mut().mhz = v.max(0.001).min(1000.0);
                    }
                }

                // ── GPIO IN toggles ───────────────────────────
                nwg::Event::OnButtonClick => {
                    for i in 0..4usize {
                        if handle == gin_btns[i].handle {
                            let mut c = cpu_cell.borrow_mut();
                            c.gpio_in[i+1] = !c.gpio_in[i+1];
                            let state = c.gpio_in[i+1];
                            drop(c);
                            gin_btns[i].set_text(&format!(
                                "Pin {}  {}", i+1,
                                if state { "HIGH ✅" } else { "LOW  [ ]" }
                            ));
                        }
                    }
                }

                // ── Fermeture ─────────────────────────────────
                nwg::Event::OnWindowClose => {
                    timer.stop();
                    nwg::stop_thread_dispatch();
                }

                _ => {}
            }
        }
    );

    window.set_visible(true);
    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

// ═══════════════════════════════════════════
//  MAIN
// ═══════════════════════════════════════════
fn main() {
    nwg::init().expect("Erreur init NWG");
    nwg::Font::set_global_family("Consolas").ok();

    loop {
        match run_setup() {
            Some(cpu) => run_simulator(cpu),
            None      => break,   // fenêtre setup fermée = on quitte
        }
    }
}