//! Print observation receipts for every corpus world so fixture files can
//! pin them. Run: cargo run -p synthetic-evaluator --example corpus_receipt

fn main() {
    for entry in synthetic_evaluator::load_corpus() {
        let observations = entry.world.sample_observations();
        let receipt = synthetic_evaluator::observation_receipt(&observations);
        println!(
            "{} {} stored={}",
            entry.id, receipt, entry.observation_receipt_sha256
        );
    }
}
