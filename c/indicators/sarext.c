/*
 * Tulip Indicators
 * https://tulipindicators.org/
 * Copyright (c) 2010-2017 Tulip Charts LLC
 * Lewis Van Winkle (LV@tulipcharts.org)
 *
 * This file is part of Tulip Indicators.
 *
 * Tulip Indicators is free software: you can redistribute it and/or modify it
 * under the terms of the GNU Lesser General Public License as published by the
 * Free Software Foundation, either version 3 of the License, or (at your
 * option) any later version.
 *
 * Tulip Indicators is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public License
 * for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with Tulip Indicators.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

#include "../indicators.h"

#include <math.h>


int ti_sarext_start(TI_REAL const *options) {
    (void)options;
    return 1;
}


static int sarext_validate_options(
    TI_REAL start_value,
    TI_REAL offset_on_reverse,
    TI_REAL acceleration_init_long,
    TI_REAL acceleration_long,
    TI_REAL acceleration_max_long,
    TI_REAL acceleration_init_short,
    TI_REAL acceleration_short,
    TI_REAL acceleration_max_short
) {
    if (!isfinite(start_value)) return TI_INVALID_OPTION;
    if (!isfinite(offset_on_reverse) || offset_on_reverse < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_init_long) || acceleration_init_long < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_long) || acceleration_long < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_max_long) || acceleration_max_long < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_init_short) || acceleration_init_short < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_short) || acceleration_short < 0.0) return TI_INVALID_OPTION;
    if (!isfinite(acceleration_max_short) || acceleration_max_short < 0.0) return TI_INVALID_OPTION;
    return TI_OKAY;
}


int ti_sarext(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *high = inputs[0];
    const TI_REAL *low = inputs[1];
    TI_REAL *output = outputs[0];

    TI_REAL start_value = options[0];
    const TI_REAL offset_on_reverse = options[1];
    TI_REAL acceleration_init_long = options[2];
    TI_REAL acceleration_long = options[3];
    const TI_REAL acceleration_max_long = options[4];
    TI_REAL acceleration_init_short = options[5];
    TI_REAL acceleration_short = options[6];
    const TI_REAL acceleration_max_short = options[7];

    const int options_rc = sarext_validate_options(
        start_value,
        offset_on_reverse,
        acceleration_init_long,
        acceleration_long,
        acceleration_max_long,
        acceleration_init_short,
        acceleration_short,
        acceleration_max_short
    );
    if (options_rc != TI_OKAY) return options_rc;
    if (size < 2) return TI_OKAY;

    if (acceleration_init_long > acceleration_max_long) {
        acceleration_init_long = acceleration_max_long;
    }
    if (acceleration_long > acceleration_max_long) {
        acceleration_long = acceleration_max_long;
    }
    if (acceleration_init_short > acceleration_max_short) {
        acceleration_init_short = acceleration_max_short;
    }
    if (acceleration_short > acceleration_max_short) {
        acceleration_short = acceleration_max_short;
    }

    int is_long;
    if (start_value == 0.0) {
        const TI_REAL up_move = high[1] - high[0];
        const TI_REAL down_move = low[0] - low[1];
        is_long = !(down_move > up_move && down_move > 0.0);
    } else {
        is_long = start_value > 0.0;
    }

    int today = 1;
    TI_REAL new_high = high[today - 1];
    TI_REAL new_low = low[today - 1];
    TI_REAL prev_high;
    TI_REAL prev_low;
    TI_REAL ep;
    TI_REAL sar;
    TI_REAL af_long = acceleration_init_long;
    TI_REAL af_short = acceleration_init_short;

    if (start_value == 0.0) {
        if (is_long) {
            ep = high[today];
            sar = new_low;
        } else {
            ep = low[today];
            sar = new_high;
        }
    } else if (start_value > 0.0) {
        ep = high[today];
        sar = start_value;
    } else {
        ep = low[today];
        sar = fabs(start_value);
    }

    new_high = high[today];
    new_low = low[today];

    while (today < size) {
        prev_high = new_high;
        prev_low = new_low;
        new_high = high[today];
        new_low = low[today];
        ++today;

        if (is_long) {
            if (new_low <= sar) {
                is_long = 0;
                sar = ep;

                if (sar < prev_high) sar = prev_high;
                if (sar < new_high) sar = new_high;

                if (offset_on_reverse != 0.0) sar += sar * offset_on_reverse;
                *output++ = -sar;

                af_short = acceleration_init_short;
                ep = new_low;

                sar = sar + af_short * (ep - sar);
                if (sar < prev_high) sar = prev_high;
                if (sar < new_high) sar = new_high;
            } else {
                *output++ = sar;

                if (new_high > ep) {
                    ep = new_high;
                    af_long += acceleration_long;
                    if (af_long > acceleration_max_long) af_long = acceleration_max_long;
                }

                sar = sar + af_long * (ep - sar);
                if (sar > prev_low) sar = prev_low;
                if (sar > new_low) sar = new_low;
            }
        } else {
            if (new_high >= sar) {
                is_long = 1;
                sar = ep;

                if (sar > prev_low) sar = prev_low;
                if (sar > new_low) sar = new_low;

                if (offset_on_reverse != 0.0) sar -= sar * offset_on_reverse;
                *output++ = sar;

                af_long = acceleration_init_long;
                ep = new_high;

                sar = sar + af_long * (ep - sar);
                if (sar > prev_low) sar = prev_low;
                if (sar > new_low) sar = new_low;
            } else {
                *output++ = -sar;

                if (new_low < ep) {
                    ep = new_low;
                    af_short += acceleration_short;
                    if (af_short > acceleration_max_short) af_short = acceleration_max_short;
                }

                sar = sar + af_short * (ep - sar);
                if (sar < prev_high) sar = prev_high;
                if (sar < new_high) sar = new_high;
            }
        }
    }

    assert(output - outputs[0] == size - ti_sarext_start(options));
    return TI_OKAY;
}
