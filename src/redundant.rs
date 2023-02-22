// let S = Compartiment(POP_GROOTTE - 1, 'S'); // vatbaar
// let E = Compartiment(1, 'E'); // besmet
// let I = Compartiment(0, 'I'); // besmettelijk
// let T = Compartiment(0, 'T'); // thuis
// let Z = Compartiment(0, 'Z'); // resp
// let z = Compartiment(0, 'z'); // open
// let R = Compartiment(0, 'R'); // hersteld

// let compartimenten = vec![S, E, I, T, Z, z, R];

// let lambda = Verbinding::Proportioneel(&S, 0.01, &E);
// let i = Verbinding::Proportioneel(&E, 0.12, &I);
// let t1 = Verbinding::Proportioneel(&I, 0.33, &T);
// let t2 = Verbinding::Proportioneel(&E, 0.02, &T);
// let z1 = Verbinding::Proportioneel(&I, 0.02, &Z);
// let z2 = Verbinding::Proportioneel(&I, 0.08, &z);
// let b = Verbinding::Proportioneel(&Z, 0.15, &z);
// let w = Verbinding::Proportioneel(&z, 0.09, &Z);
// let r1 = Verbinding::Proportioneel(&z, 0.25, &R);
// let r2 = Verbinding::Proportioneel(&T, 0.25, &R);
// let vs = Verbinding::Proportioneel(&S, 0.01, &R);
// let ve = Verbinding::Proportioneel(&E, 0.01, &R);
// let iv = Verbinding::Proportioneel(&R, 0.01, &S);
// let g = Verbinding::Constant(100, &S);

// let nat_dood_comp = compartimenten.clone();
// let n = nat_dood_comp
//     .iter()
//     .map(|c| Verbinding::Sifon(-0.002, &c))
//     .collect::<Vec<_>>();

// let ziek_dood_comp = compartimenten.clone();
// let sigma = ziek_dood_comp
//     .iter()
//     .filter(|c| c.1 != 'S' && c.1 != 'R')
//     .map(|c| Verbinding::Sifon(-0.003, &c))
//     .collect::<Vec<_>>();

// let verbindingen = [
//     n,
//     sigma,
//     vec![lambda, i, t1, t2, z1, z2, b, w, r1, r2, vs, ve, iv, g],
// ]
// .concat();
// let mut snapshot = Model(compartimenten, verbindingen);

// let mut geschiedenis: Vec<Vec<Compartiment>> = vec![snapshot.0.clone()];

// for i in 0..TIJDSSTAPPEN {
//     snapshot.stap();
//     geschiedenis.push(snapshot.0.clone());
//     println!("{:>3}: {snapshot}", i);
// }

// let mut csv_writer = csv::Writer::from_path("out.csv")?;

// csv_writer.write_record(["S", "E", "I", "T", "Zr", "Zo", "R"])?;
// for rij in geschiedenis {
//     csv_writer.write_record(rij.iter().map(|e| e.0.to_string()))?;
// }
// csv_writer.flush()?;
