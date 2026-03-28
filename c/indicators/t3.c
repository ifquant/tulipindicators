/*
 * Tulip Indicators
 * https://tulipindicators.org/
 * Copyright (c) 2010-2026 Tulip Charts LLC
 *
 * This file is part of Tulip Indicators.
 */

#include "../indicators.h"

int ti_t3_start(TI_REAL const *options) {
    const int period = (int)options[0];
    const TI_REAL vfactor = options[1];

    if (period < 2 || period != options[0]) return -1;
    if (vfactor < 0.0 || vfactor > 1.0) return -1;
    return 6 * (period - 1);
}

int ti_t3(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input = inputs[0];
    const int period = (int)options[0];
    const TI_REAL vfactor = options[1];
    TI_REAL *output = outputs[0];

    if (period < 2 || period != options[0]) return TI_INVALID_OPTION;
    if (!isfinite(vfactor) || vfactor < 0.0 || vfactor > 1.0) return TI_INVALID_OPTION;

    const int lookback = 6 * (period - 1);
    if (size <= lookback) return TI_OKAY;

    const TI_REAL k = 2.0 / (period + 1.0);
    const TI_REAL one_minus_k = 1.0 - k;

    int today = 0;
    TI_REAL temp = input[today++];
    for (int i = period - 1; i > 0; --i) temp += input[today++];
    TI_REAL e1 = temp / period;

    temp = e1;
    for (int i = period - 1; i > 0; --i) {
        e1 = k * input[today++] + one_minus_k * e1;
        temp += e1;
    }
    TI_REAL e2 = temp / period;

    temp = e2;
    for (int i = period - 1; i > 0; --i) {
        e1 = k * input[today++] + one_minus_k * e1;
        e2 = k * e1 + one_minus_k * e2;
        temp += e2;
    }
    TI_REAL e3 = temp / period;

    temp = e3;
    for (int i = period - 1; i > 0; --i) {
        e1 = k * input[today++] + one_minus_k * e1;
        e2 = k * e1 + one_minus_k * e2;
        e3 = k * e2 + one_minus_k * e3;
        temp += e3;
    }
    TI_REAL e4 = temp / period;

    temp = e4;
    for (int i = period - 1; i > 0; --i) {
        e1 = k * input[today++] + one_minus_k * e1;
        e2 = k * e1 + one_minus_k * e2;
        e3 = k * e2 + one_minus_k * e3;
        e4 = k * e3 + one_minus_k * e4;
        temp += e4;
    }
    TI_REAL e5 = temp / period;

    temp = e5;
    for (int i = period - 1; i > 0; --i) {
        e1 = k * input[today++] + one_minus_k * e1;
        e2 = k * e1 + one_minus_k * e2;
        e3 = k * e2 + one_minus_k * e3;
        e4 = k * e3 + one_minus_k * e4;
        e5 = k * e4 + one_minus_k * e5;
        temp += e5;
    }
    TI_REAL e6 = temp / period;

    const TI_REAL v2 = vfactor * vfactor;
    const TI_REAL c1 = -(v2 * vfactor);
    const TI_REAL c2 = 3.0 * (v2 - c1);
    const TI_REAL c3 = -6.0 * v2 - 3.0 * (vfactor - c1);
    const TI_REAL c4 = 1.0 + 3.0 * vfactor - c1 + 3.0 * v2;

    *output++ = c1 * e6 + c2 * e5 + c3 * e4 + c4 * e3;

    while (today < size) {
        e1 = k * input[today++] + one_minus_k * e1;
        e2 = k * e1 + one_minus_k * e2;
        e3 = k * e2 + one_minus_k * e3;
        e4 = k * e3 + one_minus_k * e4;
        e5 = k * e4 + one_minus_k * e5;
        e6 = k * e5 + one_minus_k * e6;
        *output++ = c1 * e6 + c2 * e5 + c3 * e4 + c4 * e3;
    }

    return TI_OKAY;
}
