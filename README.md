# CALCULATOR CLH-1000

Calculadora científica de desktop para Windows, reimplementada em **Rust + egui/eframe**.
Motor fiel (parser, matemática, estatística, frações) com UI de fidelidade industrial ao
leiaute de uma calculadora científica clássica de 2 linhas (estilo Casio fx-82MS).

## Funcionalidades

- Trigonometria em graus: sin, cos, tan + inversas (sin⁻¹, cos⁻¹, tan⁻¹).
- Hiperbólicas: sinh, cosh, tanh + inversas (via tecla `hyp`).
- log, ln, √, ∛, ⁿ√, x², x³, x⁻¹, xʸ, x!, nCr, nPr.
- Fração geratriz (D/C), graus/min/seg (GMS↔DMS), polar↔retangular (Pol(/Rec()).
- Memórias A–F/X/Y/Z + M+/M−, histórico (▲/▼), modo SD (estatística: média, desvio, mediana).
- Constantes π, e, Ans; gerador de aleatórios Ran#.
- Notação científica e de engenharia (ENG).
- Visor com símbolos fiéis (√, sin, cos, ∛, log, ln, nCr, π…) e separador de milhar.

## Atalhos de teclado

| Tecla | Ação |
|---|---|
| `0`–`9` | números |
| `+` `-` | soma / subtração (negação) |
| `*` ou `×` | multiplicação |
| `/` ou `÷` | divisão |
| `.` | ponto decimal |
| `,` | vírgula (separador de argumentos) |
| `=` ou Enter | igual |
| `^` | potência (xʸ) |
| `(` `)` | parênteses |
| `x` | variável X (memória) |
| Backspace | DEL |
| Esc | AC |
| `H` | hyp (hiperbólicas) |

## Build

```bash
cargo build --release
```

Gera `target/release/calc_sde.exe` (subsistema GUI, sem console).

## Testes

```bash
cargo test --lib   # 37 testes
```

## Estrutura

- `src/main.rs` — UI egui/eframe 0.37 (design system, REPLAY, visor LCD, teclado).
- `src/calc.rs` — máquina de estados (teclas, memórias, modo SD, hiperbólicas).
- `src/parser.rs` — parser de expressões (gramática, funções, constantes).
- `src/fraction.rs` — fração geratriz (D/C).
- `src/statistic.rs` — estatística (média, variância, desvio, mediana).
- `SPEC.md` — especificação de comportamento (inclui quirks preservados do original).

## Fontes (embutidas)

- **DSEG7** (visor LCD de 7 segmentos) — por keshikan, SIL Open Font License 1.1.
- **Barlow Condensed** (logo) — projeto Barlow, SIL Open Font License 1.1.

## Notas

Projeto de reimplementação independente, sem afiliação com qualquer marca de calculadora.
O motor preserva quirks históricos do aparelho de referência (ex.: `0!` = 0).
