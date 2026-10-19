//! Porte de `Statistic.java` — dados e estatísticas do modo SD.

/// Lista de dados + índice ativo + flag de edição (corresponde a `Statistic`).
#[derive(Clone)]
pub struct Statistic {
    pub dados: Vec<f64>,
    pub data_ativa: usize,
    pub editar: bool,
}

impl Default for Statistic {
    /// Inicia vazio (fiel à Casio; o app original pré-carregava uma amostra de dev).
    fn default() -> Self {
        Self {
            dados: Vec::new(),
            data_ativa: 0,
            editar: false,
        }
    }
}

impl Statistic {
    fn inteiro(d: f64) -> bool {
        (d as i64) as f64 == d
    }

    pub fn adicionar(&mut self, d: f64) {
        self.dados.push(d);
    }

    pub fn quant(&self) -> usize {
        self.dados.len()
    }

    pub fn reset(&mut self) {
        self.dados = Vec::new();
    }

    pub fn view_list(&self) -> f64 {
        self.dados[self.data_ativa]
    }

    pub fn seta_baixo(&mut self) {
        if self.data_ativa > 0 {
            self.data_ativa -= 1;
        }
    }

    pub fn seta_cima(&mut self) {
        let size = self.dados.len();
        if size > 0 {
            self.data_ativa += 1;
            if self.data_ativa >= size {
                self.data_ativa = size - 1;
            }
        }
    }

    pub fn apaga(&mut self, i: usize) {
        if self.quant() > 0 {
            self.dados.remove(i);
        }
    }

    pub fn replace(&mut self, i: usize, d: f64) {
        if self.quant() > 0 {
            self.dados[i] = d;
        }
    }

    pub fn media(&self) -> f64 {
        self.somatotal() / self.dados.len() as f64
    }

    pub fn somatotal(&self) -> f64 {
        self.dados.iter().sum()
    }

    pub fn soma_qua(&self) -> f64 {
        self.dados.iter().map(|x| x * x).sum()
    }

    pub fn variancia_n1(&self) -> f64 {
        let m = self.media();
        self.dados.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (self.dados.len() - 1) as f64
    }

    pub fn desvio_padrao_n1(&self) -> f64 {
        self.variancia_n1().sqrt()
    }

    pub fn variancia_n(&self) -> f64 {
        let m = self.media();
        self.dados.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / self.dados.len() as f64
    }

    pub fn desvio_padrao_n(&self) -> f64 {
        self.variancia_n().sqrt()
    }

    /// Mediana. Quirk preservado: ordena `dados` IN PLACE (o original faz
    /// `Collections.sort(this.dados)`), reordenando a lista.
    pub fn mediana(&mut self) -> f64 {
        let half = self.dados.len() as f64 / 2.0;
        let z_inteiro = Self::inteiro(half);
        self.dados
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if !z_inteiro {
            return self.dados[half as usize];
        }
        let i = half as usize;
        (self.dados[i - 1] + self.dados[i]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Statistic {
        let mut s = Statistic::default();
        for &v in &[10.0, 20.0, 5.0, 15.0, 55.5, 100.0] {
            s.adicionar(v);
        }
        s
    }

    #[test]
    fn default_vazio() {
        assert_eq!(Statistic::default().quant(), 0);
    }

    #[test]
    fn amostra_padrao() {
        let s = sample();
        assert_eq!(s.quant(), 6);
        assert!((s.somatotal() - 205.5).abs() < 1e-9);
        assert!((s.media() - 34.25).abs() < 1e-9);
        assert!((s.variancia_n1() - 1358.375).abs() < 1e-9);
        assert!((s.variancia_n() - 1131.9791666666667).abs() < 1e-6);
        assert!((s.desvio_padrao_n1() - 1358.375f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn mediana() {
        let mut s = sample();
        assert!((s.mediana() - 17.5).abs() < 1e-9); // (15+20)/2
    }

    #[test]
    fn navegacao_e_edicao() {
        let mut s = sample();
        assert!((s.view_list() - 10.0).abs() < 1e-9); // data_ativa = 0
        s.seta_cima();
        assert!((s.view_list() - 20.0).abs() < 1e-9);
        s.seta_cima();
        s.seta_baixo();
        assert!((s.view_list() - 20.0).abs() < 1e-9);
        s.replace(0, 99.0);
        assert!((s.dados[0] - 99.0).abs() < 1e-9);
        let n = s.quant();
        s.apaga(0);
        assert_eq!(s.quant(), n - 1);
    }
}
