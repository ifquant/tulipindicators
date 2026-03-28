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


int ti_midpoint_start(TI_REAL const *options) {
    return (int)options[0]-1;
}


int ti_midpoint(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs) {
    const TI_REAL *input = inputs[0];
    const int period = (int)options[0];
    TI_REAL *output = outputs[0];

    if (period < 1) return TI_INVALID_OPTION;
    if (size <= ti_midpoint_start(options)) return TI_OKAY;

    int trail = 0;
    int maxi = -1;
    int mini = -1;
    TI_REAL max = input[0];
    TI_REAL min = input[0];

    for (int i = period-1; i < size; ++i, ++trail) {
        const TI_REAL value = input[i];

        if (maxi < trail) {
            maxi = trail;
            max = input[maxi];
            for (int j = trail + 1; j <= i; ++j) {
                if (input[j] >= max) {
                    max = input[j];
                    maxi = j;
                }
            }
        } else if (value >= max) {
            max = value;
            maxi = i;
        }

        if (mini < trail) {
            mini = trail;
            min = input[mini];
            for (int j = trail + 1; j <= i; ++j) {
                if (input[j] <= min) {
                    min = input[j];
                    mini = j;
                }
            }
        } else if (value <= min) {
            min = value;
            mini = i;
        }

        *output++ = (max + min) * 0.5;
    }

    assert(output - outputs[0] == size - ti_midpoint_start(options));
    return TI_OKAY;
}
