/*
 * Tulip Indicators
 * https://tulipindicators.org/
 * Copyright (c) 2010-2026 Tulip Charts LLC
 *
 * This file is part of Tulip Indicators.
 */

#include "../indicators.h"

int ti_ma_dispatch_start(int period, int ma_type);
int ti_ma_dispatch_run(int size, TI_REAL const *input, int period, int ma_type, TI_REAL *output);

static int ti_macdext_parse_options(
    TI_REAL const *options,
    int *fast_period,
    int *fast_type,
    int *slow_period,
    int *slow_type,
    int *signal_period,
    int *signal_type
) {
    *fast_period = (int)options[0];
    *fast_type = (int)options[1];
    *slow_period = (int)options[2];
    *slow_type = (int)options[3];
    *signal_period = (int)options[4];
    *signal_type = (int)options[5];

    if (*fast_period < 2 || *fast_period != options[0]) return TI_INVALID_OPTION;
    if (*slow_period < 2 || *slow_period != options[2]) return TI_INVALID_OPTION;
    if (*signal_period < 1 || *signal_period != options[4]) return TI_INVALID_OPTION;
    if (*fast_type < 0 || *fast_type > 8 || *fast_type != options[1]) return TI_INVALID_OPTION;
    if (*slow_type < 0 || *slow_type > 8 || *slow_type != options[3]) return TI_INVALID_OPTION;
    if (*signal_type < 0 || *signal_type > 8 || *signal_type != options[5]) return TI_INVALID_OPTION;

    if (*slow_period < *fast_period) {
        int tmp_period = *slow_period;
        int tmp_type = *slow_type;
        *slow_period = *fast_period;
        *slow_type = *fast_type;
        *fast_period = tmp_period;
        *fast_type = tmp_type;
    }

    return TI_OKAY;
}

static int ti_macdext_uses_plain_ema(int fast_type, int slow_type, int signal_type) {
    return fast_type == 1 && slow_type == 1 && signal_type == 1;
}

int ti_macdext_start(TI_REAL const *options) {
    int fast_period, fast_type, slow_period, slow_type, signal_period, signal_type;
    if (ti_macdext_parse_options(options, &fast_period, &fast_type, &slow_period, &slow_type, &signal_period, &signal_type) != TI_OKAY) {
        return -1;
    }
    if (ti_macdext_uses_plain_ema(fast_type, slow_type, signal_type)) {
        const TI_REAL macd_options[] = {(TI_REAL)fast_period, (TI_REAL)slow_period, (TI_REAL)signal_period};
        return ti_macd_start(macd_options);
    }

    int lookback = ti_ma_dispatch_start(fast_period, fast_type);
    const int slow_lookback = ti_ma_dispatch_start(slow_period, slow_type);
    const int signal_lookback = ti_ma_dispatch_start(signal_period, signal_type);
    if (slow_lookback > lookback) lookback = slow_lookback;
    return lookback + signal_lookback;
}

int ti_macdext(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input = inputs[0];
    TI_REAL *macd = outputs[0];
    TI_REAL *signal = outputs[1];
    TI_REAL *hist = outputs[2];

    int fast_period, fast_type, slow_period, slow_type, signal_period, signal_type;
    int rc = ti_macdext_parse_options(options, &fast_period, &fast_type, &slow_period, &slow_type, &signal_period, &signal_type);
    if (rc != TI_OKAY) return rc;
    if (ti_macdext_uses_plain_ema(fast_type, slow_type, signal_type)) {
        const TI_REAL macd_options[] = {(TI_REAL)fast_period, (TI_REAL)slow_period, (TI_REAL)signal_period};
        return ti_macd(size, inputs, macd_options, outputs);
    }

    int fast_lookback = ti_ma_dispatch_start(fast_period, fast_type);
    int slow_lookback = ti_ma_dispatch_start(slow_period, slow_type);
    const int signal_lookback = ti_ma_dispatch_start(signal_period, signal_type);
    int lookback_largest = fast_lookback;
    if (slow_lookback > lookback_largest) lookback_largest = slow_lookback;
    const int lookback_total = lookback_largest + signal_lookback;
    if (size <= lookback_total) return TI_OKAY;

    TI_REAL *fast = malloc((unsigned int)(size - fast_lookback) * sizeof(TI_REAL));
    TI_REAL *slow = malloc((unsigned int)(size - slow_lookback) * sizeof(TI_REAL));
    TI_REAL *diff = malloc((unsigned int)(size - lookback_largest) * sizeof(TI_REAL));
    TI_REAL *signal_values = malloc((unsigned int)(size - lookback_total) * sizeof(TI_REAL));
    if (!fast || !slow || !diff || !signal_values) {
        free(fast);
        free(slow);
        free(diff);
        free(signal_values);
        return TI_OUT_OF_MEMORY;
    }

    rc = ti_ma_dispatch_run(size, input, fast_period, fast_type, fast);
    if (rc == TI_OKAY) rc = ti_ma_dispatch_run(size, input, slow_period, slow_type, slow);
    if (rc != TI_OKAY) {
        free(fast);
        free(slow);
        free(diff);
        free(signal_values);
        return rc;
    }

    for (int actual_index = lookback_largest; actual_index < size; ++actual_index) {
        diff[actual_index - lookback_largest] =
            fast[actual_index - fast_lookback] - slow[actual_index - slow_lookback];
    }

    rc = ti_ma_dispatch_run(size - lookback_largest, diff, signal_period, signal_type, signal_values);
    if (rc != TI_OKAY) {
        free(fast);
        free(slow);
        free(diff);
        free(signal_values);
        return rc;
    }

    const int output_len = size - lookback_total;
    for (int out_index = 0; out_index < output_len; ++out_index) {
        const TI_REAL macd_value = diff[signal_lookback + out_index];
        const TI_REAL signal_value = signal_values[out_index];
        macd[out_index] = macd_value;
        signal[out_index] = signal_value;
        hist[out_index] = macd_value - signal_value;
    }

    free(fast);
    free(slow);
    free(diff);
    free(signal_values);
    return TI_OKAY;
}
