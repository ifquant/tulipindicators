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

int ti_mavp_start(TI_REAL const *options) {
    const int min_period = (int)options[0];
    const int max_period = (int)options[1];
    const int ma_type = (int)options[2];

    if (min_period < 2 || min_period != options[0]) {
        return -1;
    }
    if (max_period < min_period || max_period != options[1]) {
        return -1;
    }
    if (ma_type < 0 || ma_type > 8 || ma_type != options[2]) {
        return -1;
    }

    return ti_ma_dispatch_start(max_period, ma_type);
}

int ti_mavp(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input = inputs[0];
    const TI_REAL *periods = inputs[1];
    const int min_period = (int)options[0];
    const int max_period = (int)options[1];
    const int ma_type = (int)options[2];
    TI_REAL *output = outputs[0];

    if (min_period < 2 || min_period != options[0]) return TI_INVALID_OPTION;
    if (max_period < min_period || max_period != options[1]) return TI_INVALID_OPTION;
    if (ma_type < 0 || ma_type > 8 || ma_type != options[2]) return TI_INVALID_OPTION;

    const int lookback = ti_ma_dispatch_start(max_period, ma_type);
    if (lookback < 0) return TI_INVALID_OPTION;
    if (size <= lookback) return TI_OKAY;

    const int output_len = size - lookback;
    int unique_count = 0;
    int unique_periods[TI_MAXINDPARAMS];
    TI_REAL *series[TI_MAXINDPARAMS];
    int series_lookbacks[TI_MAXINDPARAMS];

    for (int out_index = 0; out_index < output_len; ++out_index) {
        int period = (int)periods[lookback + out_index];
        if (period < min_period) period = min_period;
        if (period > max_period) period = max_period;

        int slot = -1;
        for (int i = 0; i < unique_count; ++i) {
            if (unique_periods[i] == period) {
                slot = i;
                break;
            }
        }

        if (slot == -1) {
            if (unique_count >= TI_MAXINDPARAMS) {
                for (int i = 0; i < unique_count; ++i) free(series[i]);
                return TI_OUT_OF_MEMORY;
            }

            const int current_lookback = ti_ma_dispatch_start(period, ma_type);
            const int current_len = size - current_lookback;
            TI_REAL *current_series = malloc((unsigned int)current_len * sizeof(TI_REAL));
            if (!current_series) {
                for (int i = 0; i < unique_count; ++i) free(series[i]);
                return TI_OUT_OF_MEMORY;
            }
            const int rc = ti_ma_dispatch_run(size, input, period, ma_type, current_series);
            if (rc != TI_OKAY) {
                free(current_series);
                for (int i = 0; i < unique_count; ++i) free(series[i]);
                return rc;
            }

            slot = unique_count++;
            unique_periods[slot] = period;
            series[slot] = current_series;
            series_lookbacks[slot] = current_lookback;
        }

        output[out_index] = series[slot][lookback + out_index - series_lookbacks[slot]];
    }

    for (int i = 0; i < unique_count; ++i) free(series[i]);
    return TI_OKAY;
}
