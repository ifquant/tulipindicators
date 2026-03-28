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


int ti_beta_start(TI_REAL const *options) {
    return (int)options[0];
}


static TI_REAL rate_of_change(TI_REAL current, TI_REAL previous) {
    if (previous != 0.0) {
        return (current - previous) / previous;
    }
    return 0.0;
}


int ti_beta(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input0 = inputs[0];
    const TI_REAL *input1 = inputs[1];
    const int period = (int)options[0];
    TI_REAL *output = outputs[0];

    if (period < 1) return TI_INVALID_OPTION;
    if (size <= ti_beta_start(options)) return TI_OKAY;

    const TI_REAL n = (TI_REAL)period;
    TI_REAL sx = 0.0;
    TI_REAL sy = 0.0;
    TI_REAL sxx = 0.0;
    TI_REAL sxy = 0.0;

    for (int i = 1; i <= period; ++i) {
        const TI_REAL x = rate_of_change(input0[i], input0[i-1]);
        const TI_REAL y = rate_of_change(input1[i], input1[i-1]);
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
    }

    TI_REAL denom = n * sxx - sx * sx;
    *output++ = denom != 0.0 ? (n * sxy - sx * sy) / denom : 0.0;

    for (int end = period + 1; end < size; ++end) {
        const TI_REAL add_x = rate_of_change(input0[end], input0[end-1]);
        const TI_REAL add_y = rate_of_change(input1[end], input1[end-1]);
        const int trailing = end - period;
        const TI_REAL sub_x = rate_of_change(input0[trailing], input0[trailing-1]);
        const TI_REAL sub_y = rate_of_change(input1[trailing], input1[trailing-1]);

        sx += add_x - sub_x;
        sy += add_y - sub_y;
        sxx += add_x * add_x - sub_x * sub_x;
        sxy += add_x * add_y - sub_x * sub_y;

        denom = n * sxx - sx * sx;
        *output++ = denom != 0.0 ? (n * sxy - sx * sy) / denom : 0.0;
    }

    assert(output - outputs[0] == size - ti_beta_start(options));
    return TI_OKAY;
}
