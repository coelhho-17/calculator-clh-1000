//! Porte fiel de `FourthActivity.eval` — parser recursivo descendente.
//!
//! A expressão é uma string de tokens de 1 caractere (ver SPEC.md §2):
//!   + - * / ^ %  (aritmética; `x` já foi substituído por `*` pelo chamador)
//!   P nPr · I nCr · ! fatorial · Q raiz enésima · ² ³ · l inverso · w ×10^
//!   V√ U∛ S sin R cos T tan i asin j acos k atan K log L ln J abs
//!   r rand W Ans e euler p π A..F X Y Z memória
//!
//! Trigo em graus. Disambiguador de função/constante replicado do original:
//! se a expressão inteira não contém nenhum de `r W e p A..F X Y Z`, uma letra
//! consome um fator como argumento; caso contrário, é constante isolada.

/// Constante de Euler e π, como no original (valores Java).
pub const E: f64 = 2.718281828459045;
pub const PI: f64 = 3.141592653589793;

/// Contexto de avaliação (memórias + estado). Corresponde a `FourthActivity`.
pub struct EvalContext {
    pub men_a: f64,
    pub men_b: f64,
    pub men_c: f64,
    pub men_d: f64,
    pub men_e: f64,
    pub men_f: f64,
    pub men_x: f64,
    pub men_y: f64,
    pub men_z: f64,
    pub ultimo_valor: f64,
    /// 0=ok, 1=Error, 2=Limit Factor, 3=Limit exceeded.
    pub erro_trans: i32,
    rng_state: u64,
}

impl Default for EvalContext {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        Self {
            men_a: 0.0,
            men_b: 0.0,
            men_c: 0.0,
            men_d: 0.0,
            men_e: 0.0,
            men_f: 0.0,
            men_x: 0.0,
            men_y: 0.0,
            men_z: 0.0,
            ultimo_valor: 0.0,
            erro_trans: 0,
            rng_state: seed | 1,
        }
    }
}

impl EvalContext {
    /// `Math.random()` — [0, 1). xorshift64* (não determinístico, como o original).
    pub fn random_number(&mut self) -> f64 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// `getRandomInt(i, i2)` — preserva o quirk do original: se `a <= b` retorna `a`;
    /// senão uniforme em [b, a].
    pub fn random_int(&mut self, a: i32, b: i32) -> i32 {
        if a <= b {
            return a;
        }
        let range = (a - b) as u32 + 1;
        let r = (self.random_number() * range as f64) as u32;
        b + r as i32
    }
}

/// `verInteiro(d)` — quirk preservado: retorna true se NÃO é inteiro.
fn ver_inteiro(d: f64) -> bool {
    d > d.floor()
}

/// Avalia `expr` (já com `x`→`*` substituído). Reseta `erro_trans` e retorna o
/// resultado (0.0 em caso de erro, com `erro_trans` = 1/2).
pub fn eval(expr: &str, ctx: &mut EvalContext) -> f64 {
    ctx.erro_trans = 0;
    let chars: Vec<char> = expr.chars().collect();
    let has_const = expr.contains('r')
        || expr.contains('W')
        || expr.contains('e')
        || expr.contains('p')
        || expr.contains('A')
        || expr.contains('B')
        || expr.contains('C')
        || expr.contains('D')
        || expr.contains('E')
        || expr.contains('F')
        || expr.contains('X')
        || expr.contains('Y')
        || expr.contains('Z');
    let mut p = Parser {
        chars,
        pos: -1,
        ch: -1,
        has_const,
    };
    match p.parse_top(ctx) {
        Ok(v) => v,
        Err(()) => {
            ctx.erro_trans = 1;
            0.0
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: isize,
    ch: i32,
    has_const: bool,
}

impl Parser {
    fn next_char(&mut self) {
        self.pos += 1;
        self.ch = if self.pos < self.chars.len() as isize {
            self.chars[self.pos as usize] as i32
        } else {
            -1
        };
    }

    fn eat(&mut self, c: char) -> bool {
        while self.ch == ' ' as i32 {
            self.next_char();
        }
        if self.ch != c as i32 {
            return false;
        }
        self.next_char();
        true
    }

    fn parse_top(&mut self, ctx: &mut EvalContext) -> Result<f64, ()> {
        self.next_char();
        let v = self.parse_expression(ctx)?;
        if self.pos as usize >= self.chars.len() {
            Ok(v)
        } else {
            Err(())
        }
    }

    fn parse_expression(&mut self, ctx: &mut EvalContext) -> Result<f64, ()> {
        let mut x = self.parse_term(ctx)?;
        loop {
            if self.eat('+') {
                x += self.parse_term(ctx)?;
            } else if self.eat('-') {
                x -= self.parse_term(ctx)?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_term(&mut self, ctx: &mut EvalContext) -> Result<f64, ()> {
        let mut x = self.parse_factor(ctx)?;
        loop {
            if self.eat('*') {
                x *= self.parse_factor(ctx)?;
            } else if self.eat('/') {
                x /= self.parse_factor(ctx)?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_factor(&mut self, ctx: &mut EvalContext) -> Result<f64, ()> {
        if self.eat('+') {
            return self.parse_factor(ctx);
        }
        if self.eat('-') {
            return Ok(-self.parse_factor(ctx)?);
        }
        let start = self.pos;
        let value: f64;
        if self.eat('(') {
            value = self.parse_expression(ctx)?;
            self.eat(')');
        } else {
            let c = self.ch;
            if (c >= '0' as i32 && c <= '9' as i32) || c == '.' as i32 {
                while (self.ch >= '0' as i32 && self.ch <= '9' as i32) || self.ch == '.' as i32 {
                    self.next_char();
                }
                let numstr: String = self.chars[start as usize..self.pos as usize]
                    .iter()
                    .collect();
                value = numstr.trim().parse::<f64>().map_err(|_| ())?;
            } else if (c >= 'A' as i32 && c <= 'Z' as i32)
                || (c >= 'a' as i32 && c <= 'z' as i32)
            {
                while (self.ch >= 'A' as i32 && self.ch <= 'Z' as i32)
                    || (self.ch >= 'a' as i32 && self.ch <= 'z' as i32)
                {
                    self.next_char();
                }
                let word: String = self.chars[start as usize..self.pos as usize]
                    .iter()
                    .collect();
                let mut factor = 0.0;
                if !self.has_const {
                    factor = self.parse_factor(ctx)?;
                }
                value = match word.as_str() {
                    "V" => factor.sqrt(),
                    "U" => factor.cbrt(),
                    "S" => factor.to_radians().sin(),
                    "R" => factor.to_radians().cos(),
                    "T" => factor.to_radians().tan(),
                    "i" => factor.asin().to_degrees(),
                    "j" => factor.acos().to_degrees(),
                    "k" => factor.atan().to_degrees(),
                    "K" => factor.log10(),
                    "L" => factor.ln(),
                    "J" => factor.abs(),
                    "a" => factor.sinh(),
                    "b" => factor.cosh(),
                    "c" => factor.tanh(),
                    "d" => factor.asinh(),
                    "f" => factor.acosh(),
                    "g" => factor.atanh(),
                    "r" => ctx.random_number(),
                    "W" => ctx.ultimo_valor,
                    "e" => E,
                    "p" => PI,
                    "A" => ctx.men_a,
                    "B" => ctx.men_b,
                    "C" => ctx.men_c,
                    "D" => ctx.men_d,
                    "E" => ctx.men_e,
                    "F" => ctx.men_f,
                    "X" => ctx.men_x,
                    "Y" => ctx.men_y,
                    "Z" => ctx.men_z,
                    _ => return Err(()),
                };
            } else {
                return Err(());
            }
        }
        // Operadores pós-fixos, na ordem exata do original (só o primeiro casa).
        if self.eat('^') {
            return Ok(value.powf(self.parse_factor(ctx)?));
        }
        if self.eat('%') {
            return Ok(value / 100.0);
        }
        if self.eat('P') {
            let arg = self.parse_factor(ctx)?;
            return Ok(arranjo(value, arg, ctx));
        }
        if self.eat('I') {
            let arg = self.parse_factor(ctx)?;
            return Ok(combinacao(value, arg, ctx));
        }
        if self.eat('!') {
            return Ok(arranjo(value, value, ctx));
        }
        if self.eat('Q') {
            let arg = self.parse_factor(ctx)?;
            return Ok(nroot(arg, value, ctx));
        }
        if self.eat('²') {
            return Ok(value * value);
        }
        if self.eat('³') {
            return Ok(value * value * value);
        }
        if self.eat('l') {
            return Ok(value.powf(-1.0));
        }
        if self.eat('w') {
            return Ok(value * 10f64.powf(self.parse_factor(ctx)?));
        }
        Ok(value)
    }
}

/// `factorial` — inteiro ≤ 69; não-inteiro ou >69 → erro_trans=2.
pub fn factorial(n: f64, ctx: &mut EvalContext) -> f64 {
    if ver_inteiro(n) {
        ctx.erro_trans = 2;
        return 1.0;
    }
    if n <= 69.0 {
        return if n >= 0.0 && n != 0.0 {
            n * factorial(n - 1.0, ctx)
        } else {
            1.0
        };
    }
    ctx.erro_trans = 2;
    1.0
}

/// `combinacao(n, r)` = nCr.
pub fn combinacao(n: f64, r: f64, ctx: &mut EvalContext) -> f64 {
    if n < r {
        ctx.erro_trans = 2;
        return 1.0;
    }
    if n > 0.0 {
        return factorial(n, ctx) / (factorial(r, ctx) * factorial(n - r, ctx));
    }
    0.0
}

/// `arranjo(n, r)` = nPr.
pub fn arranjo(n: f64, r: f64, ctx: &mut EvalContext) -> f64 {
    if n < r {
        ctx.erro_trans = 2;
        return 1.0;
    }
    if n > 0.0 {
        return factorial(n, ctx) / factorial(n - r, ctx);
    }
    0.0
}

/// `nroot(radicando, indice)` — radicando negativo → erro_trans=1.
pub fn nroot(radicando: f64, indice: f64, ctx: &mut EvalContext) -> f64 {
    ctx.erro_trans = 0;
    if radicando < 0.0 {
        ctx.erro_trans = 1;
        return 0.0;
    }
    if indice == 1.0 {
        return radicando;
    }
    if indice == 2.0 {
        return radicando.sqrt();
    }
    if indice == 3.0 {
        return radicando.cbrt();
    }
    radicando.powf(1.0 / indice)
}

/// `stringtodouble` — aceita vírgula como separador decimal.
pub fn string_to_double(s: &str) -> f64 {
    s.replace(',', ".").parse::<f64>().unwrap_or(0.0)
}

/// Formatação `DecimalFormat("0.00##")` (mín. 2, máx. 4 casas, round-half-even).
pub fn fmt_0_00xx(x: f64) -> String {
    let mut s = format!("{:.4}", x);
    if let Some(dot) = s.find('.') {
        let int_part = &s[..dot];
        let mut frac = s[dot + 1..].to_string();
        while frac.len() > 2 && frac.ends_with('0') {
            frac.pop();
        }
        s = format!("{}.{}", int_part, frac);
    }
    s
}

/// `graus(d)` — decimal → "Dº M' S''".
pub fn graus(d: f64) -> String {
    let mut g = d.trunc() as i64;
    let m_full = ((d * 1000.0 - d.trunc() * 1000.0) / 1000.0) * 60.0;
    let mut m = m_full.trunc() as i64;
    let mut s = (m_full - m as f64) * 60.0;
    if fmt_0_00xx(s) == "60.00" {
        m += 1;
        s = 0.0;
    }
    if m == 60 {
        g += 1;
        m = 0;
    }
    format!("{}º {}' {}''", g, m, fmt_0_00xx(s))
}

/// `gmstograus` — "DºMºS" → decimal.
pub fn gmstograus(s: &str) -> f64 {
    let mut t = format!("{}º00º00º00", s);
    while t.contains("ºº") {
        t = t.replace("ºº", "º");
    }
    while t.contains("..") {
        t = t.replace("..", ".");
    }
    let parts: Vec<&str> = t.split('º').collect();
    string_to_double(parts[0]) + string_to_double(parts[1]) / 60.0 + string_to_double(parts[2]) / 3600.0
}

/// `polar(x, y)` → (r, θ em graus).
pub fn polar(x: f64, y: f64) -> (f64, f64) {
    ((x * x + y * y).sqrt(), (y / x).atan().to_degrees())
}

/// `car(r, θ)` → (x, y).
pub fn car(r: f64, theta: f64) -> (f64, f64) {
    (r * theta.to_radians().cos(), r * theta.to_radians().sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(expr: &str) -> (f64, i32) {
        let mut ctx = EvalContext::default();
        let v = eval(expr, &mut ctx);
        (v, ctx.erro_trans)
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "esperado {} obtido {}",
            expected,
            actual
        );
    }

    #[test]
    fn aritmetica_basica() {
        close(e("1+1").0, 2.0);
        close(e("1+3*5").0, 16.0);
        close(e("8/9").0, 8.0 / 9.0);
        close(e("(1+2)*3").0, 9.0);
        close(e("2^3").0, 8.0);
        close(e("2^3^2").0, 512.0); // associativo à direita
        close(e("-5+3").0, -2.0);
        close(e("50%").0, 0.5); // pós-fixo % divide por 100
    }

    #[test]
    fn fatorial_e_combinatoria() {
        close(e("5!").0, 120.0);
        close(e("0!").0, 0.0); // quirk: "!" = arranjo(n,n), e arranjo(0,0)=0 (não 1)
        close(e("5P2").0, 20.0); // arranjo
        close(e("5I2").0, 10.0); // combinação
        assert_eq!(e("70!").1, 2); // >69 → Limit Factor
        assert_eq!(e("5.5!").1, 2); // não-inteiro → Limit Factor
    }

    #[test]
    fn raizes() {
        close(e("V9").0, 3.0);
        close(e("U27").0, 3.0);
        close(e("3Q8").0, 2.0); // índice antes do Q, radicando depois
        close(e("2Q9").0, 3.0);
        assert_eq!(e("3Q-8").1, 1); // radicando negativo
        assert!(e("V-1").0.is_nan()); // sqrt(-1) = NaN (sem erro)
    }

    #[test]
    fn trig_em_graus() {
        close(e("S30").0, 0.5);
        close(e("R60").0, 0.5);
        close(e("T45").0, 1.0);
        close(e("i0.5").0, 30.0);
        close(e("k1").0, 45.0);
    }

    #[test]
    fn log_e_abs() {
        close(e("K100").0, 2.0);
        close(e("L2").0, 2.0f64.ln());
        close(e("J-5").0, 5.0);
    }

    #[test]
    fn constantes_e_memorias() {
        let (v, _) = e("p");
        close(v, PI);
        close(e("e").0, E);
        let mut ctx = EvalContext::default();
        ctx.ultimo_valor = 5.0;
        ctx.men_a = 7.0;
        assert_eq!(eval("W", &mut ctx), 5.0);
        assert_eq!(eval("A", &mut ctx), 7.0);
    }

    #[test]
    fn notacao_cientifica() {
        close(e("5w3").0, 5000.0);
        close(e("2w-3").0, 0.002);
        close(e("10^3").0, 1000.0);
        close(e("e^2").0, E * E);
    }

    #[test]
    fn quirks_preservados() {
        assert_eq!(e("W2").1, 1); // "W" (Ans) seguido de dígito → Error
        assert_eq!(e("p3").1, 1); // constante seguida de dígito → Error
    }

    #[test]
    fn dms_e_gms() {
        assert_eq!(graus(1.5), "1º 30' 0.00''");
        close(gmstograus("1º30º0"), 1.5);
    }

    #[test]
    fn random_no_intervalo() {
        let mut ctx = EvalContext::default();
        for _ in 0..100 {
            let r = ctx.random_number();
            assert!((0.0..1.0).contains(&r));
        }
    }
}
