use std::marker::PhantomData;

use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};

#[derive(Debug, Clone)]
struct TutorialConfig {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,

    sl: Column<Fixed>,
    sr: Column<Fixed>,
    so: Column<Fixed>,
    sm: Column<Fixed>,
    sc: Column<Fixed>,

    PI: Column<Instance>,
}

#[derive(Debug)]
struct TutorialChip<F: FieldExt> {
    config: TutorialConfig,
    marker: PhantomData<F>,
}

trait TutorialComposer<F: FieldExt> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error>;
}

impl<F: FieldExt> TutorialChip<F> {
    fn construct(config: TutorialConfig) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> TutorialConfig {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        let sl = meta.fixed_column();
        let sr = meta.fixed_column();
        let sm = meta.fixed_column();
        let so = meta.fixed_column();
        let sc = meta.fixed_column();

        let PI = meta.instance_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        meta.enable_equality(PI);

        meta.create_gate("plonk", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let sl = meta.query_fixed(sl, Rotation::cur());
            let sr = meta.query_fixed(sr, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());
            let so = meta.query_fixed(so, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm - o * so + sc]
        });

        TutorialConfig {
            l,
            r,
            o,
            sl,
            sr,
            so,
            sm,
            sc,
            PI,
        }
    }
}

impl<F: FieldExt> TutorialComposer<F> for TutorialChip<F> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        let mut values = None;
        layouter.assign_region(
            || "multiply",
            |mut region| {
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|x| x.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|x| x.1),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|x| x.2),
                )?;

                region.assign_fixed(|| "m", self.config.sm, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::one()))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        let mut values = None;
        layouter.assign_region(
            || "add",
            |mut region| {
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|x| x.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|x| x.1),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|x| x.2),
                )?;

                region.assign_fixed(|| "l", self.config.sl, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "r", self.config.sr, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::one()))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error> {
        layouter.assign_region(|| "copy values", |mut region| region.constrain_equal(a, b))
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell, self.config.PI, row)
    }
}

#[derive(Debug, Clone, Default)]
struct TutorialCircuit<F: FieldExt> {
    x: Value<F>,
    y: Value<F>,
    constant: F,
}

impl<F: FieldExt> Circuit<F> for TutorialCircuit<F> {
    type Config = TutorialConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        TutorialChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cs = TutorialChip::construct(config);

        let x: Value<Assigned<F>> = self.x.into();
        let y: Value<Assigned<F>> = self.y.into();
        let consty = Assigned::from(self.constant);

        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        cs.copy(&mut layouter, a0, b0)?;

        let (a1, b1, c1) = cs.raw_multiply(&mut layouter, || y.map(|y| (y, y, y * y)))?;
        cs.copy(&mut layouter, a1, b1)?;

        let (a2, b2, c2) = cs.raw_add(&mut layouter, || {
            x.zip(y).map(|(x, y)| (x * x, y * y, x * x + y * y))
        })?;
        cs.copy(&mut layouter, a2, c0)?;
        cs.copy(&mut layouter, b2, c1)?;

        let (a3, b3, c3) = cs.raw_add(&mut layouter, || {
            x.zip(y)
                .map(|(x, y)| (x * x + y * y, consty, x * x + y * y + consty))
        })?;
        cs.copy(&mut layouter, a3, c2)?;
        cs.expose_public(&mut layouter, b3, 0)?;

        cs.expose_public(&mut layouter, c3, 1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    use super::*;
    #[test]
    fn test_plonk() {
        let k = 4;

        let x = Fp::from(3);
        let y = Fp::from(5);
        let constant = Fp::from(1);

        let z = x.square() + y.square() + constant;

        let circuit = TutorialCircuit {
            x: Value::known(x),
            y: Value::known(y),
            constant,
        };

        let public_input = vec![constant, z];

        let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        prover.assert_satisfied();

        // Plot the circuit
        use plotters::prelude::*;
        let root = BitMapBackend::new("plonk-test-layout.png", (1024, 1024)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("plonk test Layout", ("sans-serif", 60))
            .unwrap();
        halo2_proofs::dev::CircuitLayout::default()
            .render(k, &circuit, &root)
            .unwrap();
    }
}
