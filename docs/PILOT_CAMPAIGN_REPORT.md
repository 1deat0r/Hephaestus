# Pilot campaign report - fixture batch

Preregistration: prereg-fixture-campaign-1

Intent: debugging batch — variance calibration only, not evidence

Batch: 20 missions (10 repositories x 2 arms); clustering unit: repository; plan budget: 20.

Predeclared metrics: per-arm mean and variance,
paired-difference variance across repositories, denominator
receipts with failures retained.

## Results

Theta shows 2 decimals and value 6 decimals for reading;
the receipt digest binds the full-precision run inputs, and
value equals the interval lower bound minus theta.

| mission | repository | arm | dropped | theta | interval | value | receipt |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | trace-repo-0 | cand | 0 | 0.25 | (0.580538, 0.703837) | 0.330538 | 59aae7921c250f5d4368acfca7a9c8f0c7ea1ba871bd0a0b170c834e4d0895af |
| 1 | trace-repo-0 | base | 0 | 10.00 | (0.580538, 0.703837) | -9.419462 | 59aae7921c250f5d4368acfca7a9c8f0c7ea1ba871bd0a0b170c834e4d0895af |
| 2 | trace-repo-1 | cand | 1 | 0.26 | (0.590790, 0.712273) | 0.330790 | e1d7d52ebc3517aa082d326401581207a9cf50a56de3fd24411d2489bc7f6bde |
| 3 | trace-repo-1 | base | 1 | 10.00 | (0.590790, 0.712273) | -9.409210 | e1d7d52ebc3517aa082d326401581207a9cf50a56de3fd24411d2489bc7f6bde |
| 4 | trace-repo-2 | cand | 2 | 0.27 | (0.601092, 0.720704) | 0.331092 | 49f421e26b1d67cf02674315fc13e93694ed3f9df817f6fabde58955faa61ee4 |
| 5 | trace-repo-2 | base | 2 | 10.00 | (0.601092, 0.720704) | -9.398908 | 49f421e26b1d67cf02674315fc13e93694ed3f9df817f6fabde58955faa61ee4 |
| 6 | trace-repo-3 | cand | 3 | 0.28 | (0.611394, 0.729117) | 0.331394 | 07e520e94d71206aede30f165d280dd953849d42551087dab37f152bd36eacb7 |
| 7 | trace-repo-3 | base | 3 | 10.00 | (0.611394, 0.729117) | -9.388606 | 07e520e94d71206aede30f165d280dd953849d42551087dab37f152bd36eacb7 |
| 8 | trace-repo-4 | cand | 4 | 0.29 | (0.621694, 0.737511) | 0.331694 | 866b4cb77e5bfffca862f87d0707b0cb5ef6558f512b33d75767dfe8a036ff72 |
| 9 | trace-repo-4 | base | 4 | 10.00 | (0.621694, 0.737511) | -9.378306 | 866b4cb77e5bfffca862f87d0707b0cb5ef6558f512b33d75767dfe8a036ff72 |
| 10 | trace-repo-5 | cand | 5 | 0.30 | (0.632053, 0.745898) | 0.332053 | 335216e7361d6592e7e09af08005706fd95490febf371ea86847ffd8bbe63f33 |
| 11 | trace-repo-5 | base | 5 | 10.00 | (0.632053, 0.745898) | -9.367947 | 335216e7361d6592e7e09af08005706fd95490febf371ea86847ffd8bbe63f33 |
| 12 | trace-repo-6 | cand | 6 | 0.31 | (0.642414, 0.754264) | 0.332414 | c104d91072c92eab6265ff4252286f4379f3e784e7896dd8f332fc36f0e7c2f0 |
| 13 | trace-repo-6 | base | 6 | 10.00 | (0.642414, 0.754264) | -9.357586 | c104d91072c92eab6265ff4252286f4379f3e784e7896dd8f332fc36f0e7c2f0 |
| 14 | trace-repo-7 | cand | 7 | 0.32 | (0.652775, 0.762608) | 0.332775 | a8250b7173768958139c3219ed66e299cdff6fabade5cd3237634962f5bd330c |
| 15 | trace-repo-7 | base | 7 | 10.00 | (0.652775, 0.762608) | -9.347225 | a8250b7173768958139c3219ed66e299cdff6fabade5cd3237634962f5bd330c |
| 16 | trace-repo-8 | cand | 8 | 0.33 | (0.663132, 0.770930) | 0.333132 | 6ffa31b9a3c7a6d9f85d386909c5dea07cc940946477c72e32480d9a91c8aad2 |
| 17 | trace-repo-8 | base | 8 | 10.00 | (0.663132, 0.770930) | -9.336868 | 6ffa31b9a3c7a6d9f85d386909c5dea07cc940946477c72e32480d9a91c8aad2 |
| 18 | trace-repo-9 | cand | 9 | 0.34 | (0.673560, 0.779242) | 0.333560 | 481aad8f29926b918c1d1ea0e772f989ff6b1b1ae1d5ed03420d6bc0e27ce859 |
| 19 | trace-repo-9 | base | 9 | 10.00 | (0.673560, 0.779242) | -9.326440 | 481aad8f29926b918c1d1ea0e772f989ff6b1b1ae1d5ed03420d6bc0e27ce859 |

## Variance (from actual runs)

- arm base: mean -9.373056, variance 0.000881
- arm cand: mean 0.331944, variance 0.000001
- paired-difference variance: 0.000825

Denominators: total missions 20; failures retained: 0.

## Cost

- plan budget reserved: 20 (fixture units).
- provider spend: 0 - the fixture chain runs no metered provider;
  this is absence of spend, not a measured zero-cost deployment.
- compute and wall time: not metered in this batch (explicit gap).

## Failures

No mission failed or blocked in this batch. The count is
recorded, not omitted: failures retained in the denominator
reads 0 above. Prospective live missions are out of scope for
this fixture report.
