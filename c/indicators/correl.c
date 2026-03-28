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


int ti_correl_start(TI_REAL const *options) {
    return (int)options[0]-1;
}


int ti_correl(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input0 = inputs[0];
    const TI_REAL *input1 = inputs[1];
    const int period = (int)options[0];
    TI_REAL *output = outputs[0];

    if (period < 1) return TI_INVALID_OPTION;
    if (size <= ti_correl_start(options)) return TI_OKAY;

    const TI_REAL n = (TI_REAL)period;
    TI_REAL sx = 0.0;
    TI_REAL sy = 0.0;
    TI_REAL sxx = 0.0;
    TI_REAL syy = 0.0;
    TI_REAL sxy = 0.0;

    for (int i = 0; i < period; ++i) {
        const TI_REAL x = input0[i];
        const TI_REAL y = input1[i];
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
        sxy += x * y;
    }

    TI_REAL xdiff = n * sxx - sx * sx;
    TI_REAL ydiff = n * syy - sy * sy;
    TI_REAL denom = xdiff * ydiff;
    *output++ = denom > 0.0 ? (n * sxy - sx * sy) / sqrt(denom) : 0.0;

    for (int end = period; end < size; ++end) {
        const TI_REAL add_x = input0[end];
        const TI_REAL add_y = input1[end];
        const TI_REAL sub_x = input0[end - period];
        const TI_REAL sub_y = input1[end - period];

        sx += add_x - sub_x;
        sy += add_y - sub_y;
        sxx += add_x * add_x - sub_x * sub_x;
        syy += add_y * add_y - sub_y * sub_y;
        sxy += add_x * add_y - sub_x * sub_y;

        xdiff = n * sxx - sx * sx;
        ydiff = n * syy - sy * sy;
        denom = xdiff * ydiff;
        *output++ = denom > 0.0 ? (n * sxy - sx * sy) / sqrt(denom) : 0.0;
    }

    assert(output - outputs[0] == size - ti_correl_start(options));
    return TI_OKAY;
}
