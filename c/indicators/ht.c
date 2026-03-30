/*
 * Tulip Indicators
 * https://tulipindicators.org/
 * Copyright (c) 2010-2026 Tulip Charts LLC
 *
 * This file is part of Tulip Indicators.
 */

#include "../indicators.h"

#include <math.h>
#include <string.h>

#define HT_SHORT_LOOKBACK 32
#define HT_LONG_LOOKBACK 63
#define HT_SMOOTH_PRICE_SIZE 50
#define HT_A 0.0962
#define HT_B 0.5769

typedef struct {
    TI_REAL odd[3];
    TI_REAL even[3];
    TI_REAL prev_odd;
    TI_REAL prev_even;
    TI_REAL prev_input_odd;
    TI_REAL prev_input_even;
} ht_history;

typedef struct {
    int trailing_idx;
    TI_REAL period_wma_sub;
    TI_REAL period_wma_sum;
    TI_REAL trailing_wma_value;
} ht_price_wma;

enum ht_short_mode {
    HT_SHORT_DCPERIOD,
    HT_SHORT_PHASOR,
};

enum ht_long_mode {
    HT_LONG_DCPHASE,
    HT_LONG_SINE,
    HT_LONG_TRENDLINE,
    HT_LONG_TRENDMODE,
};

static void ht_history_init(ht_history *history) {
    memset(history, 0, sizeof(*history));
}

static TI_REAL ht_history_step(
    ht_history *history,
    int odd_bar,
    int hilbert_idx,
    TI_REAL input,
    TI_REAL adjusted_prev_period
) {
    const TI_REAL hilbert_temp = HT_A * input;
    TI_REAL value;

    if (odd_bar) {
        value = -history->odd[hilbert_idx];
        history->odd[hilbert_idx] = hilbert_temp;
        value += hilbert_temp;
        value -= history->prev_odd;
        history->prev_odd = HT_B * history->prev_input_odd;
        value += history->prev_odd;
        history->prev_input_odd = input;
    } else {
        value = -history->even[hilbert_idx];
        history->even[hilbert_idx] = hilbert_temp;
        value += hilbert_temp;
        value -= history->prev_even;
        history->prev_even = HT_B * history->prev_input_even;
        value += history->prev_even;
        history->prev_input_even = input;
    }

    return value * adjusted_prev_period;
}

static int ht_price_wma_init(
    ht_price_wma *state,
    const TI_REAL *input,
    int start_idx,
    int lookback
) {
    int today;
    const int trailing_idx = start_idx - lookback;
    const TI_REAL first = input[trailing_idx];
    const TI_REAL second = input[trailing_idx + 1];
    const TI_REAL third = input[trailing_idx + 2];

    state->trailing_idx = trailing_idx;
    state->period_wma_sub = first + second + third;
    state->period_wma_sum = first + second * 2.0 + third * 3.0;
    state->trailing_wma_value = 0.0;
    today = trailing_idx + 3;
    return today;
}

static TI_REAL ht_price_wma_step(
    ht_price_wma *state,
    const TI_REAL *input,
    TI_REAL new_price
) {
    TI_REAL smoothed;
    state->period_wma_sub += new_price;
    state->period_wma_sub -= state->trailing_wma_value;
    state->period_wma_sum += new_price * 4.0;
    state->trailing_wma_value = input[state->trailing_idx++];
    smoothed = state->period_wma_sum * 0.1;
    state->period_wma_sum -= state->period_wma_sub;
    return smoothed;
}

static int run_ht_short(
    int size,
    const TI_REAL *input,
    enum ht_short_mode mode,
    TI_REAL *output0,
    TI_REAL *output1
) {
    int today;
    int out_idx;
    int hilbert_idx;
    ht_price_wma price_wma;
    ht_history detrender, q1, ji, jq;
    TI_REAL period, smooth_period, prev_q2, prev_i2, re, im;
    TI_REAL i1_odd_prev2, i1_odd_prev3, i1_even_prev2, i1_even_prev3;
    TI_REAL smoothed, adjusted_prev_period, today_value;
    const TI_REAL rad2deg = 180.0 / (4.0 * atan(1.0));

    if (size <= HT_SHORT_LOOKBACK) {
        return 0;
    }

    today = ht_price_wma_init(&price_wma, input, HT_SHORT_LOOKBACK, HT_SHORT_LOOKBACK);
    for (int i = 0; i < 9; ++i) {
        smoothed = ht_price_wma_step(&price_wma, input, input[today++]);
    }

    hilbert_idx = 0;
    out_idx = 0;
    period = 0.0;
    smooth_period = 0.0;
    prev_q2 = prev_i2 = re = im = 0.0;
    i1_odd_prev2 = i1_odd_prev3 = 0.0;
    i1_even_prev2 = i1_even_prev3 = 0.0;
    ht_history_init(&detrender);
    ht_history_init(&q1);
    ht_history_init(&ji);
    ht_history_init(&jq);

    while (today < size) {
        TI_REAL previous_period, upper, lower;
        adjusted_prev_period = 0.075 * period + 0.54;
        today_value = input[today];
        smoothed = ht_price_wma_step(&price_wma, input, today_value);

        if ((today % 2) == 0) {
            const TI_REAL detrender_value =
                ht_history_step(&detrender, 0, hilbert_idx, smoothed, adjusted_prev_period);
            const TI_REAL q1_value =
                ht_history_step(&q1, 0, hilbert_idx, detrender_value, adjusted_prev_period);
            if (mode == HT_SHORT_PHASOR && today >= HT_SHORT_LOOKBACK) {
                output0[out_idx] = i1_even_prev3;
                output1[out_idx++] = q1_value;
            }
            const TI_REAL ji_value =
                ht_history_step(&ji, 0, hilbert_idx, i1_even_prev3, adjusted_prev_period);
            const TI_REAL jq_value =
                ht_history_step(&jq, 0, hilbert_idx, q1_value, adjusted_prev_period);
            const TI_REAL q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            const TI_REAL i2 = 0.2 * (i1_even_prev3 - jq_value) + 0.8 * prev_i2;

            if (++hilbert_idx == 3) {
                hilbert_idx = 0;
            }

            i1_odd_prev3 = i1_odd_prev2;
            i1_odd_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        } else {
            const TI_REAL detrender_value =
                ht_history_step(&detrender, 1, hilbert_idx, smoothed, adjusted_prev_period);
            const TI_REAL q1_value =
                ht_history_step(&q1, 1, hilbert_idx, detrender_value, adjusted_prev_period);
            if (mode == HT_SHORT_PHASOR && today >= HT_SHORT_LOOKBACK) {
                output0[out_idx] = i1_odd_prev3;
                output1[out_idx++] = q1_value;
            }
            const TI_REAL ji_value =
                ht_history_step(&ji, 1, hilbert_idx, i1_odd_prev3, adjusted_prev_period);
            const TI_REAL jq_value =
                ht_history_step(&jq, 1, hilbert_idx, q1_value, adjusted_prev_period);
            const TI_REAL q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            const TI_REAL i2 = 0.2 * (i1_odd_prev3 - jq_value) + 0.8 * prev_i2;

            i1_even_prev3 = i1_even_prev2;
            i1_even_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        }

        previous_period = period;
        if (im != 0.0 && re != 0.0) {
            period = 360.0 / (atan(im / re) * rad2deg);
        }
        upper = 1.5 * previous_period;
        if (period > upper) period = upper;
        lower = 0.67 * previous_period;
        if (period < lower) period = lower;
        if (period < 6.0) period = 6.0;
        else if (period > 50.0) period = 50.0;
        period = 0.2 * period + 0.8 * previous_period;

        if (mode == HT_SHORT_DCPERIOD) {
            smooth_period = 0.33 * period + 0.67 * smooth_period;
            if (today >= HT_SHORT_LOOKBACK) {
                output0[out_idx++] = smooth_period;
            }
        }

        today++;
    }

    return out_idx;
}

static int run_ht_long(
    int size,
    const TI_REAL *input,
    enum ht_long_mode mode,
    TI_REAL *output0,
    TI_REAL *output1
) {
    int today;
    int out_idx;
    int hilbert_idx;
    int smooth_price_idx;
    ht_price_wma price_wma;
    ht_history detrender, q1, ji, jq;
    TI_REAL period, smooth_period, prev_q2, prev_i2, re, im;
    TI_REAL i1_odd_prev2, i1_odd_prev3, i1_even_prev2, i1_even_prev3;
    TI_REAL smoothed, adjusted_prev_period, today_value;
    TI_REAL smooth_price[HT_SMOOTH_PRICE_SIZE];
    TI_REAL dc_phase, prev_dc_phase;
    TI_REAL i_trend1, i_trend2, i_trend3;
    int days_in_trend;
    TI_REAL prev_sine, prev_lead_sine, sine, lead_sine;
    const TI_REAL temp_real = atan(1.0);
    const TI_REAL rad2deg = 45.0 / temp_real;
    const TI_REAL deg2rad = 1.0 / rad2deg;
    const TI_REAL const_deg2rad_by_360 = temp_real * 8.0;

    if (size <= HT_LONG_LOOKBACK) {
        return 0;
    }

    today = ht_price_wma_init(&price_wma, input, HT_LONG_LOOKBACK, HT_LONG_LOOKBACK);
    for (int i = 0; i < 34; ++i) {
        smoothed = ht_price_wma_step(&price_wma, input, input[today++]);
    }

    hilbert_idx = 0;
    smooth_price_idx = 0;
    out_idx = 0;
    period = 0.0;
    smooth_period = 0.0;
    prev_q2 = prev_i2 = re = im = 0.0;
    i1_odd_prev2 = i1_odd_prev3 = 0.0;
    i1_even_prev2 = i1_even_prev3 = 0.0;
    dc_phase = prev_dc_phase = 0.0;
    i_trend1 = i_trend2 = i_trend3 = 0.0;
    days_in_trend = 0;
    prev_sine = prev_lead_sine = sine = lead_sine = 0.0;
    memset(smooth_price, 0, sizeof(smooth_price));
    ht_history_init(&detrender);
    ht_history_init(&q1);
    ht_history_init(&ji);
    ht_history_init(&jq);

    while (today < size) {
        TI_REAL previous_period, upper, lower;
        TI_REAL trendline = 0.0;
        adjusted_prev_period = 0.075 * period + 0.54;
        today_value = input[today];
        smoothed = ht_price_wma_step(&price_wma, input, today_value);
        smooth_price[smooth_price_idx] = smoothed;

        if ((today % 2) == 0) {
            const TI_REAL detrender_value =
                ht_history_step(&detrender, 0, hilbert_idx, smoothed, adjusted_prev_period);
            const TI_REAL q1_value =
                ht_history_step(&q1, 0, hilbert_idx, detrender_value, adjusted_prev_period);
            const TI_REAL ji_value =
                ht_history_step(&ji, 0, hilbert_idx, i1_even_prev3, adjusted_prev_period);
            const TI_REAL jq_value =
                ht_history_step(&jq, 0, hilbert_idx, q1_value, adjusted_prev_period);
            const TI_REAL q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            const TI_REAL i2 = 0.2 * (i1_even_prev3 - jq_value) + 0.8 * prev_i2;

            if (++hilbert_idx == 3) {
                hilbert_idx = 0;
            }

            i1_odd_prev3 = i1_odd_prev2;
            i1_odd_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        } else {
            const TI_REAL detrender_value =
                ht_history_step(&detrender, 1, hilbert_idx, smoothed, adjusted_prev_period);
            const TI_REAL q1_value =
                ht_history_step(&q1, 1, hilbert_idx, detrender_value, adjusted_prev_period);
            const TI_REAL ji_value =
                ht_history_step(&ji, 1, hilbert_idx, i1_odd_prev3, adjusted_prev_period);
            const TI_REAL jq_value =
                ht_history_step(&jq, 1, hilbert_idx, q1_value, adjusted_prev_period);
            const TI_REAL q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            const TI_REAL i2 = 0.2 * (i1_odd_prev3 - jq_value) + 0.8 * prev_i2;

            i1_even_prev3 = i1_even_prev2;
            i1_even_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        }

        previous_period = period;
        if (im != 0.0 && re != 0.0) {
            period = 360.0 / (atan(im / re) * rad2deg);
        }
        upper = 1.5 * previous_period;
        if (period > upper) period = upper;
        lower = 0.67 * previous_period;
        if (period < lower) period = lower;
        if (period < 6.0) period = 6.0;
        else if (period > 50.0) period = 50.0;
        period = 0.2 * period + 0.8 * previous_period;
        smooth_period = 0.33 * period + 0.67 * smooth_period;

        if (mode == HT_LONG_DCPHASE || mode == HT_LONG_SINE || mode == HT_LONG_TRENDMODE) {
            int idx;
            TI_REAL real_part = 0.0;
            TI_REAL imag_part = 0.0;
            const TI_REAL dc_period = smooth_period + 0.5;
            const int dc_period_int = (int)dc_period;

            prev_dc_phase = dc_phase;
            idx = smooth_price_idx;
            for (int i = 0; i < dc_period_int; ++i) {
                const TI_REAL angle = ((TI_REAL)i * const_deg2rad_by_360) / (TI_REAL)dc_period_int;
                const TI_REAL value = smooth_price[idx];
                real_part += sin(angle) * value;
                imag_part += cos(angle) * value;
                if (idx == 0) idx = HT_SMOOTH_PRICE_SIZE - 1;
                else idx--;
            }

            {
                const TI_REAL imag_abs = fabs(imag_part);
                if (imag_abs > 0.0) {
                    dc_phase = atan(real_part / imag_part) * rad2deg;
                } else if (imag_abs <= 0.01) {
                    if (real_part < 0.0) dc_phase -= 90.0;
                    else if (real_part > 0.0) dc_phase += 90.0;
                }
            }

            dc_phase += 90.0;
            dc_phase += 360.0 / smooth_period;
            if (imag_part < 0.0) dc_phase += 180.0;
            if (dc_phase > 315.0) dc_phase -= 360.0;
        }

        if (mode == HT_LONG_TRENDLINE || mode == HT_LONG_TRENDMODE) {
            TI_REAL average = 0.0;
            const TI_REAL dc_period = smooth_period + 0.5;
            const int dc_period_int = (int)dc_period;
            int idx = today;
            for (int i = 0; i < dc_period_int; ++i) {
                average += input[idx--];
            }
            if (dc_period_int > 0) {
                average /= (TI_REAL)dc_period_int;
            }
            trendline = (4.0 * average + 3.0 * i_trend1 + 2.0 * i_trend2 + i_trend3) / 10.0;
            i_trend3 = i_trend2;
            i_trend2 = i_trend1;
            i_trend1 = average;
        }

        if (today >= HT_LONG_LOOKBACK) {
            if (mode == HT_LONG_DCPHASE) {
                output0[out_idx++] = dc_phase;
            } else if (mode == HT_LONG_SINE) {
                output0[out_idx] = sin(dc_phase * deg2rad);
                output1[out_idx++] = sin((dc_phase + 45.0) * deg2rad);
            } else if (mode == HT_LONG_TRENDLINE) {
                output0[out_idx++] = trendline;
            } else {
                TI_REAL trend = 1.0;
                prev_sine = sine;
                prev_lead_sine = lead_sine;
                sine = sin(dc_phase * deg2rad);
                lead_sine = sin((dc_phase + 45.0) * deg2rad);

                if (((sine > lead_sine) && (prev_sine <= prev_lead_sine)) ||
                    ((sine < lead_sine) && (prev_sine >= prev_lead_sine))) {
                    days_in_trend = 0;
                    trend = 0.0;
                }

                days_in_trend++;
                if ((TI_REAL)days_in_trend < 0.5 * smooth_period) {
                    trend = 0.0;
                }

                {
                    const TI_REAL temp = dc_phase - prev_dc_phase;
                    if (smooth_period != 0.0 &&
                        temp > (0.67 * 360.0 / smooth_period) &&
                        temp < (1.5 * 360.0 / smooth_period)) {
                        trend = 0.0;
                    }
                }

                {
                    const TI_REAL smoothed_price = smooth_price[smooth_price_idx];
                    if (trendline != 0.0 && fabs((smoothed_price - trendline) / trendline) >= 0.015) {
                        trend = 1.0;
                    }
                }

                output0[out_idx++] = trend;
            }
        }

        smooth_price_idx = (smooth_price_idx + 1) % HT_SMOOTH_PRICE_SIZE;
        today++;
    }

    return out_idx;
}

int ti_ht_dcperiod_start(TI_REAL const *options) {
    (void)options;
    return HT_SHORT_LOOKBACK;
}

int ti_ht_dcphase_start(TI_REAL const *options) {
    (void)options;
    return HT_LONG_LOOKBACK;
}

int ti_ht_phasor_start(TI_REAL const *options) {
    (void)options;
    return HT_SHORT_LOOKBACK;
}

int ti_ht_sine_start(TI_REAL const *options) {
    (void)options;
    return HT_LONG_LOOKBACK;
}

int ti_ht_trendline_start(TI_REAL const *options) {
    (void)options;
    return HT_LONG_LOOKBACK;
}

int ti_ht_trendmode_start(TI_REAL const *options) {
    (void)options;
    return HT_LONG_LOOKBACK;
}

int ti_ht_dcperiod(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_short(size, inputs[0], HT_SHORT_DCPERIOD, outputs[0], 0);
    return TI_OKAY;
}

int ti_ht_phasor(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_short(size, inputs[0], HT_SHORT_PHASOR, outputs[0], outputs[1]);
    return TI_OKAY;
}

int ti_ht_dcphase(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_long(size, inputs[0], HT_LONG_DCPHASE, outputs[0], 0);
    return TI_OKAY;
}

int ti_ht_sine(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_long(size, inputs[0], HT_LONG_SINE, outputs[0], outputs[1]);
    return TI_OKAY;
}

int ti_ht_trendline(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_long(size, inputs[0], HT_LONG_TRENDLINE, outputs[0], 0);
    return TI_OKAY;
}

int ti_ht_trendmode(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    (void)options;
    run_ht_long(size, inputs[0], HT_LONG_TRENDMODE, outputs[0], 0);
    return TI_OKAY;
}
