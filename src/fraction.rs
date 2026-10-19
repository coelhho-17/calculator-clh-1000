//! Porte de `FgeraActivity` — gerador de fração geratriz (decimal → fração).
//!
//! Formato de saída: BC → "n/d" (ou fração mista implícita "a b/c" reduzida a
//! numerador/denominador), ABC → "int+n/d". Segue fielmente a lógica do original,
//! incluindo o quirk de entradas negativas (a extração da parte inteira ignora o
//! sinal, produzindo resultado inválido — igual ao original).
#![allow(unused_assignments)] // init morto espelha o `String str2 = str` do original

use crate::parser::string_to_double;

/// `Inteira(str)` = String.valueOf((int) stringtodouble(str)).
fn inteira_str(s: &str) -> String {
    (string_to_double(s) as i64).to_string()
}

/// `Mid(str, i, i2)` = substring(i, i2) — byte-slice (strings ASCII).
fn mid(s: &str, i: usize, i2: usize) -> &str {
    &s[i..i2]
}

/// `String.valueOf(double)` aproximado (Java). Integral → "N.0"; decimal curto;
/// científico com 'E' para magnitude extrema (corner raro no gerador de fração).
fn java_value_of(d: f64) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if d == 0.0 {
        return if d.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let a = d.abs();
    if a >= 1e7 || a < 1e-3 {
        let s = format!("{:e}", d);
        if let Some((m, e)) = s.split_once('e') {
            return format!("{}E{}", m, e.parse::<i32>().unwrap_or(0));
        }
    }
    let s = format!("{}", d);
    if s.contains('.') {
        s
    } else {
        format!("{}.0", s)
    }
}

/// `ajusteFracao` — trunca para 8 casas removendo zeros finais quando longo.
fn ajuste_fracao(d: f64) -> String {
    let mut s = java_value_of(d);
    if s.len() > 10 {
        let mut m = format!("{:.8}", string_to_double(&s));
        while m.ends_with('0') {
            m.pop();
        }
        s = m;
    }
    s.replace(',', ".")
}

/// `fracao` — extrai os dígitos fracionários de um número formatado.
fn fracao(s: &str) -> String {
    let r = s.replace(',', ".");
    let int_len = inteira_str(&r).len();
    let mid_part = mid(&r, int_len, r.len());
    let length2 = mid_part.len();
    if length2 <= 1 {
        return "0".to_string();
    }
    let l2 = if length2 > 10 { 10 } else { length2 };
    mid(mid_part, 1, l2).to_string()
}

/// `fracaoReal` — isola a parte fracionária como "0.xxx".
fn fracao_real(s: &str) -> String {
    let r = s.replace(',', ".");
    let int_len = inteira_str(&r).len();
    format!("0{}", mid(&r, int_len, r.len()))
}

/// `FracaoReducao` — reduz "num/den" por MDC (Euclides).
fn fracao_reducao(s: &str) -> String {
    let parts: Vec<&str> = s.split('/').collect();
    let mut num = string_to_double(parts[0]);
    let mut den = string_to_double(parts[1]);
    loop {
        let rem = num % den;
        if rem == 0.0 {
            let n2 = string_to_double(parts[0]) / den;
            let d2 = string_to_double(parts[1]) / den;
            return format!("{}/{}", format!("{:.0}", n2), format!("{:.0}", d2));
        }
        num = den;
        den = rem;
    }
}

fn fracao_simples_bc(s: &str) -> String {
    fracao_reducao(&format!("{}/1000", format!("{:.0}", string_to_double(s) * 1000.0)))
}

fn fracao_simples_abc(s: &str) -> String {
    let int_part = inteira_str(s);
    let fr = fracao_real(s);
    let s2 = format!("{}/1000", format!("{:.0}", string_to_double(&fr) * 1000.0));
    if int_part == "0" {
        fracao_reducao(&s2)
    } else {
        format!("{}+{}", int_part, fracao_reducao(&s2))
    }
}

fn fracao_bc(s: &str) -> String {
    let fr = fracao_real(s);
    if fr == "0.0" {
        return s.to_string();
    }
    if fr.len() <= 7 {
        return if s.len() < 8 {
            fracao_simples_bc(s)
        } else {
            s.to_string()
        };
    }
    let d = string_to_double(&fr);
    let mut s2 = String::new();
    let mut d2 = 1.0;
    loop {
        d2 += 1.0;
        s2 = format!("{:.5}", d * d2);
        let ff = fracao(&s2);
        if ff == "00000" || ff == "99999" {
            break;
        }
        if d2 == 9999.0 {
            return s.to_string();
        }
    }
    let int_part = inteira_str(s);
    let s3 = format!("{}/{}", s2, format!("{}", d2));
    if int_part == "0" {
        return fracao_reducao(&s3);
    }
    let num = string_to_double(&int_part) * d2 + string_to_double(&s2);
    fracao_reducao(&format!("{}/{}", num, d2))
}

fn fracao_abc(s: &str) -> String {
    let fr = fracao_real(s);
    if fr == "0.0" {
        return s.to_string();
    }
    if fr.len() <= 7 {
        return if s.len() < 8 {
            fracao_simples_abc(s)
        } else {
            s.to_string()
        };
    }
    let d = string_to_double(&fr);
    let mut s2 = String::new();
    let mut d2 = 1.0;
    loop {
        d2 += 1.0;
        s2 = format!("{:.5}", d * d2);
        let ff = fracao(&s2);
        if ff == "00000" || ff == "99999" {
            break;
        }
        if d2 == 9999.0 {
            return s.to_string();
        }
    }
    let int_part = inteira_str(s);
    let s3 = format!("{}/{}", s2, format!("{}", d2));
    if int_part == "0" {
        return fracao_reducao(&s3);
    }
    format!("{}+{}", int_part, fracao_reducao(&s3))
}

/// `fracaoGeratrizBC(d)`.
pub fn fracao_geratriz_bc(d: f64) -> String {
    let s = fracao_bc(&ajuste_fracao(d));
    if s.len() >= 10 {
        s
    } else {
        format!("{} ", s)
    }
}

/// `fracaoGeratrizABC(d)`.
pub fn fracao_geratriz_abc(d: f64) -> String {
    let s = fracao_abc(&ajuste_fracao(d));
    if s.len() >= 10 {
        s
    } else {
        format!("{} ", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fracoes_simples() {
        assert_eq!(fracao_geratriz_bc(0.5), "1/2 ");
        assert_eq!(fracao_geratriz_bc(2.5), "5/2 ");
        assert_eq!(fracao_geratriz_abc(2.5), "2+1/2 ");
        assert_eq!(fracao_geratriz_bc(0.25), "1/4 ");
    }

    #[test]
    fn dizima_periodica() {
        assert_eq!(fracao_geratriz_bc(1.0 / 3.0), "1/3 ");
        assert_eq!(fracao_geratriz_bc(2.0 / 3.0), "2/3 ");
    }
}
