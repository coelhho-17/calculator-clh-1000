# Calc_SDE — Especificação de Comportamento (para porte em Rust)

Fonte: decompilação do APK `com.sdecalc.ricardo.myapplication` (versionCode/Name 19).
SHA-256 do APK: 1b7c00b767eb87d10a2175b810b6243881f75ffebf8447a018a3901be620de7d
Rótulo no manifest: "Calc_SDE" (o usuário o chama de "MyCalc" — mesmo app, único calc. custom instalado).

## 0. O que é

Um emulador de calculadora científica estilo **Casio fx-82MS / fx-350MS**, em Java puro
(sem código nativo — classes.dex único, sem lib/*.so). Toda a lógica é decompilável e
portável 1:1 para Rust.

Estrutura (9 classes):
- MainActivity      — constrói a UI programaticamente (ConstraintLayout, ~50 botões).
- SecondActivity    — estado + handlers de botão + display + formatação (2175 linhas).
- FourthActivity    — parser/avaliador de expressão + funções matemáticas + memórias.
- FgeraActivity     — gerador de fração (fração geratriz).
- Statistic         — lista de dados e estatísticas (média, variância, desvio, mediana).
- Financeira        — SAC e PRICE (financeiro; mal ligado, ver §9).
- AvancoActivity    — helpers de string/double.

## 1. Modelo de estado (SecondActivity)

- `display: String`  — buffer da expressão como **string de tokens** (ex.: "1+3x5").
  Cada função/constante é um **código de 1 caractere** (ver tabela). Não há árvore.
- `tela: String`      — display principal com marcador de cursor `_` inserido.
- `telaResultado`     — linha de resultado (ou mensagem/estado).
- `shift`, `alpha`    — flags de modificador (estilo Casio). Consumidos no próximo toque.
- `mode: "Comp" | "SD"` — modo cálculo vs estatística.
- `num`, `gms`, `sto`, `expo` — flags de estado.
- `result: Double`    — último resultado.
- `ultimoComando`     — "enter" ou "".
- `memoria[10]`       — pilha de histórico (últimos 10 displays), `memoriaAtiva` índice.
- `eng: 0..5`         — notação de engenharia.
- `erro: 0|1|2|3`     — 0=ok, 1="Error", 2="Limit Factor", 3="Limit exceeded".
- `TTexp`, `posicaoCursor`.

Registros de memória (FourthActivity): `menA..menF`, `menX`, `menY`, `menZ` (Double).
`menZ` também é o acumulador de M+ / M-.

## 2. Códigos de token (display string)

Operadores/posfixos reconhecidos pelo parser (FourthActivity.eval):
| token | char | significado |
|-------|------|-------------|
| + - * / | 43,45,42,47 | aritmética básica (x vira * no substituir) |
| ^   | 94  | potência (associativo à direita) |
| %   | 37  | pós-fixo: divide por 100 |
| P   | 80  | arranjo nPr (arranjo) |
| I   | 73  | combinação nCr (combinacao) |
| !   | 33  | fatorial (implementado como arranjo(n,n)) |
| Q   | 81  | raiz enésima: Q(n, base) → nroot(parseFactor, d) |
| ² ³ | 178,179 | quadrado / cubo |
| l   | 108 | inverso (^ -1) |
| w   | 119 | notação científica ×10^ (EXP) |

Funções/constantes (letras), todas em **graus** para trig:
| token | função | nota |
|-------|--------|------|
| V | √x | sqrt |
| U | ∛x | cbrt |
| S | sin | toRadians |
| R | cos | toRadians |
| T | tan | toRadians |
| i | asin | resultado em graus |
| j | acos | resultado em graus |
| k | atan | resultado em graus |
| K | log10 | |
| L | ln | |
| J | abs | |
| r | random | Math.random() [0,1) |
| W | Ans | ultimoValor |
| e | e | 2.718281828459045 |
| p | π | 3.141592653589793 |
| A B C D E F X Y Z | memória | mem_A()..mem_Z() |

Disambiguador: se a string INTEIRA não contém nenhum de `r W e p A..F X Y Z`,
então após ler uma letra o parser lê um fator como argumento (função). Se contém,
a letra é uma constante isolada. Isso permite "e^2" = e^2 e "V4" = √4.

`parseExpression` → `parseTerm` → `parseFactor` (recursivo descendente).
Erros/NaN: qualquer exceção → erroTrans=1, retorna 0.0.

## 3. Funções matemáticas (FourthActivity)

- `factorial(n)`: se `n > floor(n)` (não-inteiro) OU `n > 69` → erroTrans=2.
  senão recursivo n! = n * (n-1)!; 0! = 1. Negativos tratados como 0/1 (ver código).
- `combinacao(n,r)` = n!/(r!(n-r)!); `arranjo(n,r)` = n!/(n-r)!.
  Se n < r → erroTrans=2, retorna 1. Se n ≤ 0 → retorna 0.
- `nroot(base, indice)`: índice<0 → erro 1; 1→x; 2→sqrt; 3→cbrt; senão pow(x, 1/índice).
- `graus(d)`: converte decimal → "Dº M' S''" (DMS).
- `gmstograus("DºMºS")`: DMS → decimal (D + M/60 + S/3600).
- `RandomNumber()` = Math.random(); `getRandomInt(i,j)` (com swap de args).
- `valorANS()` = ultimoValor; `valor_e()`; `valor_p()`.
- `car(r,θ)`, `polar(x,y)` (em SecondActivity): conversão polar↔retangular.
  car: X = r·cos(θ), Y = r·sin(θ). polar: X = √(x²+y²), Y = atan(y/x) em graus.

## 4. Mapa de botões (label → comportamento)

Formato Casio: rótulo principal + rótulo secundário SHIFT/ALPHA acima.
| botão | normal | SHIFT | ALPHA | handler |
|-------|--------|-------|-------|---------|
| SHIFT | liga shift | — | — | btn1 |
| ALPHA | liga alpha | — | — | btn2 |
| MODE | menu "1:COMP 2:SD" | (CLR) | "Ver 18.022020" | btn9 |
| x⁻¹ | token l | ! (fatorial) | — | btn03 |
| nCp | token I (nCr) | P (nPr) | — | btn04 |
| Pol( | token M | token N | — | btn11 |
| x³ | token ³ | U (∛) | — | btn12 |
| D/C | fração geratriz | — | — | btn13 |
| √x | token V | — | — | btn14 |
| x² | token ² | — | — | btn15 |
| ^ | token ^ | Q (raiz n-ésima) | — | btn16 |
| log | token K | 10^ | — | btn17 |
| ln | token L | e^ | e | btn18 |
| rad | →radianos | →graus | token A | btn19 |
| º ' '' | GMS | graus() | token B | btn20 |
| abs | token J | — | token C | btn21 |
| Sin | token S | i (asin) | token D | btn22 |
| Cos | token R | j (acos) | token E | btn23 |
| Tan | token T | k (atan) | token F | btn24 |
| STO | liga sto | — | — | btn25 |
| ENG | eng+1 | eng-1 | — | btn26 |
| ( | ( | — | — | btn27 |
| ) | ) | — | token X | btn28 |
| % | % (ou , em num) | , | token Y | btn29 |
| M+ | memMmais | M- | — | btn30 |
| 7 8 9 | dígito | — | — | btn31..33 |
| DEL | apaga (backspace) | limpa à esquerda | — | btn34 |
| AC | reset total | — | — | btn35 |
| 4 5 6 | dígito | — | — | btn36..38 |
| × | token x | — | — | btn39 |
| / | token / | — | — | btn40 |
| 1 2 3 | dígito | — | — | btn41..43 |
| + | + | — | — | btn44 |
| - | - | — | teste_Click (self-test) | btn45 |
| 0 | 0 | — | — | btn46 |
| . | . | r (Ran#) | s (RInt) | btn47 |
| EXP | w (×10^) | p (π) | — | btn48 |
| Ans | token W | — | — | btn49 |
| = | calcular | — | — | btn50 |

Modo SD: botões reaproveitados — navegação de dados (setaCima/setaBaixo), apaga,
editar/adicionar, e funções estatísticas via `saidaStatic` com tokens m/n/o/t/u/f/g.

## 5. Display e formatação (resFinal / ajusteTela)

- Entrada: 50 chars máx. → "Limit".
- `ajusteTela`: rolagem horizontal com marcador `< ... >`; conta largura de tokens
  largos (h,G..V +2; i,j,k,o,u +3; r,s +4).
- `resFinal(d)`:
  - `validarMaxMin`: result > 1e99 OU < -1e50 → erro 3 ("Limit exceeded").
  - |d| > 1e9 ou < 1e-9 → notação científica "0.00000000E0" + ajuste por `eng`
    (ENG 0..5 → divisores 1e5,1e3,1,1e-3,1e-5,1e-8 e exp +6,+3,0,-3,-6,-9).
  - senão DecimalFormat "0.0#######" (até 7 casas fracionárias).
  - sempre substitui "," por ".".
- Erros → "Error" (1), "Limit Factor" (2), "Limit exceeded" (3).
- `substituir()`: "x"→"*", ">>"→"" (limpa marcadores de STO).

## 6. Memória

- `memoria()`: pilha de 10 — shift para a direita, `memoria[0]=display` (histórico).
- STO: calcula display e grava em menA..menZ; display mostra ">>A"..">>Z".
- M+/M-: menZ ±= result.
- Botões de navegação de histórico (btn05/btn08) percorrem `memoria`.

## 7. Modo SD (estatística)

- `Statistic` mantém `ArrayList<Double> dados` + `dataAtiva` (índice) + `editar`.
- Inicia vazio (o original pré-carregava uma amostra de dev; no porte: vazio, fiel à Casio).
- Funções: quant(), media(), somatotal(), somaQua(), desvioPadraoN1 (n-1),
  varianciaN1 (n-1), varianciaN (n), desvioPadraoN (n), mediana (ordena cópia).
- `saidaStatic(token)`: m=quant, n=media, t=mediana, o=desvio n-1, u=variância n-1,
  f=somatotal, g=somaQua.
  (Nota: no original, `saidaStatic` sobrescrevia o resultado com "No Data"
  incondicionalmente — bug. No porte foi CORRIGIDO: "No Data" só com lista vazia.)
- O modo SD era inacessível no original (MODE mostrava "NO MODE SD"; código morto).
  No porte, MODE abre o menu "1:COMP 2:SD" (fiel à fx-82MS), tornando o SD alcançável.

## 8. Fração geratriz (FgeraActivity)

- `ajusteFracao`: trunca para 8 casas, remove zeros finais.
- `fracaoBC` / `fracaoABC`: procura denominador d=2..9999 tal que d·fração ≈ inteiro
  (formato "0.00000" termina em 00000 ou 99999); reduz por GCD (FracaoReducao).
  BC → "n/d"; ABC → "int+n/d" (fração mista).

## 9. Financeiro (Financeira) — provavelmente não exposto na UI

- `funcoesTecnicas()` está VAZIO (stub).
- `sac(PV, taxa, n, amort)`: d/n + ((n - amort)+1) * taxa * (d/n).
- `price(PMT, taxa, n)`: PMT·(1+i)^n·i / ((1+i)^n - 1).
Não encontrei handlers de botão que chamem sac/price — tratar como não-exposto até
confirmação em runtime (C6).

## 10. Formas especiais (funcoesTecnicas em SecondActivity)

- `M(a,b)` → polar(x,y); `N(a,b)` → car(r,θ) — exibe X.
- `h DºMºS` → gmstograus.
- `s(a,b)` → inteiro aleatório (RInt), exibido com 1 casa ("%.1f").

## 11. Self-test (teste_Click)

Dispara por ALPHA + "-". Roda 5 asserts: "1+1"=2; "W"=2; "W2"→Error; "5!"=120;
"p3"→Error. 5/5 → "Test-Ok", senão "Test-Error".

## 12. Quirks a preservar (fidelidade)

- `verInteiro(d) = d > floor(d)` (não-inteiro) — semântica invertida do nome, mantida.
- `!` (fatorial) é implementado como `arranjo(n,n)` (nPr n), e `arranjo` retorna 0 quando
  n ≤ 0 — logo **"0!" = 0** (não 1). `factorial()` interno retorna 1 para 0, mas o operador
  "!" da entrada não passa por ele para n=0.
- Fatorial só aceita inteiro ≤ 69; combinação/arranjo retornam 1 com erro 2 quando n<r.
- "W2" após "W" = "W"+"2"? Não: "W" é Ans (constante), "W2" falha o parse → Error
  (o parser lê "W" como word, mas depois vem "2" → "Unexpected"). Preservar.
- "%" divide por 100 (não é módulo); "x" é multiplicação.
- Ordem dos posfixos em parseFactor é fixa (^, %, P, I, !, Q, ², ³, l, w) — só o
  primeiro que casar é aplicado.
- "Limit" ao passar 50 chars; "Limit exceeded" para >1e99 / <-1e50.
- Comma e ponto ambos aceitos como separador decimal (normalizados para ".").

## 13. Plano de porte (Rust)

1. Motor puro (sem UI): portar `FourthActivity.eval` + funções + memórias + Statistic
   + Fgera + DMS + resFinal. Testes unitários espelhando cada regra acima (inclusive
   os quirks do §12). `cargo test` verde = C5.
2. UI: grid de botões fiel ao layout Casio (rótulos + secundários SHIFT/ALPHA),
   display principal + linha de resultado + indicadores (S/A/STO/GMS/NUM/SD, E, eng).
3. Estado (SecondActivity) como struct que orquestra display → parser → resFinal.
