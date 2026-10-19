//! Porte de `SecondActivity` — máquina de estado da calculadora.
//!
//! O modelo de entrada é uma string de tokens (ver SPEC.md §1–§2): os botões
//! concatenam códigos de 1 caractere em `display`; `calcular()` chama o parser
//! (`FourthActivity.eval`) e formata com `resFinal`.

use crate::fraction;
use crate::parser::{self, EvalContext};
use crate::statistic::Statistic;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Comp,
    Sd,
}

/// Estado completo da calculadora (SecondActivity + FourthActivity).
pub struct Calc {
    pub display: String,
    pub tela: String,
    pub tela_resultado: String,
    pub shift: bool,
    pub alpha: bool,
    pub hyp: bool,
    pub hyp_inv: bool,
    pub mode: Mode,
    pub mode_menu: bool,
    pub sto: bool,
    pub num: bool,
    pub gms: bool,
    pub expo: bool,
    pub tt_exp: i32,
    pub result: f64,
    pub ultimo_comando: String,
    pub memoria: Vec<String>,
    pub memoria_ativa: usize,
    pub posicao_cursor: usize,
    pub eng: i32,
    pub erro: i32,
    pub ctx: EvalContext,
    pub dad: Statistic,
}

impl Default for Calc {
    fn default() -> Self {
        Self {
            display: String::new(),
            tela: "_".to_string(),
            tela_resultado: "0.".to_string(),
            shift: false,
            alpha: false,
            hyp: false,
            hyp_inv: false,
            mode: Mode::Comp,
            mode_menu: false,
            sto: false,
            num: false,
            gms: false,
            expo: false,
            tt_exp: 0,
            result: 10.25,
            ultimo_comando: String::new(),
            memoria: vec![String::new(); 10],
            memoria_ativa: 0,
            posicao_cursor: 0,
            eng: 2,
            erro: 0,
            ctx: EvalContext::default(),
            dad: Statistic::default(),
        }
    }
}

impl Calc {
    fn is_comp(&self) -> bool {
        self.mode == Mode::Comp
    }

    // ---- helpers de string (AvancoActivity) ----

    fn substituir(&self, s: &str) -> String {
        s.replace('x', "*").replace(">>", "")
    }

    /// Converte uma posição em CARACTERES (a partir do início) para o índice em BYTES
    /// correspondente — necessário porque `String::insert`/`len()` operam em bytes, mas o
    /// cursor (`posicao_cursor`) é medido em caracteres (tokens ², ³, º são multi-byte).
    fn byte_index_from_chars(s: &str, char_pos: usize) -> usize {
        s.char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    }

    // ---- display / cursor ----

    /// Insere o marcador de cursor `_` e aplica rolagem (ajusteTela).
    fn posicao_marcador(&self, s: &str) -> String {
        let mut sb = s.to_string();
        let insert_at = s.chars().count().saturating_sub(self.posicao_cursor);
        let idx = Self::byte_index_from_chars(s, insert_at);
        sb.insert(idx, '_');
        self.ajuste_tela(&sb)
    }

    /// Rolagem horizontal estilo Casio (contagem de largura de token fiel ao original).
    fn ajuste_tela(&self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut n = chars.len() as i32;
        for &c in &chars {
            if "hGHIJKLMNOPQRSTUV".contains(c) {
                n += 2;
            }
            if "ijkou".contains(c) {
                n += 3;
            }
            if "rs".contains(c) {
                n += 4;
            }
        }
        if n < 14 {
            return s.to_string();
        }
        let cursor = chars.iter().position(|&c| c == '_').unwrap_or(chars.len());
        let mut sub_start = 0usize;
        let mut sub_end = cursor;
        let mut width = 0i32;
        for &c in &chars[sub_start..sub_end] {
            if "hGHIJKLMNOPQRSTUV".contains(c) {
                width += 2;
            }
            if "ijkou".contains(c) {
                width += 3;
            }
            width += 1;
        }
        if width > 13 {
            let mut w = 0i32;
            let left_len = sub_end - sub_start;
            for i in 0..left_len {
                let c = chars[sub_end - 1 - i];
                if "hGHIJKLMNOPQRSTUV".contains(c) {
                    w += 2;
                }
                if "ijkou".contains(c) {
                    w += 3;
                }
                w += 1;
                if w >= 13 {
                    sub_start = cursor - i;
                    sub_end = cursor + 1;
                    break;
                }
            }
        }
        if width < 13 {
            for (i4, &c) in chars[cursor..].iter().enumerate() {
                if "hGHIJKLMNOPQRSTUV".contains(c) {
                    width += 2;
                }
                if "ijkou".contains(c) {
                    width += 3;
                }
                width += 1;
                if width >= 13 {
                    sub_start = 0;
                    sub_end = cursor + i4;
                    break;
                }
            }
        }
        let mut out = String::new();
        if cursor > 7 {
            out.push('<');
        }
        out.extend(chars[sub_start..sub_end].iter());
        out.push('>');
        out
    }

    /// Traduz os tokens internos de 1 caractere para os símbolos visíveis (√, sin, cos, ∛…),
    /// mantendo o cursor `_` e os marcadores de rolagem `<`/`>` intactos. Não afeta o parser:
    /// o motor continua usando os tokens crus em `self.display`.
    pub fn tela_formatada(&self) -> String {
        let mut out = String::with_capacity(self.tela.len() * 2);
        for c in self.tela.chars() {
            match c {
                'V' => out.push_str("√"),
                'U' => out.push_str("∛"),
                'S' => out.push_str("sin"),
                'R' => out.push_str("cos"),
                'T' => out.push_str("tan"),
                'i' => out.push_str("sin⁻¹"),
                'j' => out.push_str("cos⁻¹"),
                'k' => out.push_str("tan⁻¹"),
                'K' => out.push_str("log"),
                'L' => out.push_str("ln"),
                'J' => out.push_str("Abs"),
                'a' => out.push_str("sinh"),
                'b' => out.push_str("cosh"),
                'c' => out.push_str("tanh"),
                'd' => out.push_str("sinh⁻¹"),
                'f' => out.push_str("cosh⁻¹"),
                'g' => out.push_str("tanh⁻¹"),
                'r' => out.push_str("Ran#"),
                'W' => out.push_str("Ans"),
                'p' => out.push_str("π"),
                'x' => out.push_str("×"),    // multiplicação (token interno minúsculo)
                'l' => out.push_str("⁻¹"),   // recíproco x⁻¹ (pós-fixo)
                'I' => out.push_str("nCr"),
                'P' => out.push_str("nPr"),
                'Q' => out.push_str("ⁿ√"),   // raiz enésima
                'w' => out.push_str("×10^"), // notação científica
                'h' => out.push('°'),        // graus (GMS)
                _ => out.push(c),
            }
        }
        out
    }

    /// `adicionaTexto` — anexa/insere um token no buffer de display.
    pub fn adiciona_texto(&mut self, s: &str) {
        if self.display == "_" {
            self.display = String::new();
        }
        if self.display.chars().count() >= 50 {
            self.tela_resultado = "Limit".to_string();
            return;
        }
        if self.ultimo_comando == "enter" {
            if "!+-/xlPI²³^".contains(s) {
                self.display = format!("W{}", s);
            } else {
                self.display = s.to_string();
            }
        } else if self.num {
            let allowed = if self.gms {
                "0123456789.º"
            } else {
                "0123456789.,"
            };
            if allowed.contains(s) {
                let insert_at = self.display.chars().count().saturating_sub(self.posicao_cursor);
                let idx = Self::byte_index_from_chars(&self.display, insert_at);
                self.display.insert(idx, s.chars().next().unwrap());
            }
        } else {
            let insert_at = self.display.chars().count().saturating_sub(self.posicao_cursor);
            let idx = Self::byte_index_from_chars(&self.display, insert_at);
            self.display.insert(idx, s.chars().next().unwrap());
        }
        self.tela = self.posicao_marcador(&self.display);
        self.ultimo_comando = String::new();
    }

    /// `apagar` — backspace respeitando o cursor.
    pub fn apagar(&mut self) {
        let insert_at = self.display.chars().count().saturating_sub(self.posicao_cursor);
        let idx = Self::byte_index_from_chars(&self.display, insert_at);
        let mut s = self.display.clone();
        s.insert(idx, '_');
        if self.posicao_cursor > 0 {
            let parts: Vec<&str> = s.split('_').collect();
            let (left, right) = (parts[0], parts.get(1).copied().unwrap_or(""));
            if !left.is_empty() {
                if !self.shift {
                    let mut l = left.to_string();
                    l.pop();
                    self.display = format!("{}{}", l, right);
                } else {
                    self.shift = false;
                    self.display = left.to_string();
                    self.posicao_cursor = 0;
                }
            }
        } else if s.chars().count() > 1 {
            let mut chars: Vec<char> = s.chars().collect();
            chars.pop();
            chars.pop();
            self.display = chars.iter().collect();
        }
        self.tela = self.posicao_marcador(&self.display);
    }

    // ---- memória (histórico) ----

    fn memoria(&mut self) {
        for i in (1..10).rev() {
            self.memoria[i] = self.memoria[i - 1].clone();
        }
        self.memoria[0] = self.display.clone();
        self.memoria_ativa = 0;
    }

    fn limpar_variaveis(&mut self) {
        for v in [
            &mut self.ctx.men_a,
            &mut self.ctx.men_b,
            &mut self.ctx.men_c,
            &mut self.ctx.men_d,
            &mut self.ctx.men_e,
            &mut self.ctx.men_f,
            &mut self.ctx.men_x,
            &mut self.ctx.men_y,
            &mut self.ctx.men_z,
        ] {
            *v = 0.0;
        }
    }

    /// `storage(var)` — grava o resultado em menA..menZ.
    fn storage(&mut self, var: char) {
        self.calcular(self.display.clone());
        if self.erro == 0 {
            let v = self.result;
            match var {
                'A' => self.ctx.men_a = v,
                'B' => self.ctx.men_b = v,
                'C' => self.ctx.men_c = v,
                'D' => self.ctx.men_d = v,
                'E' => self.ctx.men_e = v,
                'F' => self.ctx.men_f = v,
                'X' => self.ctx.men_x = v,
                'Y' => self.ctx.men_y = v,
                'Z' => self.ctx.men_z = v,
                _ => {}
            }
            self.tela = format!(">>{}", var);
            self.sto = false;
            self.posicao_cursor = 0;
        }
    }

    fn mem_mais(&mut self) {
        self.calcular(self.display.clone());
        if self.erro == 0 {
            self.ctx.men_z += self.result;
            self.exibir_result(&self.ctx.men_z.to_string());
        }
    }

    fn mem_menos(&mut self) {
        self.calcular(self.display.clone());
        if self.erro == 0 {
            self.ctx.men_z -= self.result;
            self.exibir_result(&self.ctx.men_z.to_string());
        }
    }

    fn exibir_result(&mut self, s: &str) {
        let v = parser::eval(s, &mut self.ctx);
        self.tela_resultado = self.res_final(v);
    }

    // ---- conversões ----

    fn to_graus(&mut self) {
        let v = self.ctx.ultimo_valor.to_degrees();
        self.tela_resultado = self.res_final(v);
        self.ctx.ultimo_valor = v;
    }

    fn to_radianos(&mut self) {
        let v = self.ctx.ultimo_valor.to_radians();
        self.tela_resultado = self.res_final(v);
        self.ctx.ultimo_valor = v;
    }

    // ---- formatação (resFinal) ----

    /// `validarMaxMin` — verifica o CAMPO result (quirk: não o argumento).
    fn validar_max_min(&mut self) {
        if self.result > 1e99 {
            self.erro = 3;
        }
        if self.result < -1e50 {
            self.erro = 3;
        }
    }

    /// `resFinal` — formata o resultado (decimal ou científica/ENG).
    pub fn res_final(&mut self, d: f64) -> String {
        self.validar_max_min();
        if self.erro == 3 {
            return "Limit exceeded".to_string();
        }
        self.erro = 0;
        let (divisor, offset): (f64, f64) = match self.eng {
            0 => (100000.0, 6.0),
            1 => (1000.0, 3.0),
            3 => (0.001, -3.0),
            4 => (1e-5, -6.0),
            5 => (1e-8, -9.0),
            _ => (1.0, 0.0),
        };
        let out: f64;
        if d.abs() > 1e9 {
            let (mant, expoent) = sci_split(d);
            out = mant / divisor;
            self.expo = true;
            self.tt_exp = (expoent + offset) as i32;
        } else if d.abs() < 1e-9 {
            let (mant, expoent) = sci_split(d);
            out = mant / divisor;
            self.expo = true;
            self.tt_exp = (expoent + offset) as i32;
        } else {
            self.expo = false;
            self.tt_exp = 0;
            out = d;
        }
        fmt_0_0_xxxxxxx(out)
    }

    // ---- cálculo principal ----

    /// `calcular` — avalia o display e atualiza resultado/erro.
    pub fn calcular(&mut self, s: String) {
        self.erro = self.ctx.erro_trans;
        if !s.is_empty() {
            let sub = self.substituir(&s);
            if self.funcoes_tecnicas(&sub) {
                self.num = false;
                self.ultimo_comando = "enter".to_string();
                self.posicao_cursor = 0;
                self.tela = self.posicao_marcador(&self.display);
            } else {
                self.result = parser::eval(&sub, &mut self.ctx);
                self.erro = self.ctx.erro_trans;
                match self.erro {
                    0 => {
                        self.memoria();
                        self.ctx.ultimo_valor = self.result;
                        self.ultimo_comando = "enter".to_string();
                        self.tela_resultado = self.res_final(self.result);
                        self.posicao_cursor = 0;
                        self.tela = self.posicao_marcador(&self.display);
                    }
                    1 => {
                        self.tela_resultado = "Error".to_string();
                        self.expo = false;
                    }
                    2 => {
                        self.tela_resultado = "Limit Factor".to_string();
                        self.expo = false;
                    }
                    _ => {
                        self.tela_resultado = "Limit exceeded".to_string();
                        self.expo = false;
                    }
                }
            }
        } else {
            self.result = 0.0;
            self.ctx.ultimo_valor = self.result;
        }
        self.ctx.erro_trans = 0;
    }

    /// `funcoesTecnicas` — formas especiais Pol(/Car(/GMS/RInt.
    fn funcoes_tecnicas(&mut self, s: &str) -> bool {
        if s.contains('M') {
            let args: Vec<f64> = s
                .replace('M', "")
                .replace(')', "")
                .split(',')
                .map(parser::string_to_double)
                .collect();
            let (r, th) = parser::polar(args[0], args[1]);
            self.ctx.men_x = r;
            self.ctx.men_y = th;
            self.exibir_result(&self.ctx.men_x.to_string());
            return true;
        }
        if s.contains('N') {
            let args: Vec<f64> = s
                .replace('N', "")
                .replace(')', "")
                .split(',')
                .map(parser::string_to_double)
                .collect();
            let (x, y) = parser::car(args[0], args[1]);
            self.ctx.men_x = x;
            self.ctx.men_y = y;
            self.exibir_result(&self.ctx.men_x.to_string());
            return true;
        }
        if s.contains('h') {
            let g = parser::gmstograus(&s.replace('h', "").replace(')', ""));
            self.result = g;
            self.ctx.ultimo_valor = g;
            self.ultimo_comando = "enter".to_string();
            self.gms = false;
            self.num = false;
            self.posicao_cursor = 0;
            self.tela_resultado = format!("{} ", g);
            self.tela = self.posicao_marcador(&self.display);
            return true;
        }
        if s.contains('s') {
            let args: Vec<i32> = s
                .replace('s', "")
                .replace(')', "")
                .split(',')
                .map(|x| parser::string_to_double(x) as i32)
                .collect();
            let r = self.ctx.random_int(args[0], args[1]) as f64;
            self.tela_resultado = format!("{:.1}", r);
            self.ctx.ultimo_valor = r;
            return true;
        }
        false
    }

    // ---- modo SD ----

    fn saida_static(&mut self, token: char) {
        if self.dad.quant() > 0 {
            self.tela_resultado = match token {
                'm' => self.res_final(self.dad.quant() as f64),
                'n' => self.res_final(self.dad.media()),
                't' => {
                    let m = self.dad.mediana();
                    self.res_final(m)
                }
                'o' => self.res_final(self.dad.desvio_padrao_n1()),
                'u' => self.res_final(self.dad.variancia_n1()),
                'f' => self.res_final(self.dad.somatotal()),
                'g' => self.res_final(self.dad.soma_qua()),
                _ => "Error".to_string(),
            };
        } else {
            self.tela_resultado = "No Data".to_string();
        }
        self.display.clear();
        self.posicao_cursor = 0;
        self.tela = self.posicao_marcador(&self.display);
    }

    // ---- self-test ----

    pub fn teste_click(&mut self) {
        let mut ok = 0;
        self.display = "1+1".to_string();
        self.calcular(self.display.clone());
        if self.result == 2.0 {
            ok += 1;
        }
        self.display = "W".to_string();
        self.calcular(self.display.clone());
        if self.ctx.ultimo_valor == 2.0 {
            ok += 1;
        }
        self.display = "W2".to_string();
        self.calcular(self.display.clone());
        if self.tela_resultado == "Error" {
            ok += 1;
        }
        self.display = "5!".to_string();
        self.calcular(self.display.clone());
        if self.result == 120.0 {
            ok += 1;
        }
        self.display = "p3".to_string();
        self.calcular(self.display.clone());
        if self.tela_resultado == "Error" {
            ok += 1;
        }
        self.tela.clear();
        self.display.clear();
        self.tela_resultado = if ok == 5 {
            "Test-Ok".to_string()
        } else {
            "Test-Error".to_string()
        };
    }

    // ---- reset / clear ----

    pub fn clear(&mut self) {
        for m in self.memoria.iter_mut() {
            m.clear();
        }
        self.memoria_ativa = 0;
        self.posicao_cursor = 0;
        self.shift = false;
        self.alpha = false;
        self.hyp = false;
        self.hyp_inv = false;
        self.num = false;
        self.gms = false;
        self.sto = false;
        self.limpar_variaveis();
        self.expo = false;
        self.tt_exp = 0;
        self.display.clear();
        self.tela = self.posicao_marcador(&self.display);
    }

    /// `btn35onClick` — AC (reset total).
    pub fn btn_ac(&mut self) {
        self.shift = false;
        self.alpha = false;
        self.hyp = false;
        self.hyp_inv = false;
        self.num = false;
        self.gms = false;
        self.sto = false;
        self.eng = 2;
        self.display = "_".to_string();
        self.tela = "_".to_string();
        self.tela_resultado = "0.".to_string();
        self.posicao_cursor = 0;
        self.expo = false;
        self.tt_exp = 0;
    }

    // ---- dígitos ----

    pub fn digit(&mut self, d: char) {
        if !self.is_comp() {
            if !self.dad.editar {
                self.dad.editar = true;
                self.display.clear();
            }
        }
        self.adiciona_texto(&d.to_string());
    }
}

/// `DecimalFormat("0.0#######")` — até 7 casas, no mínimo 1, round-half-even,
/// sempre com '.'.
fn fmt_0_0_xxxxxxx(x: f64) -> String {
    let mut s = format!("{:.7}", x);
    if let Some(dot) = s.find('.') {
        let int_part = &s[..dot];
        let int_grouped = separar_milhares(int_part);
        let mut frac = s[dot + 1..].to_string();
        while frac.len() > 1 && frac.ends_with('0') {
            frac.pop();
        }
        s = format!("{}.{}", int_grouped, frac);
    } else {
        s.push_str(".0");
    }
    s
}

/// Insere um separador de milhar (espaço — único glifo de espaço da DSEG7) a cada 3 dígitos
/// na parte inteira. A parte decimal permanece corrida.
fn separar_milhares(int_part: &str) -> String {
    let chars: Vec<char> = int_part.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(n + n / 3);
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Separa um double em (mantissa, expoente) no formato "0.00000000E0" do original.
fn sci_split(d: f64) -> (f64, f64) {
    // DecimalFormat("0.00000000E0") → mantissa com 8 casas + expoente inteiro.
    let s = format!("{:e}", d); // ex.: "1.23456789e9"
    let (m, e) = s.split_once('e').unwrap_or(("0", "0"));
    let mant = m.parse::<f64>().unwrap_or(0.0);
    let exp = e.parse::<f64>().unwrap_or(0.0);
    // Arredonda a mantissa para 8 casas (o original usa 0.00000000E0).
    let mant8 = (mant * 1e8).round() / 1e8;
    (mant8, exp)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Shift,
    Alpha,
    Hyp,
    Mode,
    B03,
    B04,
    B05,
    B06,
    B07,
    B08,
    B11,
    B12,
    B13,
    B14,
    B15,
    B16,
    B17,
    B18,
    B19,
    B20,
    B21,
    B22,
    B23,
    B24,
    B25,
    B26,
    B27,
    B28,
    B29,
    B30,
    B31,
    B32,
    B33,
    B34,
    B35,
    B36,
    B37,
    B38,
    B39,
    B40,
    B41,
    B42,
    B43,
    B44,
    B45,
    B46,
    B47,
    B48,
    B49,
    B50,
    VarX,
}

impl Calc {
    /// Mostra o dado ativo no modo SD (ou "No Data").
    fn show_sd_data(&mut self) {
        if self.dad.quant() > 0 {
            let v = self.dad.view_list();
            self.display = self.res_final(v);
            self.posicao_cursor = 0;
            self.tela = self.posicao_marcador(&self.display);
            self.tela_resultado = format!("Data     {}", self.dad.data_ativa + 1);
        } else {
            self.display.clear();
            self.posicao_cursor = 0;
            self.tela = self.posicao_marcador(&self.display);
            self.tela_resultado = "No Data".to_string();
        }
    }

    /// Função estatística do modo SD (token m/n/t/o/u/f/g).
    fn sd_stat(&mut self, token: char) {
        self.dad.editar = false;
        self.display.clear();
        self.adiciona_texto(&token.to_string());
        self.saida_static(token);
    }

    /// Processa um toque de tecla. Porte fiel dos handlers de `SecondActivity`.
    pub fn press(&mut self, key: Key) {
        // Menu de modo (MODE → 1:COMP / 2:SD), fiel à Casio.
        if self.mode_menu {
            match key {
                Key::B41 => {
                    self.mode = Mode::Comp;
                    self.mode_menu = false;
                    self.btn_ac();
                }
                Key::B42 => {
                    self.mode = Mode::Sd;
                    self.dad.reset();
                    self.dad.editar = false;
                    self.mode_menu = false;
                    self.btn_ac();
                }
                _ => self.mode_menu = false,
            }
            return;
        }
        match key {
            Key::Shift => {
                self.shift = !self.shift;
                self.alpha = false;
                if self.is_comp() {
                    return;
                }
                self.show_sd_data();
            }
            Key::Alpha => {
                self.alpha = !self.alpha;
                self.shift = false;
                if self.is_comp() {
                    return;
                }
                self.show_sd_data();
            }
            Key::Hyp => {
                if self.alpha {
                    self.alpha = false;
                    if self.sto {
                        self.storage('C');
                    } else {
                        self.adiciona_texto("C");
                    }
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.hyp_inv = !self.hyp_inv;
                    self.hyp = false;
                } else {
                    self.hyp = !self.hyp;
                    self.hyp_inv = false;
                }
                if self.is_comp() {
                    return;
                }
                self.show_sd_data();
            }
            Key::Mode => {
                if self.shift {
                    self.shift = false;
                    if self.is_comp() {
                        self.clear();
                    } else {
                        self.dad.reset();
                        self.display.clear();
                        self.posicao_cursor = 0;
                        self.tela = self.posicao_marcador(&self.display);
                        self.tela_resultado = "0".to_string();
                    }
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    self.tela_resultado = "Ver 18.022020".to_string();
                    return;
                }
                // MODE → menu de modo (fiel à Casio: 1:COMP 2:SD)
                self.mode_menu = true;
                self.tela_resultado = "1:COMP  2:SD".to_string();
            }
            Key::B03 => {
                if !self.is_comp() {
                    return;
                }
                if !self.shift {
                    self.adiciona_texto("l");
                } else {
                    self.shift = false;
                    self.adiciona_texto("!");
                }
            }
            Key::B04 => {
                if !self.is_comp() {
                    return;
                }
                if !self.shift {
                    self.adiciona_texto("I");
                } else {
                    self.shift = false;
                    self.adiciona_texto("P");
                }
            }
            Key::B05 => {
                if self.is_comp() {
                    self.ultimo_comando.clear();
                    if self.memoria_ativa < 9 {
                        self.memoria_ativa += 1;
                        self.display = self.memoria[self.memoria_ativa].clone();
                        self.posicao_cursor = 0;
                        self.tela = self.posicao_marcador(&self.display);
                    }
                } else {
                    self.dad.seta_cima();
                    self.show_sd_data();
                }
            }
            Key::B06 => {
                self.ultimo_comando.clear();
                self.posicao_cursor += 1;
                let len = self.display.chars().count();
                if self.posicao_cursor > len {
                    self.posicao_cursor = len;
                }
                self.tela = self.posicao_marcador(&self.display);
            }
            Key::B07 => {
                self.ultimo_comando.clear();
                if self.posicao_cursor > 0 {
                    self.posicao_cursor -= 1;
                }
                self.tela = self.posicao_marcador(&self.display);
            }
            Key::B08 => {
                if self.is_comp() {
                    self.ultimo_comando.clear();
                    if self.memoria_ativa >= 1 {
                        self.display = self.memoria[self.memoria_ativa - 1].clone();
                        self.memoria_ativa -= 1;
                        self.posicao_cursor = 0;
                        self.tela = self.posicao_marcador(&self.display);
                    }
                } else {
                    self.dad.seta_baixo();
                    self.show_sd_data();
                }
            }
            Key::B11 => {
                if !self.is_comp() {
                    return;
                }
                self.ultimo_comando.clear();
                if !self.shift {
                    self.display = "M".to_string();
                } else {
                    self.shift = false;
                    self.display = "N".to_string();
                }
                self.num = true;
                self.posicao_cursor = 0;
                self.tela = self.posicao_marcador(&self.display);
            }
            Key::B12 => {
                if !self.is_comp() {
                    return;
                }
                if !self.shift {
                    self.adiciona_texto("³");
                } else {
                    self.shift = false;
                    self.adiciona_texto("U");
                }
            }
            Key::B13 => {
                if !self.is_comp() {
                    self.sd_stat('m');
                    return;
                }
                if self.ultimo_comando.is_empty() {
                    let d = self.display.clone();
                    self.calcular(d);
                }
                if self.display == "_" {
                    self.result = 0.0;
                }
                self.erro = self.ctx.erro_trans;
                if self.erro != 0 || self.tt_exp >= 10 {
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.tela_resultado = fraction::fracao_geratriz_abc(self.result);
                } else {
                    self.tela_resultado = fraction::fracao_geratriz_bc(self.result);
                }
                self.posicao_cursor = 0;
                self.tela = self.posicao_marcador(&self.display);
            }
            Key::B14 => {
                if self.is_comp() {
                    self.adiciona_texto("V");
                } else {
                    self.sd_stat('t');
                }
            }
            Key::B15 => {
                if self.is_comp() {
                    self.adiciona_texto("²");
                }
            }
            Key::B16 => {
                if !self.is_comp() {
                    return;
                }
                if !self.shift {
                    self.adiciona_texto("^");
                } else {
                    self.shift = false;
                    self.adiciona_texto("Q");
                }
            }
            Key::B17 => {
                if !self.is_comp() {
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.adiciona_texto("10^");
                } else {
                    self.adiciona_texto("K");
                }
            }
            Key::B18 => {
                if !self.is_comp() {
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.adiciona_texto("e^");
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("e");
                } else {
                    self.adiciona_texto("L");
                }
            }
            Key::B19 => {
                if !self.is_comp() {
                    self.sd_stat('o');
                    return;
                }
                if self.sto {
                    self.storage('A');
                    return;
                }
                if self.shift {
                    self.shift = false;
                    if self.ultimo_comando == "enter" {
                        self.adiciona_texto("W");
                    }
                    let d = self.display.clone();
                    self.calcular(d);
                    self.to_graus();
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("A");
                } else {
                    if self.ultimo_comando == "enter" {
                        self.adiciona_texto("W");
                    }
                    let d = self.display.clone();
                    self.calcular(d);
                    self.to_radianos();
                }
            }
            Key::B20 => {
                if !self.is_comp() {
                    self.sd_stat('u');
                    return;
                }
                self.ultimo_comando.clear();
                if self.sto {
                    self.storage('B');
                    return;
                }
                if self.shift {
                    self.shift = false;
                    let d = self.display.clone();
                    self.calcular(d);
                    self.tela_resultado = format!("{} ", parser::graus(self.ctx.ultimo_valor));
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("B");
                } else if !self.gms {
                    self.display = "h".to_string();
                    self.gms = true;
                    self.num = true;
                    self.posicao_cursor = 0;
                    self.tela = self.posicao_marcador(&self.display);
                } else {
                    self.adiciona_texto("º");
                }
            }
            Key::B21 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('C');
                    return;
                }
                if self.shift {
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("C");
                } else {
                    self.adiciona_texto("J");
                }
            }
            Key::B22 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('D');
                    return;
                }
                if self.hyp {
                    self.hyp = false;
                    self.adiciona_texto("a");
                } else if self.hyp_inv {
                    self.hyp_inv = false;
                    self.adiciona_texto("d");
                } else if self.shift {
                    self.shift = false;
                    self.adiciona_texto("i");
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("D");
                } else {
                    self.adiciona_texto("S");
                }
            }
            Key::B23 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('E');
                    return;
                }
                if self.hyp {
                    self.hyp = false;
                    self.adiciona_texto("b");
                } else if self.hyp_inv {
                    self.hyp_inv = false;
                    self.adiciona_texto("f");
                } else if self.shift {
                    self.shift = false;
                    self.adiciona_texto("j");
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("E");
                } else {
                    self.adiciona_texto("R");
                }
            }
            Key::B24 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('F');
                    return;
                }
                if self.hyp {
                    self.hyp = false;
                    self.adiciona_texto("c");
                } else if self.hyp_inv {
                    self.hyp_inv = false;
                    self.adiciona_texto("g");
                } else if self.shift {
                    self.shift = false;
                    self.adiciona_texto("k");
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("F");
                } else {
                    self.adiciona_texto("T");
                }
            }
            Key::B25 => {
                if self.is_comp() {
                    self.sto = !self.sto;
                } else {
                    self.sd_stat('n');
                }
            }
            Key::B26 => {
                if !self.is_comp() {
                    self.sd_stat('f');
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.eng -= 1;
                    if self.eng < 0 {
                        self.eng = 0;
                    }
                } else if self.alpha {
                    self.alpha = false;
                } else {
                    self.eng += 1;
                    if self.eng > 5 {
                        self.eng = 5;
                    }
                }
                let d = self.display.clone();
                self.calcular(d);
            }
            Key::B27 => {
                if !self.is_comp() {
                    self.sd_stat('g');
                    return;
                }
                if self.sto {
                    self.storage('X');
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("X");
                } else {
                    self.adiciona_texto("(");
                }
            }
            Key::B28 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('Y');
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.adiciona_texto(",");
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("Y");
                } else {
                    self.adiciona_texto(")");
                }
            }
            Key::B29 => {
                if !self.is_comp() {
                    return;
                }
                if self.shift {
                    self.shift = false;
                    self.adiciona_texto(";");
                } else {
                    self.adiciona_texto(",");
                }
            }
            Key::B30 => {
                if self.is_comp() {
                    if self.sto {
                        self.storage('Z');
                        return;
                    }
                    if self.alpha {
                        self.alpha = false;
                        self.adiciona_texto("Z");
                        return;
                    }
                    if self.shift {
                        self.shift = false;
                        self.mem_menos();
                        return;
                    }
                    self.mem_mais();
                    return;
                }
                if self.shift {
                    let i = self.dad.data_ativa;
                    self.dad.apaga(i);
                    self.dad.seta_baixo();
                    self.show_sd_data();
                    return;
                }
                if self.alpha {
                    self.alpha = false;
                    if self.dad.quant() > 0 {
                        let d = self.display.clone();
                        self.calcular(d);
                        let r = self.result;
                        let i = self.dad.data_ativa;
                        self.dad.replace(i, r);
                        self.show_sd_data();
                    }
                    return;
                }
                if self.dad.editar {
                    let d = self.display.clone();
                    self.calcular(d);
                    self.dad.adicionar(self.result);
                    let n = self.dad.quant();
                    self.dad.data_ativa = n - 1;
                    self.show_sd_data();
                    return;
                }
                self.dad.editar = true;
                self.display.clear();
                self.posicao_cursor = 0;
                self.tela = self.posicao_marcador(&self.display);
                self.tela_resultado = "0".to_string();
            }
            Key::B31 => self.digit('7'),
            Key::B32 => self.digit('8'),
            Key::B33 => self.digit('9'),
            Key::B34 => {
                self.ultimo_comando.clear();
                self.apagar();
            }
            Key::B35 => self.btn_ac(),
            Key::B36 => self.digit('4'),
            Key::B37 => self.digit('5'),
            Key::B38 => self.digit('6'),
            Key::B39 => {
                if self.is_comp() {
                    self.adiciona_texto("x");
                }
            }
            Key::B40 => {
                if self.is_comp() {
                    self.adiciona_texto("/");
                }
            }
            Key::B41 => self.digit('1'),
            Key::B42 => self.digit('2'),
            Key::B43 => self.digit('3'),
            Key::B44 => {
                if self.is_comp() {
                    self.adiciona_texto("+");
                }
            }
            Key::B45 => {
                if !self.is_comp() {
                    return;
                }
                if self.sto {
                    self.storage('A');
                    return;
                }
                if self.shift {
                    self.shift = false;
                } else if self.alpha {
                    self.alpha = false;
                    self.adiciona_texto("A");
                } else {
                    self.adiciona_texto("-");
                }
            }
            Key::B46 => self.digit('0'),
            Key::B47 => {
                if self.is_comp() {
                    if self.shift {
                        self.shift = false;
                        self.adiciona_texto("r");
                        return;
                    }
                    if self.alpha {
                        self.alpha = false;
                        self.ultimo_comando.clear();
                        self.display = "s".to_string();
                        self.num = true;
                        self.posicao_cursor = 0;
                        self.tela = self.posicao_marcador(&self.display);
                        return;
                    }
                    self.shift = false;
                    self.alpha = false;
                    self.adiciona_texto(".");
                } else {
                    if !self.dad.editar {
                        self.dad.editar = true;
                        self.display.clear();
                    }
                    self.adiciona_texto(".");
                }
            }
            Key::B48 => {
                if !self.is_comp() {
                    return;
                }
                if !self.shift {
                    self.adiciona_texto("w");
                } else {
                    self.shift = false;
                    self.adiciona_texto("p");
                }
            }
            Key::B49 => {
                if self.is_comp() {
                    self.adiciona_texto("W");
                }
            }
            Key::B50 => {
                if !self.is_comp() {
                    return;
                }
                if "iSjRkTKLJUV".contains(self.display.as_str()) {
                    self.adiciona_texto("W");
                }
                let d = self.display.clone();
                self.calcular(d);
            }
            Key::VarX => {
                // variável X (memória) — inserida diretamente, sem passar pelo modificador ALPHA
                self.adiciona_texto("X");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc() -> Calc {
        Calc::default()
    }

    #[test]
    fn formatacao_decimal() {
        let mut c = calc();
        assert_eq!(c.res_final(2.0), "2.0");
        assert_eq!(c.res_final(0.5), "0.5");
        assert_eq!(c.res_final(1.0 / 3.0), "0.3333333");
        assert_eq!(c.res_final(123.456789), "123.456789");
        assert_eq!(c.res_final(123.4567891), "123.4567891");
    }

    #[test]
    fn separador_de_milhares() {
        let mut c = calc();
        assert_eq!(c.res_final(67108866.236068), "67 108 866.236068");
        assert_eq!(c.res_final(1234.5), "1 234.5");
        assert_eq!(c.res_final(999.0), "999.0");
        assert_eq!(c.res_final(-1234567.0), "-1 234 567.0");
        assert_eq!(separar_milhares("67108866"), "67 108 866");
    }

    #[test]
    fn formatacao_cientifica_e_eng() {
        let mut c = calc();
        assert_eq!(c.res_final(1e10), "1.0");
        assert!(c.expo);
        assert_eq!(c.tt_exp, 10);
    }

    #[test]
    fn limite_excedido() {
        let mut c = calc();
        c.result = 1e100;
        assert_eq!(c.res_final(1e100), "Limit exceeded");
    }

    #[test]
    fn calcular_fluxo_basico() {
        let mut c = calc();
        c.display = "1+3*5".to_string();
        c.calcular(c.display.clone());
        assert!((c.result - 16.0).abs() < 1e-9);
        assert_eq!(c.tela_resultado, "16.0");
        assert_eq!(c.memoria[0], "1+3*5");
    }

    #[test]
    fn adiciona_texto_e_cursor() {
        let mut c = calc();
        c.adiciona_texto("1");
        c.adiciona_texto("+");
        c.adiciona_texto("2");
        assert_eq!(c.display, "1+2");
    }

    #[test]
    fn substituir_x_por_asterisco() {
        let c = calc();
        assert_eq!(c.substituir("3x5"), "3*5");
        assert_eq!(c.substituir(">>A"), "A");
    }

    #[test]
    fn self_test_ok() {
        let mut c = calc();
        c.teste_click();
        assert_eq!(c.tela_resultado, "Test-Ok");
    }

    #[test]
    fn press_soma() {
        let mut c = calc();
        for k in [Key::B41, Key::B44, Key::B42, Key::B50] {
            c.press(k);
        }
        assert!((c.result - 3.0).abs() < 1e-9);
        assert_eq!(c.tela_resultado, "3.0");
    }

    #[test]
    fn press_raiz_quadrada() {
        let mut c = calc();
        for k in [Key::B14, Key::B33, Key::B50] {
            c.press(k);
        }
        assert!((c.result - 3.0).abs() < 1e-9);
    }

    #[test]
    fn press_shift_raiz_cubica() {
        let mut c = calc();
        c.press(Key::Shift);
        c.press(Key::B12); // U (∛)
        c.press(Key::B42); // 2
        c.press(Key::B31); // 7
        c.press(Key::B50);
        assert!((c.result - 3.0).abs() < 1e-9);
    }

    #[test]
    fn press_hyp_sinh_de_um() {
        let mut c = calc();
        c.press(Key::Hyp);
        c.press(Key::B22); // sin → sinh
        c.press(Key::B41); // 1
        c.press(Key::B50);
        assert!((c.result - 1.0_f64.sinh()).abs() < 1e-9);
    }

    #[test]
    fn press_shift_hyp_asinh_de_um() {
        let mut c = calc();
        c.press(Key::Shift);
        c.press(Key::Hyp); // hyp⁻¹
        c.press(Key::B24); // tan → atanh
        c.press(Key::B46); // 0
        c.press(Key::B50);
        assert!((c.result - 0.0).abs() < 1e-9);
    }

    #[test]
    fn press_sto_memoria_d() {
        let mut c = calc();
        c.press(Key::B37); // 5
        c.press(Key::B25); // STO
        c.press(Key::B22); // Sin → storage D
        assert!((c.ctx.men_d - 5.0).abs() < 1e-9);
        assert_eq!(c.tela, ">>D");
    }

    #[test]
    fn press_fracao_geratriz() {
        let mut c = calc();
        for k in [Key::B46, Key::B47, Key::B37, Key::B50, Key::B13] {
            c.press(k); // 0 . 5 = D/C
        }
        assert_eq!(c.tela_resultado, "1/2 ");
    }

    #[test]
    fn press_varx_usa_memoria_x() {
        let mut c = calc();
        c.ctx.men_x = 3.0;
        for k in [Key::B42, Key::B39, Key::VarX, Key::B50] {
            c.press(k); // 2 × X =
        }
        assert!((c.result - 6.0).abs() < 1e-9);
    }

    #[test]
    fn tela_formatada_mapeia_simbolos() {
        let mut c = calc();
        c.press(Key::B14); // √
        c.press(Key::B42); // 2
        assert!(c.tela_formatada().starts_with("√2"), "{}", c.tela_formatada());

        let mut c2 = calc();
        c2.press(Key::B23); // cos
        c2.press(Key::B43); // 3
        c2.press(Key::B46); // 0
        assert!(c2.tela_formatada().starts_with("cos30"), "{}", c2.tela_formatada());

        let mut c3 = calc();
        c3.press(Key::B12); // x³ → token ³
        c3.press(Key::B39); // × → token x
        assert!(c3.tela_formatada().contains("×"), "{}", c3.tela_formatada());
    }

    #[test]
    fn press_alpha_neg_minus_recalls_a() {
        let mut c = calc();
        c.press(Key::B43); // 3
        c.press(Key::B44); // + (operador)
        c.press(Key::Alpha);
        c.press(Key::B45); // alpha + (−) → insere "A"
        assert!(c.display.contains('A'), "display: {}", c.display);
    }

    #[test]
    fn sd_estatistica_mostra_valor() {
        let mut c = calc();
        c.mode = Mode::Sd;
        for v in [10.0, 20.0, 5.0, 15.0, 55.5, 100.0] {
            c.dad.adicionar(v);
        }
        c.press(Key::B25); // SD → n (média)
        assert_eq!(c.tela_resultado, "34.25");
    }

    #[test]
    fn mode_menu_entra_sd() {
        let mut c = calc();
        c.press(Key::Mode);
        assert_eq!(c.tela_resultado, "1:COMP  2:SD");
        c.press(Key::B42); // 2 → SD
        assert_eq!(c.mode, Mode::Sd);
        assert_eq!(c.dad.quant(), 0);
        assert_eq!(c.tela_resultado, "0.");
    }

    #[test]
    fn mode_menu_volta_comp() {
        let mut c = calc();
        c.mode = Mode::Sd;
        c.press(Key::Mode);
        c.press(Key::B41); // 1 → Comp
        assert_eq!(c.mode, Mode::Comp);
    }
}
