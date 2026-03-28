/*
 * Tulip Indicators
 * https://tulipindicators.org/
 * Copyright (c) 2010-2026 Tulip Charts LLC
 *
 * This file is part of Tulip Indicators.
 */

#include "../indicators.h"

#define TI_MA_TYPE_SMA 0
#define TI_MA_TYPE_EMA 1
#define TI_MA_TYPE_WMA 2
#define TI_MA_TYPE_DEMA 3
#define TI_MA_TYPE_TEMA 4
#define TI_MA_TYPE_TRIMA 5
#define TI_MA_TYPE_KAMA 6
#define TI_MA_TYPE_MAMA 7
#define TI_MA_TYPE_T3 8

int ti_ma_dispatch_start(int period, int ma_type) {
    const TI_REAL period_option[] = {(TI_REAL)period};
    const TI_REAL t3_options[] = {(TI_REAL)period, 0.7};
    const TI_REAL mama_options[] = {0.5, 0.05};

    if (period <= 1) {
        return 0;
    }

    switch (ma_type) {
        case TI_MA_TYPE_SMA: return ti_sma_start(period_option);
        case TI_MA_TYPE_EMA: return ti_ema_start(period_option);
        case TI_MA_TYPE_WMA: return ti_wma_start(period_option);
        case TI_MA_TYPE_DEMA: return ti_dema_start(period_option);
        case TI_MA_TYPE_TEMA: return ti_tema_start(period_option);
        case TI_MA_TYPE_TRIMA: return ti_trima_start(period_option);
        case TI_MA_TYPE_KAMA: return ti_kama_start(period_option);
        case TI_MA_TYPE_MAMA: return ti_mama_start(mama_options);
        case TI_MA_TYPE_T3: return ti_t3_start(t3_options);
        default: return -1;
    }
}

int ti_ma_dispatch_run(
    int size,
    TI_REAL const *input,
    int period,
    int ma_type,
    TI_REAL *output
) {
    const TI_REAL *inputs[] = {input};
    TI_REAL *outputs[] = {output};
    const TI_REAL period_option[] = {(TI_REAL)period};
    const TI_REAL t3_options[] = {(TI_REAL)period, 0.7};
    const TI_REAL mama_options[] = {0.5, 0.05};

    switch (ma_type) {
        case TI_MA_TYPE_SMA:
            return ti_sma(size, inputs, period_option, outputs);
        case TI_MA_TYPE_EMA:
            return ti_ema(size, inputs, period_option, outputs);
        case TI_MA_TYPE_WMA:
            return ti_wma(size, inputs, period_option, outputs);
        case TI_MA_TYPE_DEMA:
            return ti_dema(size, inputs, period_option, outputs);
        case TI_MA_TYPE_TEMA:
            return ti_tema(size, inputs, period_option, outputs);
        case TI_MA_TYPE_TRIMA:
            return ti_trima(size, inputs, period_option, outputs);
        case TI_MA_TYPE_KAMA:
            return ti_kama(size, inputs, period_option, outputs);
        case TI_MA_TYPE_T3:
            return ti_t3(size, inputs, t3_options, outputs);
        case TI_MA_TYPE_MAMA: {
            const int lookback = ti_mama_start(mama_options);
            if (size <= lookback) {
                return TI_OKAY;
            }

            const int output_len = size - lookback;
            TI_REAL *fama = malloc((unsigned int)output_len * sizeof(TI_REAL));
            if (!fama) {
                return TI_OUT_OF_MEMORY;
            }
            TI_REAL *mama_outputs[] = {output, fama};
            const int rc = ti_mama(size, inputs, mama_options, mama_outputs);
            free(fama);
            return rc;
        }
        default:
            return TI_INVALID_OPTION;
    }
}

int ti_ma_start(TI_REAL const *options) {
    const int period = (int)options[0];
    const int ma_type = (int)options[1];

    if (period < 1) {
        return -1;
    }
    if (ma_type < TI_MA_TYPE_SMA || ma_type > TI_MA_TYPE_T3) {
        return -1;
    }

    return ti_ma_dispatch_start(period, ma_type);
}

int ti_ma(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const int period = (int)options[0];
    const int ma_type = (int)options[1];

    if (period < 1 || period != options[0]) {
        return TI_INVALID_OPTION;
    }
    if (ma_type < TI_MA_TYPE_SMA || ma_type > TI_MA_TYPE_T3 || ma_type != options[1]) {
        return TI_INVALID_OPTION;
    }

    return ti_ma_dispatch_run(size, inputs[0], period, ma_type, outputs[0]);
}
