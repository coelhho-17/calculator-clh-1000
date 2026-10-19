//! calculator-clh-1000 — calculadora científica de desktop (estilo clássico de 2 linhas).
//!
//! Estrutura:
//! - `parser`: avaliador de expressão (port de FourthActivity.eval) + funções
//!   matemáticas (fatorial, arranjo, combinação, raiz enésima, DMS, polar/retangular).
//!
//! Camadas seguintes: formatação (resFinal/ENG), estatística, fração geratriz e a
//! máquina de estado da calculadora (SecondActivity) — acrescentadas nas próximas
//! etapas.

pub mod parser;
pub mod fraction;
pub mod statistic;
pub mod calc;

pub use calc::{Calc, Key, Mode};
pub use parser::{eval, EvalContext};
