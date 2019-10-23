use std::env;
use std::fs::File;

use rand::OsRng;

use filecoin_proofs::constants::*;
use filecoin_proofs::parameters::public_params;
use filecoin_proofs::types::*;
use storage_proofs::circuit::stacked::StackedCompound;
use storage_proofs::compound_proof::CompoundProof;
use storage_proofs::crypto::pedersen::JJ_PARAMS;
use storage_proofs::stacked::StackedDrg;

// Run this from the command-line, passing the path to the file to which the parameters will be written.
pub fn main() {
    pretty_env_logger::init_timed();

    let args: Vec<String> = env::args().collect();
    let out_file = &args[1];

    let public_params = public_params(
        PaddedBytesAmount::from(SectorSize(SECTOR_SIZE_ONE_KIB)),
        usize::from(PoRepProofPartitions(2)),
    );

    let circuit = <StackedCompound as CompoundProof<
        _,
        StackedDrg<DefaultTreeHasher, DefaultPieceHasher>,
        _,
    >>::blank_circuit(&public_params, &JJ_PARAMS);
    let mut params = phase21::MPCParameters::new(circuit).unwrap();

    let rng = &mut OsRng::new().unwrap();
    let hash = params.contribute(rng);

    {
        let circuit = <StackedCompound as CompoundProof<
            _,
            StackedDrg<DefaultTreeHasher, DefaultPieceHasher>,
            _,
        >>::blank_circuit(&public_params, &JJ_PARAMS);
        let contributions = params.verify(circuit).expect("parameters should be valid!");

        // We need to check the `contributions` to see if our `hash`
        // is in it (see above, when we first contributed)
        assert!(phase21::contains_contribution(&contributions, &hash));
    }

    let mut buffer = File::create(out_file).unwrap();
    params.write(&mut buffer).unwrap();
}
