#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use calculator_clh_1000::{Calc, Key, Mode};
use eframe::egui;

type Btn = (Key, &'static str, Option<&'static str>, Option<&'static str>);

// ---------------------------------------------------------------------------
// Design tokens — Casio fx-82MS (S-V.P.A.M.)
// ---------------------------------------------------------------------------
const fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

const CHASSIS: egui::Color32 = rgb(34, 46, 53);
const BEZEL: egui::Color32 = rgb(186, 196, 206);
const LCD_BG: egui::Color32 = rgb(198, 208, 196);
const LCD_TEXT: egui::Color32 = rgb(12, 18, 15);
const LCD_GHOST: egui::Color32 = rgb(176, 186, 176); // inativo (opacity ~0.15, quase apagado)
const GOLD: egui::Color32 = rgb(255, 213, 79);
const ALPHA_RED: egui::Color32 = rgb(255, 138, 128);
const RED: egui::Color32 = rgb(211, 47, 47);
const SHIFT_KEY: egui::Color32 = rgb(251, 192, 45);
const ALPHA_KEY: egui::Color32 = rgb(194, 24, 91);
const SCI_KEY: egui::Color32 = rgb(55, 71, 79);
const NUM_KEY: egui::Color32 = rgb(176, 190, 197);
const OP_KEY: egui::Color32 = rgb(120, 144, 156);
const NAV_KEY: egui::Color32 = rgb(69, 90, 100);
const INK: egui::Color32 = rgb(17, 17, 17);
const WHITE: egui::Color32 = egui::Color32::WHITE;
const SILVER: egui::Color32 = rgb(200, 208, 216);
const STROKE: egui::Color32 = rgb(15, 20, 25);

const KEY_H: f32 = 30.0; // altura do corpo da tecla
const LABEL_H: f32 = 14.0; // altura da faixa de serigrafia

#[derive(Clone, Copy)]
enum KStyle {
    Shift,
    Alpha,
    Nav,
    Sci,
    Num,
    Op,
    Red,
}

impl KStyle {
    fn fill(self) -> egui::Color32 {
        match self {
            KStyle::Shift => SHIFT_KEY,
            KStyle::Alpha => ALPHA_KEY,
            KStyle::Nav => NAV_KEY,
            KStyle::Sci => SCI_KEY,
            KStyle::Num => NUM_KEY,
            KStyle::Op => OP_KEY,
            KStyle::Red => RED,
        }
    }
    fn text(self) -> egui::Color32 {
        match self {
            KStyle::Shift | KStyle::Num => INK,
            _ => WHITE,
        }
    }
    fn corner(self) -> f32 {
        match self {
            KStyle::Shift | KStyle::Alpha | KStyle::Nav => 6.0,
            _ => 4.0,
        }
    }
    fn amber(self) -> egui::Color32 {
        match self {
            KStyle::Num => rgb(184, 120, 0),
            _ => GOLD,
        }
    }
    fn red(self) -> egui::Color32 {
        match self {
            KStyle::Num => rgb(176, 40, 40),
            _ => ALPHA_RED,
        }
    }
}

struct App {
    calc: Calc,
}

impl Default for App {
    fn default() -> Self {
        Self { calc: Calc::default() }
    }
}

fn middle_rows() -> Vec<Vec<Btn>> {
    // 3 linhas × 6 teclas (as 4 teclas x⁻¹/nCr/Pol(/x³ ficam no bloco superior, ao lado do REPLAY).
    vec![
        // linha 1: a b/c, √, x², ^, log, ln
        vec![
            (Key::B13, "a b/c", Some("d/c"), None),
            (Key::B14, "√", None, None),
            (Key::B15, "x²", None, None),
            (Key::B16, "^", Some("ⁿ√"), None),
            (Key::B17, "log", Some("10ⁿ"), None),
            (Key::B18, "ln", Some("eⁿ"), Some("e")),
        ],
        // linha 2: (−), °'″, hyp, sin, cos, tan
        vec![
            (Key::B45, "(−)", None, Some("A")),
            (Key::B20, "° ' ''", Some("DMS"), Some("B")),
            (Key::Hyp, "hyp", Some("hyp⁻¹"), Some("C")),
            (Key::B22, "sin", Some("sin⁻¹"), Some("D")),
            (Key::B23, "cos", Some("cos⁻¹"), Some("E")),
            (Key::B24, "tan", Some("tan⁻¹"), Some("F")),
        ],
        // linha 3: RCL, ENG, (, ), ,, M+
        vec![
            (Key::B25, "RCL", Some("STO"), None),
            (Key::B26, "ENG", None, None),
            (Key::B27, "(", None, Some("X")),
            (Key::B28, ")", Some(","), Some("Y")),
            (Key::B29, ",", Some(";"), None),
            (Key::B30, "M+", Some("M−"), Some("Z")),
        ],
    ]
}

/// As 3 primeiras linhas do bloco numérico (a última é especial: tecla 0 dupla).
fn bottom_rows() -> Vec<Vec<(Btn, KStyle)>> {
    vec![
        vec![
            ((Key::B31, "7", None, None), KStyle::Num),
            ((Key::B32, "8", None, None), KStyle::Num),
            ((Key::B33, "9", None, None), KStyle::Num),
            ((Key::B34, "DEL", None, None), KStyle::Red),
            ((Key::B35, "AC", None, None), KStyle::Red),
        ],
        vec![
            ((Key::B36, "4", None, None), KStyle::Num),
            ((Key::B37, "5", None, None), KStyle::Num),
            ((Key::B38, "6", None, None), KStyle::Num),
            ((Key::B39, "×", None, None), KStyle::Op),
            ((Key::B40, "÷", None, None), KStyle::Op),
        ],
        vec![
            ((Key::B41, "1", None, None), KStyle::Num),
            ((Key::B42, "2", None, None), KStyle::Num),
            ((Key::B43, "3", None, None), KStyle::Num),
            ((Key::B44, "+", None, None), KStyle::Op),
            ((Key::B45, "−", None, None), KStyle::Op),
        ],
    ]
}

fn keyboard_key(ui: &egui::Ui) -> Option<Key> {
    ui.input(|i| {
        // 1) Caracteres digitados (Event::Text) — cobre dígitos, operadores e símbolos,
        //    incluindo "*" (Shift+8), que NÃO tem variante própria em egui::Key.
        for ev in &i.events {
            if let egui::Event::Text(t) = ev {
                let k = match t.as_str() {
                    "0" => Key::B46,
                    "1" => Key::B41,
                    "2" => Key::B42,
                    "3" => Key::B43,
                    "4" => Key::B36,
                    "5" => Key::B37,
                    "6" => Key::B38,
                    "7" => Key::B31,
                    "8" => Key::B32,
                    "9" => Key::B33,
                    "+" => Key::B44,
                    "-" => Key::B45,
                    "*" | "×" => Key::B39, // multiplicação
                    "/" | "÷" => Key::B40,
                    "." => Key::B47,
                    "," => Key::B29,
                    "=" => Key::B50,
                    "^" => Key::B16, // potência
                    "(" => Key::B27,
                    ")" => Key::B28,
                    "x" | "X" => Key::VarX, // variável X (memória)
                    _ => continue,
                };
                return Some(k);
            }
        }
        // 2) Teclas especiais sem caractere imprimível correspondente + hyp.
        for ev in &i.events {
            if let egui::Event::Key { key, pressed: true, .. } = ev {
                let k = match key {
                    egui::Key::Enter => Key::B50,
                    egui::Key::Backspace => Key::B34,
                    egui::Key::Escape => Key::B35,
                    egui::Key::H => Key::Hyp,
                    _ => continue,
                };
                return Some(k);
            }
        }
        None
    })
}

/// Variante com largura explícita (linha com tecla 0 dupla).
fn key_w(ui: &mut egui::Ui, w: f32, btn: &Btn, style: KStyle, pressed: &mut Option<Key>) {
    let (k, main, shift, alpha) = *btn;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
        // espaço fixo de serigrafia (mesma altura em TODAS as teclas → linha de base idêntica)
        ui.add_space(LABEL_H);
        let (rect, response) = ui.allocate_exact_size(egui::vec2(w, KEY_H), egui::Sense::click());
        let painter = ui.painter();
        painter.rect(
            rect,
            style.corner(),
            style.fill(),
            egui::Stroke::new(1.0, STROKE),
            egui::StrokeKind::Inside,
        );
        // texto interno centralizado no meio exato da tecla (horizontal e vertical)
        painter.text(
            rect.center() + egui::vec2(0.0, 1.0),
            egui::Align2::CENTER_CENTER,
            main,
            egui::FontId::proportional(14.0),
            style.text(),
        );
        // serigrafias secundárias — desenhadas via painter no espaço reservado acima do botão
        let band_mid_y = rect.top() - LABEL_H / 2.0 - 2.0;
        if let Some(a) = alpha {
            painter.text(
                egui::pos2(rect.left() + 4.0, band_mid_y),
                egui::Align2::LEFT_CENTER,
                a,
                egui::FontId::proportional(11.0),
                style.red(),
            );
        }
        if let Some(s) = shift {
            painter.text(
                egui::pos2(rect.center().x, band_mid_y),
                egui::Align2::CENTER_CENTER,
                s,
                egui::FontId::proportional(11.0),
                style.amber(),
            );
        }
        if response.clicked() {
            *pressed = Some(k);
        }
    });
}

/// Uma fileira de teclas com larguras iguais, preenchendo exatamente a largura disponível.
/// `n_cols` é o número de colunas da grade (largura uniforme mesmo que haja menos teclas).
fn key_row(ui: &mut egui::Ui, items: &[(Btn, KStyle)], n_cols: usize, pressed: &mut Option<Key>) {
    let gap = 8.0;
    let avail = ui.available_width();
    let n = n_cols.max(1) as f32;
    let w = ((avail - gap * (n - 1.0)) / n).max(0.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (b, st) in items {
            key_w(ui, w, b, *st, pressed);
        }
    });
}

/// REPLAY: corpo circular (fundo do direcional) com centro claro e 4 setas em raio simétrico.
fn replay_cluster(ui: &mut egui::Ui, pressed: &mut Option<Key>) {
    let w = 110.0;
    let h = 80.0;
    let (area, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let painter = ui.painter();
    let cx = area.center().x;
    let cy = area.center().y;
    // corpo circular (fundo azulado do direcional)
    painter.circle_filled(egui::pos2(cx, cy), 34.0, NAV_KEY);
    painter.circle_stroke(egui::pos2(cx, cy), 34.0, egui::Stroke::new(1.0, STROKE));
    // centro circular claro (prateado)
    painter.circle_filled(egui::pos2(cx, cy), 15.0, BEZEL);
    painter.circle_stroke(egui::pos2(cx, cy), 15.0, egui::Stroke::new(1.0, STROKE));
    // 4 setas no mesmo raio (círculo simétrico); laterais maiores, superior/inferior menores
    let arrows: [(egui::Pos2, (f32, f32), f32, Key); 4] = [
        (egui::pos2(cx, cy - 32.0), (0.0, -1.0), 6.5, Key::B05), // cima
        (egui::pos2(cx, cy + 32.0), (0.0, 1.0), 6.5, Key::B08),  // baixo
        (egui::pos2(cx - 32.0, cy), (-1.0, 0.0), 9.0, Key::B07), // esquerda
        (egui::pos2(cx + 32.0, cy), (1.0, 0.0), 9.0, Key::B06),  // direita
    ];
    let mut rects: Vec<(egui::Rect, Key)> = Vec::with_capacity(4);
    for (c, (dx, dy), s, k) in &arrows {
        let dir = egui::vec2(*dx, *dy);
        let perp = egui::vec2(-*dy, *dx);
        let tip = *c + dir * *s;
        let b1 = *c - dir * *s + perp * *s;
        let b2 = *c - dir * *s - perp * *s;
        painter.add(egui::Shape::convex_polygon(
            vec![tip, b1, b2],
            WHITE,
            egui::Stroke::NONE,
        ));
        rects.push((egui::Rect::from_center_size(*c, egui::vec2(26.0, 24.0)), *k));
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            for (r, k) in &rects {
                if r.contains(pos) {
                    *pressed = Some(*k);
                    break;
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let tela = self.calc.tela_formatada();
        let expo = self.calc.expo;
        let result = if self.calc.expo {
            format!("{}×10^{}", self.calc.tela_resultado, self.calc.tt_exp)
        } else {
            self.calc.tela_resultado.clone()
        };
        let shift = self.calc.shift;
        let alpha = self.calc.alpha;
        let hyp = self.calc.hyp;
        let hyp_inv = self.calc.hyp_inv;
        let mem_m = self.calc.ctx.men_z != 0.0;
        let sto = self.calc.sto;
        let is_sd = self.calc.mode == Mode::Sd;
        let eng_on = self.calc.eng != 2;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(CHASSIS).corner_radius(14.0).inner_margin(egui::Margin {
                left: 18,
                right: 18,
                top: 18,
                bottom: 10,
            }))
            .show(ui, |ui| {
                // zera o padding interno dos botões e compacta o espaçamento vertical
                ui.style_mut().spacing.button_padding = egui::vec2(0.0, 0.0);
                ui.style_mut().spacing.item_spacing.y = 3.0;

                // ---- logo ----
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("CALCULATOR")
                            .font(egui::FontId::new(21.0, egui::FontFamily::Name("barlow_condensed".into())))
                            .color(SILVER),
                    );
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("CLH-0001").size(11.0).strong().color(rgb(150, 158, 166)));
                });

                // ---- (faixa vermelha removida no rebranding V2) ----
                ui.add_space(6.0);

                // ---- visor ----
                egui::Frame::new().fill(BEZEL).corner_radius(8.0).inner_margin(6.0).show(ui, |ui| {
                    egui::Frame::new().fill(LCD_BG).corner_radius(2.0).inner_margin(6.0).show(ui, |ui| {
                        // barra de status distribuída uniformemente
                        let inds: [(&str, bool); 11] = [
                            ("S", shift),
                            ("A", alpha),
                            ("hyp", hyp || hyp_inv),
                            ("M", mem_m),
                            ("STO", sto),
                            ("RCL", false),
                            ("D", true),
                            ("R", false),
                            ("G", false),
                            ("SD", is_sd),
                            ("ENG", eng_on),
                        ];
                        let n = inds.len() as f32;
                        let slot = ui.available_width() / n;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            for (txt, on) in inds {
                                let color = if on { LCD_TEXT } else { LCD_GHOST };
                                ui.add_sized(
                                    [slot, 12.0],
                                    egui::Label::new(
                                        egui::RichText::new(txt).size(9.5).strong().color(color),
                                    ),
                                );
                            }
                        });

                        ui.add_space(4.0);

                        // linha superior — fórmula (matriz de pontos, alinhada à direita, largura fixa, clip à esquerda)
                        let avail = ui.available_width();
                        {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(avail, 22.0), egui::Sense::hover());
                            let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
                            painter.text(
                                rect.right_center(),
                                egui::Align2::RIGHT_CENTER,
                                &tela,
                                egui::FontId::monospace(20.0),
                                LCD_TEXT,
                            );
                        }

                        ui.add_space(3.0);

                        // linha inferior — resultado (7 segmentos, alinhado à direita, largura fixa, clip à esquerda)
                        let result_font = if expo {
                            egui::FontId::monospace(28.0)
                        } else {
                            egui::FontId::new(28.0, egui::FontFamily::Name("dseg7".into()))
                        };
                        {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(avail, 30.0), egui::Sense::hover());
                            let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
                            painter.text(
                                rect.right_center(),
                                egui::Align2::RIGHT_CENTER,
                                &result,
                                result_font,
                                LCD_TEXT,
                            );
                        }
                    });
                });

                ui.add_space(8.0);

                let mut pressed: Option<Key> = None;

                // ---- zona superior: 2 fileiras combinadas, REPLAY centralizado entre elas ----
                {
                    let gap = 8.0;
                    let avail = ui.available_width();
                    let col_w = ((avail - gap * 5.0) / 6.0).max(0.0);
                    let replay_w = col_w * 2.0 + gap; // REPLAY ocupa as 2 colunas centrais
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        // Esquerda: SHIFT/ALPHA (topo) + x⁻¹/nCr (base)
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                key_w(ui, col_w, &(Key::Shift, "SHIFT", None, None), KStyle::Shift, &mut pressed);
                                key_w(ui, col_w, &(Key::Alpha, "ALPHA", None, None), KStyle::Alpha, &mut pressed);
                            });
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                key_w(ui, col_w, &(Key::B03, "x⁻¹", Some("x!"), None), KStyle::Sci, &mut pressed);
                                key_w(ui, col_w, &(Key::B04, "nCr", Some("nPr"), None), KStyle::Sci, &mut pressed);
                            });
                        });
                        // Centro: REPLAY (rótulo + cruz centralizados no vão entre as 2 fileiras)
                        ui.vertical(|ui| {
                            ui.add_space(5.0); // desce o cluster para o centro exato entre SHIFT e x⁻¹
                            let mut job = egui::text::LayoutJob::default();
                            job.append(
                                "REPLAY",
                                0.0,
                                egui::TextFormat {
                                    font_id: egui::FontId::proportional(9.0),
                                    color: rgb(150, 160, 168),
                                    ..Default::default()
                                },
                            );
                            job.halign = egui::Align::Center;
                            ui.add_sized([replay_w, 9.0], egui::Label::new(job).selectable(false));
                            ui.horizontal(|ui| {
                                ui.add_space((replay_w - 110.0).max(0.0) / 2.0);
                                replay_cluster(ui, &mut pressed);
                            });
                        });
                        // Direita: MODE/ON (topo) + Pol(/x³ (base)
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                key_w(ui, col_w, &(Key::Mode, "MODE", Some("CLR"), None), KStyle::Nav, &mut pressed);
                                key_w(ui, col_w, &(Key::B35, "ON", None, None), KStyle::Nav, &mut pressed);
                            });
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                key_w(ui, col_w, &(Key::B11, "Pol(", Some("Rec("), None), KStyle::Sci, &mut pressed);
                                key_w(ui, col_w, &(Key::B12, "x³", Some("³√"), None), KStyle::Sci, &mut pressed);
                            });
                        });
                    });
                }

                ui.add_space(4.0);

                // ---- zona intermediária (3 linhas × 6 teclas) ----
                for row in middle_rows() {
                    let gap = 8.0;
                    let avail = ui.available_width();
                    let w = ((avail - gap * 5.0) / 6.0).max(0.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for (k, m, s, a) in &row {
                            key_w(ui, w, &(*k, m, *s, *a), KStyle::Sci, &mut pressed);
                        }
                    });
                    ui.add_space(3.0);
                }

                ui.add_space(5.0);

                // ---- zona inferior (5 colunas, 3 primeiras linhas) ----
                for row in bottom_rows() {
                    key_row(ui, &row, 5, &mut pressed);
                    ui.add_space(3.0);
                }

                // ---- última linha: 0 | . | EXP | Ans | = (5 colunas iguais, fiel à fx-82MS) ----
                ui.add_space(3.0);
                let last_row: [(Btn, KStyle); 5] = [
                    ((Key::B46, "0", None, None), KStyle::Num),
                    ((Key::B47, ".", Some("Ran#"), Some("RInt")), KStyle::Num),
                    ((Key::B48, "EXP", Some("π"), None), KStyle::Op),
                    ((Key::B49, "Ans", None, None), KStyle::Op),
                    ((Key::B50, "=", None, None), KStyle::Op),
                ];
                key_row(ui, &last_row, 5, &mut pressed);

                if let Some(k) = keyboard_key(ui) {
                    pressed = Some(k);
                }
                if let Some(k) = pressed {
                    self.calc.press(k);
                }
            });
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "dseg7".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/DSEG7Classic-Bold.ttf")).into(),
    );
    fonts.font_data.insert(
        "barlow_condensed".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/BarlowCondensed-Bold.ttf")).into(),
    );
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 654.0])
            .with_min_inner_size([430.0, 654.0])
            .with_max_inner_size([430.0, 654.0])
            .with_resizable(false)
            .with_title("Calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
