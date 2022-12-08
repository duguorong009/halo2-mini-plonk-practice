use std::marker::PhantomData;

use halo2_proofs::{halo2curves::FieldExt, plonk::*, poly::Rotation};

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

    fn configure(&self, meta: ConstraintSystem<F>) -> TutorialConfig {
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
        meta.enable_constant(o);

        meta.enable_constant(PI);

        meta.create_gate(
            || "plonk",
            |mut meta| {
                let l = meta.query_advice(l, Rotation::cur());
                let r = meta.query_advice(r, Rotation::cur());
                let o = meta.query_advice(o, Rotation::cur());

                let sl = meta.query_fixed(sl, Rotation::cur());
                let sr = meta.query_fixed(sr, Rotation::cur());
                let sm = meta.query_fixed(sm, Rotation::cur());
                let so = meta.query_fixed(so, Rotation::cur());
                let sc = meta.query_fixed(sc, Rotation::cur());

                vec![l * sl + r * sr + l * r * sm + (-F::one()) * o * so]
            },
        );

        meta.create_gate(
            || "public input",
            |mut meta| {
                let l = meta.query_advice(l, Rotation::cur());

                let sl = meta.query_fixed(sl, Rotation::cur());

                let PI = meta.query_instance(PI, Rotation::cur());

                vec![sl * (l - PI)]
            },
        );

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

impl<F: FieldExt> TutorialComposer<F> for TutorialChip<F> {}
