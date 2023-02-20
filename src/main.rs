#![allow(non_snake_case)]

const TIJDSSTAPPEN: u64 = 300;
const POP_GROOTTE:  u64 = 100000;
const PG_LENGTE:  usize = 5;

extern crate csv;

#[derive(Clone, Copy)]
struct Compartiment(u64, char);

#[derive(Clone, Copy)]
enum Verbinding<'a> {
    Constant(i64, &'a Compartiment),
    Sifon(f64, &'a Compartiment),
    Proportioneel(&'a Compartiment, f64, &'a Compartiment)
}

struct Snapshot(Vec<Compartiment>);

impl Snapshot {
    fn stap(&mut self, verbindingen: &Vec<Verbinding>) -> Self {
        let oorspr = self.0.clone(); // hiernaar verwijzen in de berekeningen
        let mut offsets: Vec<(char, i64)> = self.0.clone()
                                              .into_iter()
                                              .map(|e| (e.1, 0))
                                              .collect();

        for v in verbindingen {
            match v {
                Verbinding::Constant(t, comp) => {
                    let index = offsets.iter()
                                       .map(|e| e.0)
                                       .position(|x| x == comp.1)
                                       .expect("verwijzing naar onbestaand compartiment");
                    offsets[index].1 += t;
                },
                Verbinding::Sifon(factor, comp) => {
                    let index = offsets.iter()
                                       .map(|e| e.0)
                                       .position(|x| x == comp.1)
                                       .expect("verwijzing naar onbestaand compartiment");
                    offsets[index].1 += (oorspr.iter()
                                           .find(|e| e.1 == comp.1)
                                           .expect("verwijzing naar onbestaand compartiment")
                                           .0 as f64 * factor) as i64;
                }
                Verbinding::Proportioneel(bron, factor, doel) => {
                    let werkwaarde = oorspr.iter()
                                           .find(|e| e.1 == bron.1)
                                           .expect("verwijzing naar onbestaand compartiment")
                                           .0;

                    let diff = (werkwaarde as f64 * factor) as i64;
                    let bron_index = offsets.iter()
                                       .map(|e| e.0)
                                       .position(|x| x == bron.1)
                                       .expect("verwijzing naar onbestaand compartiment");
                    let doel_index = offsets.iter()
                                       .map(|e| e.0)
                                       .position(|x| x == doel.1)
                                       .expect("verwijzing naar onbestaand compartiment");

                    offsets[bron_index].1 -= diff;
                    offsets[doel_index].1 += diff;
                }
            }
        }

        for c in self.0.iter_mut() {
            c.0 = ((c.0 as i64) + offsets.iter().find(|e| e.0 == c.1).expect("je mam").1).max(0) as u64;
        }

        Self(self.0.clone())
    }
}

impl std::fmt::Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "TOTAAL: {:?}: ", &self.0.iter().map(|e| e.0).sum::<u64>())?;
        for c in &self.0 {
            write!(f, "{}: {:>w$}, ", c.1, c.0, w = PG_LENGTE)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let S = Compartiment(POP_GROOTTE - 1, 'S'); // vatbaar
    let E = Compartiment(              1, 'E'); // besmet
    let I = Compartiment(              0, 'I'); // besmettelijk
    let T = Compartiment(              0, 'T'); // thuis
    let Z = Compartiment(              0, 'Z'); // resp
    let z = Compartiment(              0, 'z'); // open
    let R = Compartiment(              0, 'R'); // hersteld

    let compartimenten = vec![S, E, I, T, Z, z, R];

    let lambda = Verbinding::Proportioneel(&S, 0.01, &E);
    let i      = Verbinding::Proportioneel(&E, 0.12, &I);
    let t1     = Verbinding::Proportioneel(&I, 0.33, &T);
    let t2     = Verbinding::Proportioneel(&E, 0.02, &T);
    let z1     = Verbinding::Proportioneel(&I, 0.02, &Z);
    let z2     = Verbinding::Proportioneel(&I, 0.08, &z);
    let b      = Verbinding::Proportioneel(&Z, 0.15, &z);
    let w      = Verbinding::Proportioneel(&z, 0.09, &Z);
    let r1     = Verbinding::Proportioneel(&z, 0.25, &R);
    let r2     = Verbinding::Proportioneel(&T, 0.25, &R);
    let vs     = Verbinding::Proportioneel(&S, 0.01, &R);
    let ve     = Verbinding::Proportioneel(&E, 0.01, &R);
    let iv     = Verbinding::Proportioneel(&R, 0.01, &S);
    let g      = Verbinding::Constant(100, &S);

    let nat_dood_comp = compartimenten.clone();
    let n = nat_dood_comp.iter().map(
        |c| Verbinding::Sifon(-0.002, &c)
    ).collect::<Vec<_>>();

    let ziek_dood_comp = compartimenten.clone();
    let sigma = ziek_dood_comp.iter().filter(|c| c.1 != 'S' && c.1 != 'R').map(
        |c| Verbinding::Sifon(-0.003, &c)
    ).collect::<Vec<_>>();

    let mut snapshot = Snapshot(compartimenten);
    let verbindingen = [n, sigma, vec![lambda, i, t1, t2, z1, z2, b, w, r1, r2, vs, ve, iv, g]].concat();

    let mut geschiedenis: Vec<Vec<Compartiment>> = vec![snapshot.0.clone()];

    for i in 0..TIJDSSTAPPEN {
        snapshot.stap(&verbindingen);
        geschiedenis.push(snapshot.0.clone());
        println!("{:>3}: {snapshot}", i);
    }

    let mut csv_writer = csv::Writer::from_path("out.csv")?;

    csv_writer.write_record(["S", "E", "I", "T", "Zr", "Zo", "R"])?;
    for rij in geschiedenis {
        csv_writer.write_record(rij.iter().map(|e| e.0.to_string()))?;
    }
    csv_writer.flush()?;

    Ok(())
}
