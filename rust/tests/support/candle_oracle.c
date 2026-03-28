#include "candles.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int read_value(FILE *fp, double *out) {
    return fscanf(fp, " %lf", out) == 1;
}

static int print_metadata(void) {
    const int count = tc_candle_count();
    printf("%d\n", count);
    for (int i = 0; i < count; ++i) {
        printf("%s\t%s\t%llu\n",
               tc_candles[i].name,
               tc_candles[i].full_name,
               (unsigned long long)tc_candles[i].pattern);
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "--metadata") == 0) {
        return print_metadata();
    }

    unsigned long long patterns = 0;
    int input_len = 0;
    int period = 0;
    tc_config config;
    if (scanf("%llu %d %d %lf %lf %lf %lf %lf %lf",
              &patterns, &input_len, &period,
              &config.body_none, &config.body_short, &config.body_long,
              &config.wick_none, &config.wick_long, &config.near) != 9) {
        fprintf(stderr, "failed to read header\n");
        return 2;
    }
    config.period = period;

    double *inputs[4] = {0};
    for (int i = 0; i < 4; ++i) {
        inputs[i] = calloc((size_t)input_len, sizeof(double));
        for (int j = 0; j < input_len; ++j) {
            if (!read_value(stdin, &inputs[i][j])) {
                fprintf(stderr, "failed to read input %d[%d]\n", i, j);
                return 2;
            }
        }
    }

    tc_result *result = tc_result_new();
    if (!result) {
        fprintf(stderr, "failed to allocate result\n");
        return 3;
    }

    const int ret = tc_run((tc_set)patterns, input_len, (const double *const *)inputs, &config, result);
    if (ret != TC_OKAY) {
        fprintf(stderr, "tc_run failed: %d\n", ret);
        return 3;
    }

    tc_set *sets = calloc((size_t)input_len, sizeof(tc_set));
    if (!sets) {
        fprintf(stderr, "failed to allocate candle set buffer\n");
        return 3;
    }

    const int count = tc_result_count(result);
    for (int i = 0; i < count; ++i) {
        const tc_hit hit = tc_result_get(result, i);
        if (hit.index >= 0 && hit.index < input_len) {
            sets[hit.index] = hit.patterns;
        }
    }

    printf("%d\n", input_len);
    for (int i = 0; i < input_len; ++i) {
        printf("%llu\n", (unsigned long long)sets[i]);
    }

    free(sets);
    tc_result_free(result);
    for (int i = 0; i < 4; ++i) {
        free(inputs[i]);
    }
    return 0;
}
