use std::marker::PhantomData;

use halo2_proofs::{circuit::*, halo2curves::FieldExt, plonk::*, poly::Rotation};

mod simple;

fn main() {
    println!("Hello, world!");
}

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

    fn configure(&self, meta: &mut ConstraintSystem<F>) -> TutorialConfig {
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

            vec![l.clone() * sl + r.clone() * sr + l * r * sm - o * so]
        });

        meta.create_gate("public input", |meta| {
            let l = meta.query_advice(l, Rotation::cur());

            let sl = meta.query_fixed(sl, Rotation::cur());

            let PI = meta.query_instance(PI, Rotation::cur());

            vec![sl * (l - PI)]
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
        let values = f();
        layouter.assign_region(
            || "add",
            |mut region| {
                let lhs =
                    region.assign_advice(|| "lhs", self.config.l, 0, || values.map(|x| x.0))?;
                let rhs =
                    region.assign_advice(|| "rhs", self.config.r, 0, || values.map(|x| x.1))?;
                let out =
                    region.assign_advice(|| "out", self.config.o, 0, || values.map(|x| x.2))?;

                region.assign_fixed(
                    || "enable multiplication",
                    self.config.sm,
                    0,
                    || Value::known(F::one()),
                )?;
                region.assign_fixed(
                    || "enable out",
                    self.config.so,
                    0,
                    || Value::known(F::one()),
                )?;

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
        let values = f();
        layouter.assign_region(
            || "add",
            |mut region| {
                let lhs =
                    region.assign_advice(|| "lhs", self.config.l, 0, || values.map(|x| x.0))?;
                let rhs =
                    region.assign_advice(|| "rhs", self.config.r, 0, || values.map(|x| x.1))?;
                let out =
                    region.assign_advice(|| "out", self.config.o, 0, || values.map(|x| x.2))?;

                region.assign_fixed(
                    || "enable lhs",
                    self.config.sl,
                    0,
                    || Value::known(F::one()),
                )?;
                region.assign_fixed(
                    || "enable rhs",
                    self.config.sr,
                    0,
                    || Value::known(F::one()),
                )?;
                region.assign_fixed(
                    || "enable out",
                    self.config.so,
                    0,
                    || Value::known(F::one()),
                )?;

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
