#![allow(non_snake_case)]

use eval::{to_value, ExecOptions, Expr};
use itertools::{iproduct, Itertools};
use std::collections::{HashMap, HashSet};

const TIJDSSTAPPEN: u64 = 300;
const POP_GROOTTE: u64 = 100000;
const PG_LENGTE: usize = 5;

extern crate csv;

#[derive(Clone, Copy, Debug)]
struct Compartiment(u64, char);

#[derive(Clone, Copy, Debug)]
enum Verbinding<'a> {
    Constant(i64, &'a Compartiment),
    Fractie(f64, &'a Compartiment),
    Proportioneel(&'a Compartiment, f64, &'a Compartiment),
    Overdracht(&'a Compartiment, u64, &'a Compartiment),
}

struct Model<'a>(Vec<Compartiment>, Vec<Verbinding<'a>>);

impl<'a> Model<'a> {
    fn stap(&mut self) -> Self {
        let oorspr = self.0.clone(); // hiernaar verwijzen in de berekeningen
        let mut offsets: Vec<(char, i64)> = self.0.clone().into_iter().map(|e| (e.1, 0)).collect();
        for v in &self.1 {
            match v {
                Verbinding::Constant(t, comp) => {
                    let index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == comp.1)
                        .expect("verwijzing naar onbestaand compartiment");
                    offsets[index].1 += t;
                }
                Verbinding::Fractie(factor, comp) => {
                    let index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == comp.1)
                        .expect("verwijzing naar onbestaand compartiment");
                    offsets[index].1 += (oorspr
                        .iter()
                        .find(|e| e.1 == comp.1)
                        .expect("verwijzing naar onbestaand compartiment")
                        .0 as f64
                        * factor) as i64;
                }
                Verbinding::Proportioneel(bron, factor, doel) => {
                    let werkwaarde = oorspr
                        .iter()
                        .find(|e| e.1 == bron.1)
                        .expect("verwijzing naar onbestaand compartiment")
                        .0;

                    let diff = (werkwaarde as f64 * factor) as i64;
                    let bron_index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == bron.1)
                        .expect("verwijzing naar onbestaand compartiment");
                    let doel_index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == doel.1)
                        .expect("verwijzing naar onbestaand compartiment");

                    offsets[bron_index].1 -= diff;
                    offsets[doel_index].1 += diff;
                }
                Verbinding::Overdracht(bron, hoeveelheid, doel) => {
                    let bron_index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == bron.1)
                        .expect("verwijzing naar onbestaand compartiment");
                    let doel_index = offsets
                        .iter()
                        .map(|e| e.0)
                        .position(|x| x == doel.1)
                        .expect("verwijzing naar onbestaand compartiment");

                    offsets[bron_index].1 -= *hoeveelheid as i64;
                    offsets[doel_index].1 += *hoeveelheid as i64;
                }
            }
        }

        for c in self.0.iter_mut() {
            c.0 = ((c.0 as i64) + offsets.iter().find(|e| e.0 == c.1).expect("je mam").1).max(0)
                as u64;
        }

        Self(self.0.clone(), self.1.clone())
    }
}

impl<'a> std::fmt::Display for Model<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "TOTAAL: {:?}: ",
            &self.0.iter().map(|e| e.0).sum::<u64>()
        )?;
        for c in &self.0 {
            write!(f, "{}: {:>w$}, ", c.1, c.0, w = PG_LENGTE)?;
        }
        Ok(())
    }
}

// TODO: na een stap worden $(uitdrukkingen) nog niet herberekend.
// mss aanpassing aan Verbinding zodat alleen de nodigen worden herberekend?

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zcm_bestand =
        std::fs::read_to_string("model.zcm").expect("kon modelbeschrijving niet inladen");
    let bestand_tokens = zcm_bestand
        .lines()
        .map(|l| l.split_once("//").unwrap_or((l, "")).0)
        .filter(|l| !l.chars().all(|c| c.is_whitespace()))
        .map(|l| {
            l.trim_end().strip_suffix(';').expect(&format!(
                "syntaxisfout: regel {} eindigt niet met een puntkomma.",
                l
            ))
        })
        .map(|c| c.split(" ").filter(|s| s != &"").collect::<Vec<_>>())
        .collect::<Vec<_>>();
    println!("{:?}", bestand_tokens);
    let compartimenten = &bestand_tokens
        .iter()
        .filter(|t| t[0] == "def")
        .map(|t| {
            Compartiment(
                t.get(2)
                    .unwrap_or(&"0")
                    .parse()
                    .expect("ongeldige getaldefinitie"),
                t[1].chars()
                    .next()
                    .expect("compartimentdefinitie zonder naam"),
            )
        })
        .collect::<Vec<_>>();

    println!("{:?}", compartimenten);

    let compartiment_namen = compartimenten
        .iter()
        .map(|c| c.1)
        .collect::<HashSet<char>>();

    let constanten = &bestand_tokens
        .iter()
        .filter(|t| t[0] == "const")
        .map(|t| (t[1], t[2].parse::<f64>().expect("ongeldige getaldefinitie")))
        .collect::<HashMap<&str, f64>>();

    println!("{:?}", constanten);

    let verbindingen = &bestand_tokens
        .iter()
        .filter(|t| t[1] == ">")
        .map(|t| {
            //TODO REKENING HOUDEN MET $(uitdrukkingen)
            let mut gebruik_absoluut = false;
            let waarde = if constanten.contains_key(t[3]) {
                // een constante werd gebruikt
                *constanten.get(t[3]).unwrap()
            } else if t[3].chars().next().unwrap() == '$' {
                // een $(uitdrukking) werd gebruikt
                // we moeten dus een absolute hoeveelheid gebruiken
                gebruik_absoluut = true;
                let volledig = &t[3..].concat();
                let uitdrk = volledig
                    .strip_prefix("$(")
                    .unwrap()
                    .strip_suffix(")")
                    .unwrap();

                // het kan zijn dat een uitdrukking voor een constante gebruikt wordt,
                // en in dat geval is het sneller om die waarde direct te parsen.

                let mss_parse = uitdrk.parse::<f64>();

                if mss_parse.is_ok() {
                    mss_parse.unwrap()
                } else {
                    let ctx = constanten
                        .iter()
                        .map(|(k, v)| (String::from(*k), to_value(v)))
                        .chain(
                            compartimenten
                                .iter()
                                .map(|c| (String::from(c.1), to_value(c.0))),
                        )
                        .collect::<HashMap<String, eval::Value>>();

                    ExecOptions::new(&Expr::new(uitdrk))
                        .contexts(&[ctx])
                        .exec()
                        .expect(&format!("kon uitdrukking {} niet parseren.", uitdrk))
                        .as_f64()
                        .expect(&format!(
                            "uitdrukking {} retourneerde geen f64-castbare waarde",
                            uitdrk
                        ))
                }
            } else {
                t[3].parse::<f64>().unwrap()
            };
            let bronnen = t[0].chars();
            if bronnen.clone().all(|c| compartiment_namen.contains(&c)) {
                // we hebben een of meerdere bronnen
                let bronnen_compart_refs =
                    bronnen.map(|b| compartimenten.iter().find(|c| c.1 == b).unwrap());

                let doelen = t[2].chars();

                if doelen.clone().all(|c| compartiment_namen.contains(&c)) {
                    // proportioneel
                    let doelen_compart_refs =
                        doelen.map(|b| compartimenten.iter().find(|c| c.1 == b).unwrap());
                    //cartesisch product nemen dus

                    if gebruik_absoluut {
                        return iproduct!(bronnen_compart_refs, doelen_compart_refs)
                            .map(|(b, d)| Verbinding::Overdracht(b, waarde as u64, d))
                            .collect::<Vec<Verbinding>>();
                    } else {
                        return iproduct!(bronnen_compart_refs, doelen_compart_refs)
                            .map(|(b, d)| Verbinding::Proportioneel(b, waarde, d))
                            .collect::<Vec<Verbinding>>();
                    }
                } else if doelen.clone().next().unwrap() == '-' {
                    // we gaan van een of meerdere bronnen naar het niets.

                    if gebruik_absoluut {
                        return bronnen_compart_refs
                            .map(|b| Verbinding::Constant(-waarde as i64, b))
                            .collect::<Vec<Verbinding>>();
                    } else {
                        return bronnen_compart_refs
                            .map(|b| Verbinding::Fractie(-waarde, b))
                            .collect::<Vec<Verbinding>>();
                    }
                } else {
                    panic!(
                        "ongeldige verbindingsdefinitie: ongeldig doel in {}",
                        doelen.as_str()
                    );
                }
            } else if bronnen.clone().next().unwrap() == '-' {
                // de bron is het niets

                let doelen = t[2].chars();

                if doelen.clone().all(|c| compartiment_namen.contains(&c)) {
                    let doelen_compart_refs =
                        doelen.map(|b| compartimenten.iter().find(|c| c.1 == b).unwrap());
                    if gebruik_absoluut {
                        return doelen_compart_refs
                            .map(|d| Verbinding::Constant(waarde as i64, d))
                            .collect::<Vec<Verbinding>>();
                    } else {
                        return doelen_compart_refs
                            .map(|d| Verbinding::Fractie(waarde, d))
                            .collect::<Vec<Verbinding>>();
                    }
                } else if doelen.clone().next().unwrap() == '-' {
                    eprintln!("onzinnige verbindingsdefinitie - > -");
                    return vec![];
                } else {
                    panic!(
                        "ongeldige verbindingsdefinitie: ongeldig doel in {}",
                        doelen.as_str()
                    );
                }
            } else {
                panic!(
                    "ongeldige verbingingsdefinitie: ongeldige bron in {}",
                    bronnen.as_str()
                );
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    println!("{:?}", verbindingen);

    let model = Model(compartimenten.clone(), verbindingen.clone());

    Ok(())
}
