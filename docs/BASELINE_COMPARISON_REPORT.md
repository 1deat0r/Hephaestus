# Baseline comparison report - deterministic arms

Arms compare at matched envelopes: identical compute budget,
tool access, and model access. The base arm is the honest-negative
control; its low correctness is expected, not a finding.
No promotion gate ran: false promotions reads 0 by absence.
Correctness, tokens, cost, and promotion counts replay exactly;
latency, throughput, and memory are measured per run and vary.

baseline base: correctness 0.000000, tokens: 2975, cost 0.0, latency total 10 ms, throughput mean 858823.703 tok/s, peak memory max 4648960 bytes
candidate cand: correctness 1.000000, tokens: 2975, cost 0.0, latency total 11 ms, throughput mean 865125.341 tok/s, peak memory max 4648960 bytes

false promotions: 0 (no gate ran; count by absence).

## Method notes

- correctness: positive-margin rate on the fixture trace (actual runs).
- tokens: fixed-tokenizer counts over bytes actually run, not provider metering;
  counting runs outside the timed section, so throughput divides tokens by chain time only.
- provider spend: 0 - no metered provider exists in this path (absence, not a finding).
- latency: local wall time per run in nanoseconds, each run rounded up to whole ms, then summed.
- throughput: measured tokens per measured second from true nanoseconds;
  the latency total sums ceiled ms instead, so the two bases differ by rounding.
- peak memory: process-wide kernel VmHWM high-water sampled after each run, maxed
  (0 where unreadable); the mark never decreases, so per-run values are max-so-far
  with no per-run isolation, and the max equals the final sample.

## Runs

base runs: 10; candidate runs: 10.

## Run receipts

Each digest binds the subsampled input bytes only (not theta or arm),
so base and cand runs over one subsample share the digest by construction.

- base 0: 59aae7921c250f5d4368acfca7a9c8f0c7ea1ba871bd0a0b170c834e4d0895af
- base 1: e1d7d52ebc3517aa082d326401581207a9cf50a56de3fd24411d2489bc7f6bde
- base 2: 49f421e26b1d67cf02674315fc13e93694ed3f9df817f6fabde58955faa61ee4
- base 3: 07e520e94d71206aede30f165d280dd953849d42551087dab37f152bd36eacb7
- base 4: 866b4cb77e5bfffca862f87d0707b0cb5ef6558f512b33d75767dfe8a036ff72
- base 5: 335216e7361d6592e7e09af08005706fd95490febf371ea86847ffd8bbe63f33
- base 6: c104d91072c92eab6265ff4252286f4379f3e784e7896dd8f332fc36f0e7c2f0
- base 7: a8250b7173768958139c3219ed66e299cdff6fabade5cd3237634962f5bd330c
- base 8: 6ffa31b9a3c7a6d9f85d386909c5dea07cc940946477c72e32480d9a91c8aad2
- base 9: 481aad8f29926b918c1d1ea0e772f989ff6b1b1ae1d5ed03420d6bc0e27ce859
- cand 0: 59aae7921c250f5d4368acfca7a9c8f0c7ea1ba871bd0a0b170c834e4d0895af
- cand 1: e1d7d52ebc3517aa082d326401581207a9cf50a56de3fd24411d2489bc7f6bde
- cand 2: 49f421e26b1d67cf02674315fc13e93694ed3f9df817f6fabde58955faa61ee4
- cand 3: 07e520e94d71206aede30f165d280dd953849d42551087dab37f152bd36eacb7
- cand 4: 866b4cb77e5bfffca862f87d0707b0cb5ef6558f512b33d75767dfe8a036ff72
- cand 5: 335216e7361d6592e7e09af08005706fd95490febf371ea86847ffd8bbe63f33
- cand 6: c104d91072c92eab6265ff4252286f4379f3e784e7896dd8f332fc36f0e7c2f0
- cand 7: a8250b7173768958139c3219ed66e299cdff6fabade5cd3237634962f5bd330c
- cand 8: 6ffa31b9a3c7a6d9f85d386909c5dea07cc940946477c72e32480d9a91c8aad2
- cand 9: 481aad8f29926b918c1d1ea0e772f989ff6b1b1ae1d5ed03420d6bc0e27ce859

## Model arms: absent

No consented provider contract exists, so no model arm ran.
Consequence: this report backs no model-vs-deterministic claim.
Rerun with consented arms to extend it; this file then regenerates.
